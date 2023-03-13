use actix_cors::Cors;
use actix_web::{
    get, http,
    middleware::Logger,
    rt::spawn,
    web::{Data, Path, Query},
    App, HttpResponse, HttpResponseBuilder, HttpServer, Responder,
};

use dtos::{ChatMessage, GetChatDto};
use events::{ChatCreatedEvent, ChatMessageSentEvent};
use eventstore::{Client, RecordedEvent, StreamPosition, SubscribeToStreamOptions};
use eyre::{Error, Result};
use log::{error, info};
use serde::{Deserialize, Serialize};
use serde_json::{json, to_string};
use std::{
    collections::HashMap,
    env,
    fmt::Display,
    sync::{Arc, Mutex},
};

trait ContentTypeJsonExt {
    fn content_type_json(&mut self) -> &mut Self;
}

impl ContentTypeJsonExt for HttpResponseBuilder {
    fn content_type_json(&mut self) -> &mut Self {
        self.content_type("application/json")
    }
}

trait NotFoundExt {
    fn to_not_found_res(self) -> HttpResponse;
}

impl<Data: Display> NotFoundExt for Data {
    fn to_not_found_res(self) -> HttpResponse {
        let json = json!({
            "message": self.to_string()
        })
        .to_string();
        HttpResponse::NotFound().content_type_json().body(json)
    }
}

trait OkExt {
    fn to_ok_res(&self) -> HttpResponse;
}

impl<Body: Serialize> OkExt for Body {
    fn to_ok_res(&self) -> HttpResponse {
        let json = match to_string(&self) {
            Ok(it) => it,
            Err(_) => {
                return HttpResponse::InternalServerError()
                    .content_type_json()
                    .body(
                        json!({
                            "message":"Internal server error"
                        })
                        .to_string(),
                    )
            }
        };
        HttpResponse::Ok().content_type_json().body(json)
    }
}

#[derive(Serialize, Debug, Clone)]
struct Chat {
    chat_id: String,
    subject: String,
}

#[derive(Debug)]
struct State {
    chats: HashMap<String, Vec<ChatMessage>>,
    user_chats: HashMap<String, Vec<Chat>>,
}

impl State {
    fn new() -> State {
        State {
            chats: HashMap::new(),
            user_chats: HashMap::new(),
        }
    }
    fn handle(&mut self, event: &RecordedEvent) {
        let event_type = event.event_type.as_str();
        info!("Handing {}", event_type);
        match event_type {
            "ChatCreatedEvent" => {
                if let Ok(event) = event.as_json::<ChatCreatedEvent>() {
                    self.chats.insert(event.chat_id.clone(), vec![]);
                    let chats = self
                        .user_chats
                        .entry(event.user_id)
                        .or_insert_with(Vec::new);
                    chats.push(Chat {
                        chat_id: event.chat_id,
                        subject: event.subject,
                    })
                }
            }
            "ChatMessageSentEvent" => {
                if let Ok(event) = event.as_json::<ChatMessageSentEvent>() {
                    match self.chats.get_mut(&event.chat_id) {
                        Some(messages) => messages.insert(
                            0,
                            ChatMessage {
                                message: event.message,
                                sent_by: event.user_id,
                                message_id: event.message_id,
                            },
                        ),
                        None => error!("Chat message found for unknown chat {}", event.chat_id),
                    }
                }
            }
            _ => info!("Unrecognized event type {}", event_type),
        };
    }
}

fn create_es_client() -> Result<Client> {
    let uri = env::var("EVENTSTORE_URI")?;
    let settings = uri.parse()?;
    let client = Client::new(settings)?;
    Ok(client)
}

async fn bootstrap_es(proj: Arc<Mutex<State>>) -> Result<()> {
    info!("Bootstrapping eventstore");
    let stream = "chat-stream";
    let client = create_es_client()?;

    info!("Subscribing to subscription");
    let options = SubscribeToStreamOptions::default().start_from(StreamPosition::Start);
    let mut sub = client.subscribe_to_stream(stream, &options).await;
    info!("Subscribed to {}", stream);
    loop {
        let event = sub
            .next()
            .await
            .map(|it| it.event)?
            .ok_or(Error::msg("Unable to fetch event"))?;
        // Wrap in a block to make sure the mutex guard is dropped properly.
        {
            let mut guard = proj.lock().unwrap();
            guard.handle(&event);
        }
    }
}

async fn run_es(proj: Arc<Mutex<State>>) {
    let res = bootstrap_es(proj).await;
    if let Err(err) = res {
        error!("{}", err);
        std::process::exit(1);
    }
}

#[get("/chats/{chat_id}")]
async fn get_chat(chat_id: Path<String>, proj: Data<Arc<Mutex<State>>>) -> impl Responder {
    let state = proj.lock().expect("Unable to unlock state mutex");
    let chat = match state.chats.get(chat_id.as_str()) {
        Some(it) => it,
        None => return format!("Chat with id {} not found", chat_id).to_not_found_res(),
    };
    let dto = GetChatDto {
        messages: chat.to_vec(),
    };
    dto.to_ok_res()
}

#[derive(Deserialize)]
struct UserQuery {
    user_id: String,
}

#[derive(Serialize)]
struct GetChatsDto {
    chats: Vec<Chat>,
}

#[get("/chats")]
async fn get_chats(user: Query<UserQuery>, proj: Data<Arc<Mutex<State>>>) -> impl Responder {
    let state = match proj.lock() {
        Ok(it) => it,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let vecs = match state.user_chats.get(&user.user_id) {
        Some(it) => it,
        None => return "Could not find any chats for the user".to_not_found_res(),
    };
    let dto = GetChatsDto {
        chats: (*vecs).clone(),
    };
    dto.to_ok_res()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    // Need to use an Arc with mutex here because the state will be mutated at the same time it might be accessed.
    let projection = Arc::new(Mutex::new(State::new()));

    spawn(run_es(projection.clone()));

    let port = 8080;
    info!("Started server on http://localhost:{}", port);
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET"])
            .allowed_headers(vec![
                http::header::AUTHORIZATION,
                http::header::ACCEPT,
                http::header::CONTENT_TYPE,
            ])
            .max_age(3600);

        App::new()
            .wrap(Logger::default())
            .wrap(cors)
            .app_data(Data::new(projection.clone()))
            .service(get_chat)
            .service(get_chats)
    })
    .workers(2)
    .bind(("0.0.0.0", port))?
    .run()
    .await
}

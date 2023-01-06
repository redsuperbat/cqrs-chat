use actix_cors::Cors;
use actix_web::{
    get, http,
    middleware::Logger,
    rt::spawn,
    web::{Data, Path, Query},
    App, HttpResponse, HttpServer, Responder,
};

use dtos::{ChatMessage, GetChatDto};
use events::{ChatCreatedEvent, ChatMessageSentEvent};
use eventstore::{Client, PersistentSubscriptionOptions, RecordedEvent, StreamPosition};
use log::{error, info};
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use std::{
    collections::HashMap,
    env,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

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

async fn setup_proj_eventstore(proj: Arc<Mutex<State>>) {
    info!("Bootstrapping eventstore");
    let uri = env::var("EVENTSTORE_URI").unwrap();
    let settings = uri.parse().unwrap();
    let client = Client::new(settings).unwrap();

    let consumer_grp = Uuid::new_v4().to_string();

    info!("Creating persistent subscription");
    let options = PersistentSubscriptionOptions::default().start_from(StreamPosition::Start);
    client
        .create_persistent_subscription("chat-stream", &consumer_grp, &options)
        .await
        .unwrap();

    info!("Subscribing to persistent subscription");
    let mut sub = client
        .subscribe_to_persistent_subscription("chat-stream", &consumer_grp, &Default::default())
        .await
        .unwrap();
    loop {
        let e = sub.next().await.unwrap();
        let event = e.event.as_ref().unwrap();
        // Wrap in a block to make sure the mutex guard is dropped properly.
        {
            let mut guard = proj.lock().unwrap();
            guard.handle(event);
        }
        sub.ack(e).await.unwrap();
    }
}

#[get("/chats/{chat_id}")]
async fn get_chat(chat_id: Path<String>, proj: Data<Arc<Mutex<State>>>) -> impl Responder {
    let state = proj.lock().unwrap();
    let chat = match state.chats.get(chat_id.as_str()) {
        Some(it) => it,
        None => return HttpResponse::NotFound().finish(),
    };
    let dto = GetChatDto {
        messages: chat.to_vec(),
    };
    let body = match to_string(&dto) {
        Ok(it) => it,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };
    HttpResponse::Ok()
        .content_type("application/json")
        .body(body)
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
        None => return HttpResponse::NotFound().finish(),
    };
    let dto = GetChatsDto {
        chats: (*vecs).clone(),
    };
    let body = match to_string(&dto) {
        Ok(it) => it,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };
    HttpResponse::Ok()
        .content_type("application/json")
        .body(body)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    // Need to use an Arc with mutex here because the state will be mutated at the same time it might be accessed.
    let projection = Arc::new(Mutex::new(State::new()));

    spawn(setup_proj_eventstore(projection.clone()));

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

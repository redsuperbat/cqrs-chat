use std::env;

use actix_cors::Cors;
use actix_web::{http, middleware::Logger, post, web, App, HttpResponse, HttpServer, Responder};
use dtos::JsonResponse;
use events::{ChatCreatedEvent, ChatMessageSentEvent};
use eventstore::{Client, EventData};
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use sha2::{Digest, Sha256};
use validator::Validate;

fn http_ok<T: Serialize>(message: &str, data: Option<T>) -> HttpResponse {
    let body = JsonResponse {
        message: message.to_string(),
        data,
    };
    HttpResponse::Ok()
        .content_type("application/json")
        .body(to_string(&body).unwrap())
}

fn hash_string(string: &str) -> String {
    let mut sha256 = Sha256::new();
    sha256.update(string);
    let hash = sha256.finalize();
    format!("{hash:x}")
}

#[derive(Deserialize, Validate)]
struct CreateChatDto {
    #[validate(length(min = 1, max = 36))]
    username: String,
    #[validate(length(min = 1, max = 36))]
    subject: String,
}

#[post("/create-chat")]
async fn create_chat(client: web::Data<Client>, json: web::Json<CreateChatDto>) -> impl Responder {
    match json.validate() {
        Ok(_) => (),
        Err(e) => return HttpResponse::BadRequest().body(to_string(&e).unwrap()),
    };
    let id = uuid::Uuid::new_v4().to_string();
    let user_id = uuid::Uuid::new_v4().to_string() + &json.username;
    let event_data = ChatCreatedEvent {
        chat_id: id,
        user_id: hash_string(&user_id),
        subject: json.subject.clone(),
    };
    info!("Producing event {:?}", &event_data);
    let event = EventData::json("ChatCreatedEvent", &event_data).unwrap();
    client
        .append_to_stream("chat-stream", &Default::default(), event)
        .await
        .unwrap();
    http_ok("Chat created successfully", Some(event_data))
}

#[derive(Deserialize, Serialize, Validate)]
struct SendChatMessageDto {
    #[validate(length(min = 36, max = 36))]
    chat_id: String,
    #[validate(length(min = 1, max = 120))]
    user_id: String,
    #[validate(length(min = 1, max = 255))]
    message: String,
}

#[post("/send-chat-message")]
async fn send_chat_message(
    client: web::Data<Client>,
    json: web::Json<SendChatMessageDto>,
) -> impl Responder {
    match json.validate() {
        Ok(()) => (),
        Err(e) => return HttpResponse::BadRequest().body(to_string(&e).unwrap()),
    };

    let message_id = uuid::Uuid::new_v4().to_string();

    let event_data = ChatMessageSentEvent {
        message_id,
        user_id: json.user_id.clone(),
        chat_id: json.chat_id.clone(),
        message: json.message.clone(),
    };
    let event = EventData::json("ChatMessageSentEvent", &event_data).unwrap();
    client
        .append_to_stream("chat-stream", &Default::default(), event)
        .await
        .unwrap();
    http_ok("Message sent successfully", Some(event_data))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let uri = env::var("EVENTSTORE_URI").unwrap();
    let settings = uri.parse().unwrap();

    let client = Client::new(settings).unwrap();
    let port = 8081;

    info!("Started server on http://localhost:{}", port);
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![
                http::header::AUTHORIZATION,
                http::header::ACCEPT,
                http::header::CONTENT_TYPE,
            ])
            .max_age(3600);
        App::new()
            .app_data(web::Data::new(client.clone()))
            .wrap(Logger::default())
            .wrap(cors)
            .service(create_chat)
            .service(send_chat_message)
    })
    .workers(2)
    .bind(("0.0.0.0", port))?
    .run()
    .await
}

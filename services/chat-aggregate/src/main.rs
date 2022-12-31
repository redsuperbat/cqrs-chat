use std::env;

use actix_cors::Cors;
use actix_web::{http, middleware::Logger, post, web, App, HttpResponse, HttpServer, Responder};
use events::{ChatCreatedEvent, ChatMessageSentEvent};
use eventstore::{Client, EventData};
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use sha2::{Digest, Sha256};

#[derive(Serialize)]
struct JsonResponse<T> {
    message: String,
    code: u8,
    data: Option<T>,
}

fn http_ok<T: Serialize>(message: &str, data: Option<T>) -> impl Responder {
    let body = JsonResponse {
        code: 200,
        message: message.to_string(),
        data,
    };
    HttpResponse::Ok()
        .content_type("application/json")
        .body(to_string(&body).unwrap())
}

fn hash_username(username: &str) -> String {
    let mut sha256 = Sha256::new();
    sha256.update(username);
    let hash = sha256.finalize();
    format!("{:x}", hash)
}

#[derive(Deserialize)]
struct CreateChatDto {
    username: String,
}

#[post("/create-chat")]
async fn create_chat(client: web::Data<Client>, body: web::Json<CreateChatDto>) -> impl Responder {
    let id = uuid::Uuid::new_v4().to_string();
    let event_data = ChatCreatedEvent {
        chat_id: id.clone(),
        user_id: hash_username(&body.username),
    };
    let event = EventData::json("ChatCreatedEvent", &event_data).unwrap();
    client
        .append_to_stream("chat-stream", &Default::default(), event)
        .await
        .unwrap();
    http_ok("Chat created successfully", Some(event_data))
}

#[derive(Deserialize)]
struct SendChatMessageDto {
    chat_id: String,
    message: String,
    username: String,
}

#[post("/send-chat-message")]
async fn send_chat_message(
    client: web::Data<Client>,
    body: web::Json<SendChatMessageDto>,
) -> impl Responder {
    let message_id = uuid::Uuid::new_v4().to_string();

    let event_data = ChatMessageSentEvent {
        message_id,
        // :x is a hexadecimal format!
        user_id: hash_username(&body.username),
        chat_id: body.chat_id.clone(),
        message: body.message.clone(),
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
    env::set_var("RUST_LOG", "info");
    env_logger::init();
    let uri = env::var("EVENTSTORE_URI").unwrap();
    let settings = uri.parse().unwrap();

    let client = Client::new(settings).unwrap();
    let port = 8081;

    info!("Started server on port {}", port);
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
    .bind(("localhost", port))?
    .run()
    .await
}

use std::{env, fmt::Display};

use actix_cors::Cors;
use actix_web::{
    http, middleware::Logger, post, web, App, HttpResponse, HttpResponseBuilder, HttpServer,
    Responder,
};
use dtos::JsonResponse;
use events::{ChatCreatedEvent, ChatMessageSentEvent};
use eventstore::{Client, EventData};
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::{json, to_string};
use sha2::{Digest, Sha256};
use validator::{Validate, ValidationErrors};

trait ContentTypeJsonExt {
    fn content_type_json(&mut self) -> &mut Self;
}

impl ContentTypeJsonExt for HttpResponseBuilder {
    fn content_type_json(&mut self) -> &mut Self {
        self.content_type("application/json")
    }
}

trait JsonRespExt {
    fn to_json_response(&self, message: &str) -> HttpResponse;
}

impl<DataStruct: Serialize> JsonRespExt for DataStruct {
    fn to_json_response(&self, message: &str) -> HttpResponse {
        JsonResponse {
            data: self.clone(),
            message: message.to_string(),
        };
        let json_string = match to_string(&self) {
            Ok(it) => it,
            Err(_) => return HttpResponse::InternalServerError().finish(),
        };

        HttpResponse::Ok().content_type_json().body(json_string)
    }
}

trait ValidationErrorResponseExt {
    fn to_response(&self) -> HttpResponse;
}

impl ValidationErrorResponseExt for ValidationErrors {
    fn to_response(&self) -> HttpResponse {
        let msg = self
            .field_errors()
            .into_iter()
            .map(|(str, _)| format!("Field {} is invalid", str))
            .collect::<Vec<_>>()
            .join(" and ");

        let json = json!({
            "message": msg,
            "code": 400
        })
        .to_string();
        HttpResponse::BadRequest().content_type_json().body(json)
    }
}

fn hash_string(string: impl Display) -> String {
    let mut sha256 = Sha256::new();
    sha256.update(string.to_string());
    let hash = sha256.finalize();
    format!("{hash:x}")
}

#[derive(Deserialize, Validate, Debug)]
struct CreateChatDto {
    #[validate(length(min = 1, max = 36))]
    username: String,
    #[validate(length(min = 1, max = 36))]
    subject: String,
    user_id: Option<String>,
}

#[post("/create-chat")]
async fn create_chat(client: web::Data<Client>, json: web::Json<CreateChatDto>) -> impl Responder {
    match json.validate() {
        Ok(_) => (),
        Err(e) => return e.to_response(),
    };
    let id = uuid::Uuid::new_v4().to_string();
    let user_id = json.user_id.clone().unwrap_or(hash_string(
        uuid::Uuid::new_v4().to_string() + &json.username,
    ));
    info!("{:?}", json);
    let event_data = ChatCreatedEvent {
        chat_id: id,
        user_id,
        subject: json.subject.clone(),
    };
    info!("Producing event {:?}", &event_data);
    let event = EventData::json("ChatCreatedEvent", &event_data).unwrap();
    client
        .append_to_stream("chat-stream", &Default::default(), event)
        .await
        .unwrap();
    event_data.to_json_response("Chat created successfully")
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
        Err(e) => return e.to_response(),
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
    event_data.to_json_response("Message sent successfully!")
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

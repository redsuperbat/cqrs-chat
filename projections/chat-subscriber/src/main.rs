use std::{
    env,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use actix::{Actor, AsyncContext, Handler, Message, StreamHandler};
use actix_cors::Cors;
use actix_web::{
    http::{self},
    middleware::Logger,
    web::{self, Data},
    App, Error, HttpRequest, HttpResponse, HttpServer,
};
use actix_web_actors::ws::{self};
use events::ChatMessageSentEvent;
use eventstore::{
    Client, DeletePersistentSubscriptionOptions, PersistentSubscription,
    PersistentSubscriptionOptions, StreamPosition,
};
use eyre::Result;
use log::info;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

async fn create_sub(client: &Client, consumer_grp: &str) -> Result<PersistentSubscription> {
    info!("Creating persistent subscription with id {}", consumer_grp);
    let options = PersistentSubscriptionOptions::default().start_from(StreamPosition::End);
    client
        .create_persistent_subscription("chat-stream", consumer_grp, &options)
        .await?;
    client
        .subscribe_to_persistent_subscription("chat-stream", consumer_grp, &Default::default())
        .await
        .map_err(eyre::Error::from)
}

async fn delete_sub(client: &Client, consumer_grp: &str) -> Result<()> {
    info!("Deleting subscription with id {}", consumer_grp);
    let options = DeletePersistentSubscriptionOptions::default();
    client
        .delete_persistent_subscription("chat-stream", consumer_grp, &options)
        .await
        .map_err(eyre::Error::from)
}

/// Define HTTP actor
struct ChatServer {
    client_count: Arc<AtomicUsize>,
    chat_id: String,
    consumer_grp_id: String,
}

impl Actor for ChatServer {
    type Context = ws::WebsocketContext<Self>;
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Event {
    body: String,
}
impl Handler<Event> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Event, ctx: &mut Self::Context) {
        ctx.text(msg.body);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ChatServer {
    fn started(&mut self, ctx: &mut Self::Context) {
        self.client_count.fetch_add(1, Ordering::SeqCst);
        info!(
            "Client connected. #clients {}",
            self.client_count.load(Ordering::SeqCst)
        );
        let recipient = ctx.address();
        let chat_id = self.chat_id.clone();
        let group_id = self.consumer_grp_id.clone();
        let client = create_eventstore_client().expect("Unable to create eventstore client");
        let fut = async move {
            let mut sub = match create_sub(&client, &group_id).await {
                Ok(sub) => sub,
                Err(_) => return,
            };

            loop {
                let event = sub.next().await.map(|it| it.event).ok().flatten();
                let event = match event {
                    Some(it) => it,
                    None => {
                        log::error!("None was received in recorded event, breaking loop");
                        break;
                    }
                };
                if let Ok(event) = event.as_json::<ChatMessageSentEvent>() {
                    if event.chat_id == chat_id {
                        let json_string = json!({
                            "message": event.message,
                            "sent_by": event.user_id,
                            "message_id": event.message_id,
                        })
                        .to_string();
                        let dto = Event { body: json_string };
                        recipient.do_send(dto);
                    }
                }
                sub.ack_ids(vec![event.id])
                    .await
                    .expect("unable to ack event");
            }
        };
        let fut = actix::fut::wrap_future::<_, Self>(fut);
        // Since the execution of the future is automatically closed when the
        // StreamHandler is finished there is no need to manually drop the connection to
        // Eventstore
        ctx.spawn(fut);
    }

    fn finished(&mut self, ctx: &mut Self::Context) {
        self.client_count.fetch_sub(1, Ordering::SeqCst);
        info!(
            "Client disconnected. #clients {}",
            self.client_count.load(Ordering::SeqCst)
        );
        let group_id = self.consumer_grp_id.clone();
        let client = create_eventstore_client().expect("Unable to create eventstore client");
        let fut = async move {
            delete_sub(&client, &group_id)
                .await
                .expect("Unable to delete subscription")
        };
        let fut = actix::fut::wrap_future(fut);
        ctx.spawn(fut);
    }

    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        if let Ok(ws::Message::Ping(msg)) = msg {
            info!("Ping {:?}", msg);
            ctx.pong(&msg)
        }
    }
}

#[derive(Deserialize)]
struct WsQuery {
    chat_id: String,
}

async fn index(
    req: HttpRequest,
    stream: web::Payload,
    client_count: Data<Arc<AtomicUsize>>,
    query: web::Query<WsQuery>,
) -> Result<HttpResponse, Error> {
    let consumer_grp_id = Uuid::new_v4().to_string();
    ws::start(
        ChatServer {
            client_count: client_count.as_ref().clone(),
            chat_id: query.chat_id.clone(),
            consumer_grp_id,
        },
        &req,
        stream,
    )
}

fn create_eventstore_client() -> Result<Client> {
    let uri = env::var("EVENTSTORE_URI")?;
    let settings = uri.parse()?;
    Client::new(settings).map_err(eyre::Error::from)
}

fn create_cors() -> Cors {
    Cors::default()
        .allow_any_origin()
        .allowed_methods(vec!["GET"])
        .allowed_headers(vec![
            http::header::AUTHORIZATION,
            http::header::ACCEPT,
            http::header::CONTENT_TYPE,
        ])
        .max_age(3600)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let client_count = Arc::new(AtomicUsize::new(0));
    let port = 8082;
    info!("Started server at http://localhost:{}", port);
    HttpServer::new(move || {
        let cors = create_cors();
        App::new()
            .wrap(Logger::default())
            .wrap(cors)
            .app_data(Data::new(client_count.clone()))
            .route("/ws/", web::get().to(index))
    })
    .workers(4)
    .bind(("0.0.0.0", port))?
    .run()
    .await
}

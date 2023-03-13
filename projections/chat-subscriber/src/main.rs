use std::{
    env, process,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use actix::{spawn, Actor, ActorContext, AsyncContext, Handler, Message, StreamHandler};
use actix_cors::Cors;
use actix_web::{
    http,
    web::{get, Data, Payload, Query},
    App, Error as ActixError, HttpRequest, HttpResponse, HttpServer,
};
use actix_web_actors::ws;
use events::ChatMessageSentEvent;
use eventstore::{Client, PersistentSubscriptionOptions, StreamPosition};
use eyre::{Error, Result};
use log::{error, info, warn};
use serde::Deserialize;
use serde_json::json;
use tokio::sync::broadcast::Sender;

struct ChatServer {
    client_count: Arc<AtomicUsize>,
    chat_id: String,
    tx: Sender<ChatMessageSentEvent>,
}

impl Actor for ChatServer {
    type Context = ws::WebsocketContext<ChatServer>;
}

#[derive(Message)]
#[rtype(result = "()")]
struct Event(String);

impl Handler<Event> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Event, ctx: &mut Self::Context) {
        ctx.text(msg.0);
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
        let mut rx = self.tx.subscribe();
        let fut = async move {
            loop {
                let event = match rx.recv().await {
                    Ok(e) => e,
                    Err(err) => {
                        error!("Unable to receive event from channel. {}", err.to_string());
                        break;
                    }
                };

                if event.chat_id != chat_id {
                    continue;
                };

                let json_string = json!({
                    "message": event.message,
                    "sent_by": event.user_id,
                    "message_id": event.message_id,
                })
                .to_string();
                recipient.do_send(Event(json_string))
            }
        };
        let fut = actix::fut::wrap_future::<_, Self>(fut);
        ctx.spawn(fut);
    }

    fn finished(&mut self, ctx: &mut Self::Context) {
        self.client_count.fetch_sub(1, Ordering::SeqCst);
        info!(
            "Client disconnected. #clients {}",
            self.client_count.load(Ordering::SeqCst)
        );
        ctx.stop();
    }

    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        if let Ok(ws::Message::Close(reason)) = msg {
            if let Some(res) = reason {
                info!(
                    "Code {:?}, reason {}",
                    res.code,
                    res.description.unwrap_or_default()
                );
            }
            self.finished(ctx);
        }
    }
}

#[derive(Deserialize)]
struct WsQuery {
    chat_id: String,
}

async fn index(
    req: HttpRequest,
    stream: Payload,
    query: Query<WsQuery>,
    // Data
    client_count: Data<Arc<AtomicUsize>>,
    tx: Data<Sender<ChatMessageSentEvent>>,
) -> Result<HttpResponse, ActixError> {
    ws::start(
        ChatServer {
            client_count: client_count.as_ref().clone(),
            chat_id: query.chat_id.clone(),
            tx: tx.as_ref().clone(),
        },
        &req,
        stream,
    )
}

async fn create_sub(client: &Client, consumer_grp: &str) -> Result<()> {
    info!("Creating persistent subscription with id {}", consumer_grp);
    let options = PersistentSubscriptionOptions::default().start_from(StreamPosition::End);
    client
        .create_persistent_subscription("chat-stream", consumer_grp, &options)
        .await
        .map_err(eyre::Error::from)
}

fn create_eventstore_client() -> Result<Client> {
    let uri = env::var("EVENTSTORE_URI")?;
    let settings = uri.parse()?;
    Client::new(settings).map_err(eyre::Error::from)
}

async fn bootstrap_es(tx: Sender<ChatMessageSentEvent>) -> Result<()> {
    let client = create_eventstore_client().expect("Unable to create es client");
    let grp_id = env::var("GROUP_ID")?;
    match create_sub(&client, &grp_id).await {
        Ok(_) => info!("Created group {}", &grp_id),
        Err(err) => warn!("Err creating group {}", err.to_string()),
    };

    let mut sub = client
        .subscribe_to_persistent_subscription("chat-stream", grp_id, &Default::default())
        .await?;

    while let Ok(event) = sub.next().await {
        let event = event.event.ok_or(Error::msg("No recorded event"))?;
        if let Ok(event) = event.as_json::<ChatMessageSentEvent>() {
            tx.send(event)?;
        }
        sub.ack_ids(vec![event.id]).await?;
    }
    Ok(())
}

async fn run_es(tx: Sender<ChatMessageSentEvent>) {
    let res = bootstrap_es(tx).await;
    if let Err(err) = res {
        error!("{}", err);
        process::exit(1);
    }
}

fn create_cors() -> Cors {
    Cors::default()
        .allowed_origin("http://localhost:3000")
        .allowed_origin("https://chat.netterberg.me")
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
    let (tx, _) = tokio::sync::broadcast::channel::<ChatMessageSentEvent>(100);
    spawn(run_es(tx.clone()));
    info!("Started server at http://localhost:{}", port);
    HttpServer::new(move || {
        let cors = create_cors();
        App::new()
            .wrap(cors)
            .app_data(Data::new(client_count.clone()))
            .app_data(Data::new(tx.clone()))
            .route("/ws/", get().to(index))
    })
    .workers(4)
    .bind(("0.0.0.0", port))?
    .run()
    .await
}

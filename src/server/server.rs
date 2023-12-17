use axum::{
    extract::connect_info::ConnectInfo,
    extract::ws::{CloseFrame, Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use axum_extra::TypedHeader;
use log::{error, info};
use std::borrow::Cow;
use std::ops::ControlFlow;
use std::{net::SocketAddr, path::PathBuf};
use std::sync::mpsc;
use tower_http::{
    services::ServeDir,
    trace::{DefaultMakeSpan, TraceLayer},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use futures::{sink::SinkExt, stream::StreamExt};

use super::listeners::http::HttpListener;
use super::jobs::Job;
use super::sessions::Session;

pub struct Server {
    jobs: Vec<Job>,
    listeners: Vec<HttpListener>,
    sessions: Vec<Session>,
}

impl Server {
    pub fn new() -> Self {
        Self {
            jobs: Vec::new(),
            listeners: Vec::new(),
            sessions: Vec::new(),
        }
    }

    pub async fn run(&mut self) {
        // Spawn threads for HTTP listeners.
        // let (mut tx, mut rx) = mpsc::channel();

        let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");

        let app = Router::new()
            .fallback_service(ServeDir::new(assets_dir).append_index_html_on_directories(true))
            .route("/hermit", get(Self::ws_handler))
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::default().include_headers(true)),
            );

        let listener = tokio::net::TcpListener::bind("127.0.0.1:9999")
            .await
            .unwrap();
        info!("listening on {}", listener.local_addr().unwrap());

        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    }

    async fn ws_handler(
        ws: WebSocketUpgrade,
        user_agent: Option<TypedHeader<headers::UserAgent>>,
        ConnectInfo(addr): ConnectInfo<SocketAddr>,
    ) -> impl IntoResponse {
        let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
            user_agent.to_string()
        } else {
            String::from("unknown browser")
        };

        info!("`{user_agent}` at {addr} connected.");

        ws.on_upgrade(move |socket| Self::handle_socket(socket, addr))
    }

    async fn handle_socket(mut socket: WebSocket, who: SocketAddr)  {
        // if socket.send(Message::Text("hello from Hermit C2 server.".to_string())).await.is_ok() {
        //     info!("Sending a message to {who}...");
        // } else {
        //     error!("Could not send a message to {who}.");
        //     return;
        // }
        
        loop {
            if let Some(msg) = socket.recv().await {
                if let Ok(msg) = msg {
                    // if Self::process_message(msg, who).is_break() {
                    //         let _ = socket.send(Message::Text("Finish from server.".to_string())).await;
                    //         return;
                    //     }

                    match msg {
                        Message::Text(text) => {
                            if text.contains("add") {
                                let mut args = match shellwords::split(text.as_str()) {
                                    Ok(args) => { args }
                                    Err(err) => { error!("Can't parse command line: {err}"); vec!["".to_string()] }
                                };
                                let url = args[1].to_string();
                                let _ = socket.send(Message::Text(format!("Listener `{url}` added."))).await;
                            } else if text.contains("delete") {
                                let mut args = match shellwords::split(text.as_str()) {
                                    Ok(args) => { args }
                                    Err(err) => { error!("Could not delete listener: {err}"); vec!["".to_string()] }
                                };
                                let listener_id = args[1].to_string();
                                let _  = socket.send(Message::Text(format!("Listener `{listener_id}` stopped.")));
                            } else if text.contains("start") {
                                let mut args = match shellwords::split(text.as_str()) {
                                    Ok(args) => { args }
                                    Err(err) => { error!("Could not start listener: {err}"); vec!["".to_string()] }
                                };
                                let listener_id = args[1].to_string();
                                let _ = socket.send(Message::Text(format!("Listener `{listener_id}` started.")));
                            } else if text.contains("stop") {
                                let mut args = match shellwords::split(text.as_str()) {
                                    Ok(args) => { args }
                                    Err(err) => { error!("Could not stop listener: {err}"); vec!["".to_string()] }
                                };
                                let listener_id = args[1].to_string();
                                let _ = socket.send(Message::Text(format!("Listener `{listener_id}` stopped."))).await;
                            } else if text.contains("listeners") {
                                let _ = socket.send(Message::Text("List listeners".to_string())).await;
                            } else if text.contains("generate") {
                                let _ = socket.send(Message::Text("Generate a payload".to_string())).await;
                            } else if text.contains("payloads") {
                                let _ = socket.send(Message::Text("List payloads".to_string())).await;
                            } else {
                                let _ = socket.send(Message::Text(format!("Unknown command: {text}"))).await;
                            }
                        }
                        _ => {}
                    }
                } else {
                    info!("Client {who} abruptly disconnected.");
                    return;
                }
            }
        }

        let (mut sender, mut receiver) = socket.split();

    }

    // fn process_message(msg: Message, who: SocketAddr) -> ControlFlow<(), ()> {
    //     match msg {
    //         Message::Text(t) => {
    //             info!(">>> {who} sent str: {t:?}");
    //         }
    //         Message::Binary(d) => {
    //             info!(">>> {} sent {} bytes: {:?}", who, d.len(), d);
    //         }
    //         Message::Close(c) => {
    //             if let Some(cf) = c {
    //                 info!(
    //                     ">>> {} sent close with code {} and reason `{}`",
    //                     who, cf.code, cf.reason
    //                 );
    //             } else {
    //                 info!(">>> {who} somehow sent close message without CloseFrame");
    //             }
    //             return ControlFlow::Break(());
    //         }
    //         Message::Pong(v) => {
    //             info!(">>> {who} sent pong with {v:?}");
    //         }
    //         Message::Ping(v) => {
    //             info!(">>> {who} sent ping with {v:?}");
    //         }
    //     }

    //     ControlFlow::Continue(())
    // }
}

use axum::{
    Extension,
    extract::connect_info::ConnectInfo,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use axum_extra::TypedHeader;
use log::{error, info, warn};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tower_http::{
    add_extension::AddExtensionLayer,
    trace::{DefaultMakeSpan, TraceLayer},
};
use url::Url;

use super::listeners::listener::Listener;
use super::jobs::{find_job, format_jobs, Job, JobMessage};
use super::sessions::Session;

#[derive(Debug)]
pub struct Server {
    pub jobs: Arc<Mutex<Vec<Job>>>,
    listeners: Vec<Listener>,
    sessions: Vec<Session>,

    sender: broadcast::Sender<JobMessage>,
    receiver: Arc<Mutex<broadcast::Receiver<JobMessage>>>,
}

impl Server {
    pub fn new(sender: broadcast::Sender<JobMessage>, receiver: Arc<Mutex<broadcast::Receiver<JobMessage>>>) -> Self {
        Self {
            jobs: Arc::new(Mutex::new(Vec::new())),
            listeners: Vec::new(),
            sessions: Vec::new(),
            sender,
            receiver,
        }
    }
}

pub async fn run() {
    let (mut tx, mut rx) = broadcast::channel(100);

    // Spawn thread for jobs.
    // tokio::spawn(async move {
    //     loop {
    //         if let Ok(msg) = rx.recv().await {
    //             match(msg) {
    //                 ServerMessage::StartJob(id) => {},
    //                 ServerMessage::StopJob(id) => {},
    //                 ServerMessage::DeleteJob(id) => {},
    //                 _ => {},
    //             }
    //         }
    //     }
    // });

    let server = Arc::new(Mutex::new(Server::new(tx, Arc::new(Mutex::new(rx)))));

    let app = Router::new()
        .route("/hermit", get(ws_handler))
        .layer(
            AddExtensionLayer::new(server),
        )
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
    Extension(server): Extension<Arc<Mutex<Server>>>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("unknown browser")
    };

    info!("`{user_agent}` at {addr} connected.");

    ws.on_upgrade(move |socket| handle_socket(socket, addr, server))
}

async fn handle_socket(mut socket: WebSocket, who: SocketAddr, server: Arc<Mutex<Server>>)  {
    
    // if socket.send(Message::Text("hello from Hermit C2 server.".to_string())).await.is_ok() {
        //     info!("Sending a message to {who}...");
        // } else {
            //     error!("Could not send a message to {who}.");
            //     return;
            // }

    // let mut server_lock = server.lock().await;

    
    loop {
        if let Some(msg) = socket.recv().await {
            if let Ok(msg) = msg {
                // if Self::process_message(msg, who).is_break() {
                    //         let _ = socket.send(Message::Text("Finish from server.".to_string())).await;
                    //         return;
                    //     }
                    
                match msg {
                    Message::Text(text) => {
                        let server_clone = Arc::clone(&server);
                        let server_lock = server_clone.lock().await;
                        let receiver = &server_lock.receiver;

                        if text.starts_with("add") {
                            let args = match shellwords::split(text.as_str()) {
                                Ok(args) => { args }
                                Err(err) => { error!("Can't parse command line: {err}"); vec!["".to_string()] }
                            };
                            let name = &args[1];
                            let url = Url::parse(&args[2]).unwrap();

                            let mut jobs = server_lock.jobs.lock().await;
                            let next_id = jobs.len() as u32;

                            let new_job = Job::new(
                                next_id,
                                name.to_string(),
                                url.scheme().to_owned(),
                                url.host().unwrap().to_string(),
                                url.port().unwrap(),
                                Arc::clone(&receiver),
                            );

                            jobs.push(new_job);
                            let _ = socket.send(Message::Text(format!("Listener `{url}` added."))).await;

                        } else if text.starts_with("delete") {
                            let args = match shellwords::split(text.as_str()) {
                                Ok(args) => { args }
                                Err(err) => { error!("Could not delete listener: {err}"); vec!["".to_string()] }
                            };
                            let target = args[1].to_string();

                            let mut jobs = server_lock.jobs.lock().await;
                            let mut jobs_owned = jobs.to_owned();

                            let job = match find_job(&mut jobs_owned, target.to_string()).await {
                                Some(j) => j,
                                None => {
                                    let _ = socket.send(Message::Text(format!("Listener `{target}` not found."))).await;
                                    continue;
                                }
                            };

                            info!("job found.");

                            jobs.remove(job.id as usize);
                            let _ = socket.send(Message::Text(format!("Listener `{target}` deleted."))).await;

                        } else if text.starts_with("start") {
                            let args = match shellwords::split(text.as_str()) {
                                Ok(args) => { args }
                                Err(err) => { error!("Could not start listener: {err}"); vec!["".to_string()] }
                            };
                            let target = args[1].to_string();

                            let mut jobs = server_lock.jobs.lock().await;
                            let job = match find_job(&mut jobs, target.to_string()).await {
                                Some(j) => j,
                                None => {
                                    let _ = socket.send(Message::Text(format!("Listener `{target}` not found."))).await;
                                    continue;
                                }
                            };

                            job.running = true;
                            let _ = server_lock.sender.send(JobMessage::Start(job.id));
                            let _  = socket.send(Message::Text(format!("Listener `{target}` started."))).await;

                        } else if text.starts_with("stop") {
                            let args = match shellwords::split(text.as_str()) {
                                Ok(args) => { args }
                                Err(err) => { error!("Could not stop listener: {err}"); vec!["".to_string()] }
                            };
                            let target = args[1].to_string();

                            let mut jobs = server_lock.jobs.lock().await;
                            let job = match find_job(&mut jobs, target.to_string()).await {
                                Some(j) => j,
                                None => {
                                    let _ = socket.send(Message::Text(format!("Listener `{target}` not found."))).await;
                                    continue;
                                }
                            };

                            job.running = false;
                            let _ = server_lock.sender.send(JobMessage::Stop(job.id));
                            let _ = socket.send(Message::Text(format!("Listener `{target}` stopped."))).await;

                        } else if text.starts_with("listeners") {
                            let mut jobs = server_lock.jobs.lock().await;
                            let output = format_jobs(&mut jobs);
                            let _ = socket.send(Message::Text(output)).await;

                        } else if text.starts_with("generate") {
                            let _ = socket.send(Message::Text("Generate an implant.".to_string())).await;

                        } else if text.starts_with("implants") {
                            let _ = socket.send(Message::Text("List implants".to_string())).await;

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

    // let (mut sender, mut receiver) = socket.split();

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
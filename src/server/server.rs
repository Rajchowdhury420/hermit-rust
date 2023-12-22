use axum::{
    Extension,
    extract::connect_info::ConnectInfo,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use axum_extra::TypedHeader;
use log::{error, info};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tower_http::{
    add_extension::AddExtensionLayer,
    trace::{DefaultMakeSpan, TraceLayer},
};
use url::Url;

use super::jobs::{check_dupl_job, find_job, format_jobs, Job, JobMessage};
use super::agents::{Agent, format_agents};
use crate::implants::generate::generate;
use crate::config::Config;
// use super::sessions::Session;

#[derive(Debug)]
pub struct Server {
    pub config: Config,
    pub jobs: Arc<Mutex<Vec<Job>>>,
    tx_job: Arc<Mutex<broadcast::Sender<JobMessage>>>,
    pub agents: Arc<Mutex<Vec<Agent>>>,
    // sessions: Vec<Session>,

}

impl Server {
    pub fn new(config: Config, tx_job: broadcast::Sender<JobMessage>) -> Self {
        Self {
            config,
            jobs: Arc::new(Mutex::new(Vec::new())),
            tx_job: Arc::new(Mutex::new(tx_job)),
            agents: Arc::new(Mutex::new(Vec::new())),
            // sessions: Vec::new(),
        }
    }

    pub async fn add_agent(&mut self, agent: &mut Agent) {
        let mut agents = self.agents.lock().await;

        // Update the agent ID
        agent.id = agents.len() as u32;

        agents.push(agent.to_owned());
    }
}

pub async fn run(config: Config) {
    let (tx_job, _rx_job) = broadcast::channel(100);
    let server = Arc::new(Mutex::new(Server::new(config, tx_job)));

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

async fn handle_socket(socket: WebSocket, who: SocketAddr, server: Arc<Mutex<Server>>)  {
    let socket_arc = Arc::new(Mutex::new(socket));
    
    loop {
        let socket_clone = Arc::clone(&socket_arc);
        let mut socket_lock = socket_clone.lock().await;

        if let Some(msg) = socket_lock.recv().await {
            if let Ok(msg) = msg {
                match msg {
                    Message::Text(text) => {
                        let server_clone = Arc::clone(&server);
                        let server_lock = server_clone.lock().await;

                        if text.starts_with("add") {
                            let args = match shellwords::split(text.as_str()) {
                                Ok(args) => { args }
                                Err(err) => { error!("Can't parse command line: {err}"); vec!["".to_string()] }
                            };
                            let name = &args[1];
                            let url = Url::parse(&args[2]).unwrap();
                            
                            let mut jobs = server_lock.jobs.lock().await;

                            // Check if the url already exists.
                            match check_dupl_job(&mut jobs, url.to_owned()) {
                                Ok(_) => {},
                                Err(e) => {
                                    error!("{e}");
                                    let _ = socket_lock.send(Message::Text(format!("Error: This URL already exists."))).await;
                                    let _ = socket_lock.send(Message::Text("done".to_string())).await;
                                    continue;
                                },
                            }

                            let next_id = jobs.len() as u32;

                            let rx_job = server_lock.tx_job.lock().await;

                            let new_job = Job::new(
                                next_id,
                                name.to_string(),
                                url.scheme().to_owned(),
                                url.host().unwrap().to_string(),
                                url.port().unwrap(),
                                Arc::new(Mutex::new(rx_job.subscribe())),
                                Arc::clone(&server_clone),
                            );

                            jobs.push(new_job);
                            let _ = socket_lock.send(Message::Text(format!("Listener `{url}` added."))).await;
                            let _ = socket_lock.send(Message::Text("done".to_string())).await;

                        } else if text.starts_with("delete") {
                            let args = match shellwords::split(text.as_str()) {
                                Ok(args) => { args }
                                Err(err) => {
                                    error!("Could not delete listener: {err}");
                                    vec!["".to_string()]
                                }
                            };
                            let target = args[1].to_string();

                            let mut jobs = server_lock.jobs.lock().await;
                            let mut jobs_owned = jobs.to_owned();

                            let job = match find_job(&mut jobs_owned, target.to_string()).await {
                                Some(j) => j,
                                None => {
                                    let _ = socket_lock.send(Message::Text(format!("Listener `{target}` not found."))).await;
                                    let _ = socket_lock.send(Message::Text("done".to_string())).await;
                                    continue;
                                }
                            };

                            if !job.running {
                                job.handle.lock().await.abort();
                                jobs.remove(job.id as usize);
                                let _ = socket_lock.send(Message::Text(format!("Listener `{target}` deleted."))).await;
                            } else {
                                let _ = socket_lock.send(
                                    Message::Text(format!("Listener `{target}` cannot be deleted because it's running. Please stop it before deleting."))
                                ).await;
                            }

                            let _ = socket_lock.send(Message::Text("done".to_string())).await;

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
                                    let _ = socket_lock.send(Message::Text(format!("Listener `{target}` not found."))).await;
                                    let _ = socket_lock.send(Message::Text("done".to_string())).await;
                                    continue;
                                }
                            };

                            if !job.running {
                                let _ = server_lock.tx_job.lock().await.send(JobMessage::Start(job.id));
                                job.running = true;
                                let _ = socket_lock.send(Message::Text(format!("Listener `{target}` started."))).await;
                            } else {
                                let _ = socket_lock.send(Message::Text(format!("Listener `{target}` is alread running"))).await;
                            }
                            let _ = socket_lock.send(Message::Text("done".to_string())).await;

                        } else if text.starts_with("stop") {
                            let args = match shellwords::split(text.as_str()) {
                                Ok(args) => { args }
                                Err(err) => {
                                    error!("Could not stop listener: {err}");
                                    vec!["".to_string()]
                                }
                            };
                            let target = args[1].to_string();

                            let mut jobs = server_lock.jobs.lock().await;
                            let job = match find_job(&mut jobs, target.to_string()).await {
                                Some(j) => j,
                                None => {
                                    let _ = socket_lock.send(Message::Text(format!("Listener `{target}` not found."))).await;
                                    let _ = socket_lock.send(Message::Text("done".to_string())).await;
                                    continue;
                                }
                            };

                            if job.running {
                                let _ = server_lock.tx_job.lock().await.send(JobMessage::Stop(job.id));
                                job.running = false;
                                let _ = socket_lock.send(Message::Text(format!("Listener `{target}` stopped."))).await;
                            } else {
                                let _ = socket_lock.send(Message::Text(format!("Listener `{target}` is already stopped."))).await;
                            }
                            let _ = socket_lock.send(Message::Text("done".to_string())).await;

                        } else if text.starts_with("listeners") {
                            let mut jobs = server_lock.jobs.lock().await;
                            let output = format_jobs(&mut jobs);
                            let _ = socket_lock.send(Message::Text(output)).await;
                            let _ = socket_lock.send(Message::Text("done".to_string())).await;

                        } else if text.starts_with("agents") {
                            let mut agents = server_lock.agents.lock().await;
                            let output = format_agents(&mut agents);
                            let _ = socket_lock.send(Message::Text(output)).await;
                            let _ = socket_lock.send(Message::Text("done".to_string())).await;

                        } else if text.starts_with("generate") {
                            let args = match shellwords::split(text.as_str()) {
                                Ok(args) => { args }
                                Err(err) => {
                                    error!("Could not parse arguments: {err}");
                                    vec!["".to_string()]
                                }
                            };
                            let i_name = args[1].to_string();
                            let i_listener_url = args[2].to_string();
                            let i_os = args[3].to_string();
                            let i_arch = args[4].to_string();
                            let i_format = args[5].to_string();

                            // Generate an implant
                            match generate(
                                &server_lock.config,
                                i_name.to_string(),
                                i_listener_url.to_string(),
                                i_os.to_string(),
                                i_arch.to_string(),
                                i_format.to_string(),
                            ) {
                                Ok((output, buffer)) => {
                                    let _ = socket_lock.send(
                                        Message::Text(format!(
                                            "generated {}/client/implants/{}",
                                            server_lock.config.app_dir.display(),
                                            output.split("/").last().unwrap()
                                        ))).await;
                                    let _ = socket_lock.send(Message::Binary(buffer)).await;
                                    let _ = socket_lock.send(Message::Text("done".to_string())).await;

                                },
                                Err(e) => {
                                    let _ = socket_lock.send(
                                        Message::Text(format!("Could not generate an imaplant: {e}"))
                                    ).await;
                                    let _ = socket_lock.send(
                                        Message::Text("done".to_string())).await;
                                }
                            }

                        } else if text.starts_with("implants") {
                            let _ = socket_lock.send(Message::Text("List implants.".to_string())).await;
                            let _ = socket_lock.send(Message::Text("done".to_string())).await;

                        } else {
                            let _ = socket_lock.send(Message::Text(format!("Unknown command: {text}"))).await;
                            let _ = socket_lock.send(Message::Text("done".to_string())).await;
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
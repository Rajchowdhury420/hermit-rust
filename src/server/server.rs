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
use std::io::{Error, ErrorKind};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex, MutexGuard};
use tower_http::{
    add_extension::AddExtensionLayer,
    trace::{DefaultMakeSpan, TraceLayer},
};

use super::{
    agents::Agent,
    jobs::{Job, JobMessage},
    handlers::{
        agent::handle_agent,
        implant::handle_implant,
        listener::handle_listener,
        task::handle_task,
    },
};
use crate::{
    implants::implant::Implant,
    config::Config,
    utils::fs::{mkdir, mkfile},
};

#[derive(Debug)]
pub struct Server {
    pub config: Config,
    pub jobs: Arc<Mutex<Vec<Job>>>,
    pub tx_job: Arc<Mutex<broadcast::Sender<JobMessage>>>,
    pub agents: Arc<Mutex<Vec<Agent>>>,
    pub implants: Arc<Mutex<Vec<Implant>>>,
}

impl Server {
    pub fn new(
        config: Config,
        tx_job: broadcast::Sender<JobMessage>,
    ) -> Self {
        Self {
            config,
            jobs: Arc::new(Mutex::new(Vec::new())),
            tx_job: Arc::new(Mutex::new(tx_job)),
            agents: Arc::new(Mutex::new(Vec::new())),
            implants: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn add_agent(&mut self, new_agent: &mut Agent) -> Result<(), Error> {
        let mut agents = self.agents.lock().await;

        // Check if the same agent already exists
        for agent in agents.iter() {
            if  agent.hostname == new_agent.hostname &&
                agent.os == new_agent.os &&
                agent.arch == new_agent.arch &&
                agent.listener_url == new_agent.listener_url
            {
                return Err(Error::new(ErrorKind::Other, "This agent has already registered"));
            }
        }

        // Create a directory and files for the new agent
        mkdir(format!("agents/{}/task", new_agent.name.to_owned())).unwrap();
        mkdir(format!("agents/{}/screenshots", new_agent.name.to_owned())).unwrap();
        mkfile(format!("agents/{}/task/name", new_agent.name.to_owned())).unwrap();
        mkfile(format!("agents/{}/task/result", new_agent.name.to_owned())).unwrap();

        // Update the agent ID and push
        new_agent.id = agents.len() as u32;
        agents.push(new_agent.to_owned());
        Ok(())
    }

    pub async fn add_implant(&mut self, new_implant: &mut Implant) -> Result<(), Error> {
        let mut implants = self.implants.lock().await;

        // Update the implant ID
        new_implant.id = implants.len() as u32;
        implants.push(new_implant.to_owned());
        Ok(())
    }

    pub async fn is_dupl_implant(&mut self, new_implant: &mut Implant) -> bool {
        let implants = self.implants.lock().await;

        // Check if the same implant already exists
        for implant in implants.iter() {
            if  implant.listener_url == new_implant.listener_url &&
                implant.os == new_implant.os &&
                implant.arch == new_implant.arch &&
                implant.format == new_implant.format &&
                implant.sleep == new_implant.sleep
            {
                return true;
            }
        }
        return false;
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

async fn handle_socket(
    socket: WebSocket,
    who: SocketAddr,
    server: Arc<Mutex<Server>>
)  {
    let socket_arc = Arc::new(Mutex::new(socket));
        
    loop {
        let socket_clone = Arc::clone(&socket_arc);
        let mut socket_lock = socket_clone.lock().await;

        if let Some(msg) = socket_lock.recv().await {
            if let Ok(msg) = msg {
                handle_message(msg, socket_lock, Arc::clone(&server)).await;
            } else {
                info!("Client {who} abruptly disconnected.");
                return;
            }
        }
    }
}

async fn handle_message(
    msg: Message,
    mut socket_lock: MutexGuard<'_, WebSocket>,
    server: Arc<Mutex<Server>>
) {
    match msg {
        Message::Text(text) => {
            let args = match shellwords::split(text.as_str()) {
                Ok(args) => { args }
                Err(err) => {
                    error!("Can't parse command line: {err}");
                    // vec!["".to_string()]
                    return;
                }
            };

            match args[0].as_str() {
                "listener" => {
                    handle_listener(
                        args,
                        &mut socket_lock,
                        Arc::clone(&server),
                    ).await;
                }
                "agent" => {
                    handle_agent(
                        args,
                        &mut socket_lock,
                        Arc::clone(&server),
                    ).await;
                }
                "implant" => {
                    handle_implant(
                        text.to_owned(),
                        args,
                        &mut socket_lock,
                        Arc::clone(&server),
                    ).await;
                }
                "task" => {
                    handle_task(
                        text.to_owned(),
                        args,
                        &mut socket_lock,
                        Arc::clone(&server),
                    ).await;
                }
                _ => {
                    let _ = socket_lock.send(
                        Message::Text(format!("Unknown command: {text}"))
                    ).await;
                    let _ = socket_lock.send(
                        Message::Text("[done]".to_owned())
                    ).await;
                }
            }
        }
        _ => {}
    }
}

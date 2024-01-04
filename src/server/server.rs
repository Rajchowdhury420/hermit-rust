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
    certs::https::{create_server_certs, create_root_ca},
    crypto::aesgcm,
    db,
    listeners::listener::Listener,
    jobs::{find_job, Job, JobMessage},
    handlers::{
        agent::handle_agent,
        implant::handle_implant,
        listener::handle_listener,
        task::handle_task,
    },
};
use crate::{
    config::Config,
    server::db::DB_PATH,
    utils::fs::{mkdir, mkfile, exists_file},
};

#[derive(Debug)]
pub struct Server {
    pub config: Config,
    pub db: db::DB,
    pub jobs: Arc<Mutex<Vec<Job>>>,  // Jobs contain listeners
    pub tx_job: Arc<Mutex<broadcast::Sender<JobMessage>>>,
}

impl Server {
    pub fn new(
        config: Config,
        db: db::DB,
        tx_job: broadcast::Sender<JobMessage>,
    ) -> Self {
        Self {
            config,
            db,
            jobs: Arc::new(Mutex::new(Vec::new())),
            tx_job: Arc::new(Mutex::new(tx_job)),
        }
    }

    pub async fn add_listener(
        &mut self,
        name: String,
        hostnames: Vec<String>,
        protocol: String,
        host: String,
        port: u16,
        init: bool,
    ) -> Result<(), Error> {
        let _ = mkdir(format!("server/listeners/{}/certs", name.to_string()));

        // If the protocol is `https`, create the server certificates
        if protocol == "https" {
            create_server_certs(
                name.to_string(),
                hostnames.to_owned(),
                host.to_string()
            );
        }

        let listener = Listener::new(
            name.to_string(),
            hostnames,
            protocol.to_string(),
            host.to_string(),
            port.to_owned(),
        );

        // Check duplicate in database if not init
        if !init {
            match db::exists_listener(
                self.db.path.to_string(),
                listener.clone())
            {
                Ok(exists) => {
                    if exists {
                        return Err(Error::new(ErrorKind::Other, "Listener already exists."));
                    }
                }
                Err(e) => {
                    return Err(Error::new(ErrorKind::Other, e.to_string()));
                }
            }
        }

        // Create a new job
        let mut jobs_lock = self.jobs.lock().await;
        let rx_job_lock = self.tx_job.lock().await;

        let new_job = Job::new(
            (jobs_lock.len() + 1) as u32,
            listener.clone(),
            Arc::new(Mutex::new(rx_job_lock.subscribe())),
            self.db.path.to_string(),
        );

        jobs_lock.push(new_job);

        // Add listener to database
        db::add_listener(self.db.path.to_string(), &listener).unwrap();

        Ok(())
    }

    pub async fn delete_listener(&mut self, listener_name: String) -> Result<(), Error> {
        let mut jobs = self.jobs.lock().await;
        let mut jobs_owned = jobs.to_owned();

        let job = match find_job(&mut jobs_owned, listener_name.to_owned()).await {
            Some(j) => j,
            None => {
                return Err(Error::new(ErrorKind::Other, "Listener not found."));
            }
        };

        if job.running {
            return Err(
                Error::new(
                    ErrorKind::Other,
                    "Listener cannot be deleted because it's running. Please stop it before deleting."
                ));
        }

        job.handle.lock().await.abort();
        jobs.remove((job.id - 1) as usize);

        // Remove listener from database
        db::delete_listener(
            self.db.path.to_string(),
            job.listener.name.to_string()).unwrap();

        Ok(())
    }
}

pub async fn run(config: Config) {
    // Initialize database
    let db = db::DB::new();
    let db_path = db.path.to_string();

    let (tx_job, _rx_job) = broadcast::channel(100);
    let server = Arc::new(Mutex::new(Server::new(config, db, tx_job)));

    // Load data from database or initialize
    if exists_file("server/hermit.db".to_string()) {
        // Load all listeners
        let all_listeners = db::get_all_listeners(db_path.to_string()).unwrap();
        if all_listeners.len() > 0 {
            let mut server_lock = server.lock().await;
            for listener in all_listeners {
                let _ = server_lock.add_listener(
                    listener.name,
                    listener.hostnames,
                    listener.protocol,
                    listener.host,
                    listener.port,
                    true,
                ).await;
            }
        }
    } else {
        mkfile(DB_PATH.to_string()).unwrap();
        db::init_db(db_path.to_string()).unwrap();
    }

    // Generate the root certificates if they don't exist yet.
    if  !exists_file("server/root_cert.pem".to_string()) ||
        !exists_file("server/root_key.pem".to_string())
    {
        let _ = create_root_ca();
    }

    // Generate kaypair (for secure communication with agents) if it does not exist yet in database
    let keypair_exists = match db::exists_keypair(db_path.to_string()) {
        Ok(exists) => exists,
        Err(e) => {
            error!("Error: {}", e.to_string());
            return;
        }
    };
    if !keypair_exists {
        let (secret, public) = aesgcm::generate_keypair();
        let encoded_secret = aesgcm::encode(secret.as_bytes());
        let encoded_public = aesgcm::encode(public.as_bytes());

        let _ = db::add_keypair(db_path.to_string(), encoded_secret, encoded_public);
    }

    let app = Router::new()
        .route("/hermit", get(ws_handler))
        .layer(
            AddExtensionLayer::new(server),
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:9999")
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

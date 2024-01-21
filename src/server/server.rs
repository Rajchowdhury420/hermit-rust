use log::{error, info};
use std::io::{Error, ErrorKind};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

use super::{
    certs::https::{create_server_certs, create_root_ca},
    crypto::aesgcm,
    db::{self, DB_PATH},
    grpc,
    listeners::listener::Listener,
    jobs::{find_job, Job, JobMessage},
};
use crate::{
    config::Config,
    utils::fs::{mkdir, mkfile, exists_file},
};

#[derive(Debug)]
pub struct HermitServer {
    pub config: Config,
    pub port: u16,
    pub db: db::DB,
    pub jobs: Arc<Mutex<Vec<Job>>>,  // Jobs contain listeners
    pub tx_job: Arc<Mutex<broadcast::Sender<JobMessage>>>,
}

impl HermitServer {
    pub fn new(
        config: Config,
        port: u16,
        db: db::DB,
        tx_job: broadcast::Sender<JobMessage>,
    ) -> Self {
        Self {
            config,
            port,
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

    pub async fn delete_all_listeners(&mut self) -> Result<(), Error> {
        self.jobs = Arc::new(Mutex::new(Vec::new()));
        db::delete_all_listeners(self.db.path.to_string()).unwrap();
        Ok(())
    }
}

pub async fn run(config: Config, host: String, port: u16) {
    // Initialize database
    let db = db::DB::new();
    let db_path_abs = db.path.to_string(); // This is an absolute path

    let (tx_job, _rx_job) = broadcast::channel(100);
    let server = Arc::new(Mutex::new(HermitServer::new(config, port.to_owned(), db, tx_job)));

    // Load data from database or initialize
    if exists_file(DB_PATH.to_string()) {
        // Load all listeners
        let all_listeners = db::get_all_listeners(db_path_abs.to_string()).unwrap();
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
        db::init_db(db_path_abs.to_string()).unwrap();
    }

    // Generate the root certificates if they don't exist yet.
    if  !exists_file("server/root_cert.pem".to_string()) ||
        !exists_file("server/root_key.pem".to_string())
    {
        let _ = create_root_ca();
    }

    // Generate kaypair for listeners (for secure communication with agents) if it does not exist yet in database
    let keypair_exists = match db::exists_keypair(db_path_abs.to_string()) {
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

        let _ = db::add_keypair(db_path_abs.to_string(), encoded_secret, encoded_public);
    }

    // Start gRPC server
    let addr: SocketAddr = format!("{host}:{port}").parse().unwrap();
    info!("Start gRPC server on http://{}:{}", addr.ip(), addr.port());
    let hermitrpc_service = grpc::HermitRpcService::new(server);
    tonic::transport::Server::builder()
        .add_service(grpc::pb_hermitrpc::hermit_rpc_server::HermitRpcServer::new(hermitrpc_service))
        .serve(addr)
        .await
        .unwrap();
}

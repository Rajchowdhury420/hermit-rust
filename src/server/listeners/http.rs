use axum::{
    extract::Request,
    routing::{get, post},
    Router
};
use hyper::body::Incoming;
use hyper_util::rt::TokioIo;
use log::info;
use std::time::Duration;
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast, watch};
use tower::Service;
use tower_http::trace::TraceLayer;
use tower_http::timeout::TimeoutLayer;

use crate::server::jobs::JobMessage;

pub async fn start_http_listener(job_id: u32, host: String, port: u16, receiver: Arc<Mutex<broadcast::Receiver<JobMessage>>>) {
    let app = Router::new()
        .route("/", get(hello))
        .route("/info", get(info))
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
                info!("Accepted.");
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

async fn hello() -> String {
    "Hello, World!".to_string()
}

async fn info() -> String {
    "System information".to_string()
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
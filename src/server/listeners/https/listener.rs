use axum::{
    extract::Request,
    routing::{get, post},
    Router,
};
use futures_util::pin_mut;
use hyper::body::Incoming;
use hyper_util::rt::TokioIo;
use log::{error, info};
use rustls_pemfile::{certs, private_key};
use std::{
    fs::File,
    io::BufReader,
    sync::Arc,
    time::Duration,
};
use tokio::{
    net::TcpListener,
    sync::{broadcast, Mutex, watch},
};
use tokio_rustls::{
    rustls::ServerConfig,
    TlsAcceptor,
};
use tower_http::{
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tower_service::Service;

use super::handlers::{
    handler_404,
    handler_home,
    handler_register,
    handler_task_ask,
    handler_task_result,
    handler_task_upload,
};
use crate::{
    server::jobs::JobMessage,
    utils::fs::get_app_dir,
};

fn init_router(db_path: String) -> Router {
    let route_home = "/";
    let route_register = "/r";
    let route_task_ask = "/t/a";
    let route_task_upload = "/t/u";
    let route_task_result = "/t/r";

    Router::new()
        .route(route_home, get(handler_home))
        .route(route_register, post(handler_register))
        .with_state(db_path.to_string())
        .route(route_task_ask, post(handler_task_ask))
        .with_state(db_path.to_string())
        .route(route_task_upload, post(handler_task_upload))
        .with_state(db_path.to_string())
        .route(route_task_result, post(handler_task_result))
        .with_state(db_path.to_string())
        .layer((
            TraceLayer::new_for_http(),
            TimeoutLayer::new(Duration::from_secs(10)),
        ))
}

pub async fn run(
    job_id: u32,
    name: String,
    host: String,
    port: u16,
    receiver: Arc<Mutex<broadcast::Receiver<JobMessage>>>,
    db_path: String,
) {

    let cert_path_abs = format!("{}/server/listeners/{}/certs/cert.pem", get_app_dir(), name.to_string());
    let key_path_abs = format!("{}/server/listeners/{}/certs/key.pem", get_app_dir(), name.to_string());

    let certs = certs(
        &mut BufReader::new(
            &mut File::open(cert_path_abs).unwrap()
        )).collect::<Result<Vec<_>, _>>().unwrap();

    let private_key = private_key(
        &mut BufReader::new(
            &mut BufReader::new(
                File::open(key_path_abs).unwrap()
            ))).unwrap().unwrap();

    let server_config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, private_key)
        .map_err(|err| {
            error!("{}", err);
            return;
        }).unwrap();

    let tls_acceptor = TlsAcceptor::from(Arc::new(server_config));

    let app = init_router(db_path.to_string());
    // Set a handler for unknown paths
    let app = app.fallback(handler_404);

    let listener = TcpListener::bind(format!("{host}:{port}"))
        .await
        .unwrap();

    info!("Start HTTPS listener on {}", listener.local_addr().unwrap());

    let (close_tx, close_rx) = watch::channel(());

    pin_mut!(listener);
    loop {
        let receiver_clone_1 = Arc::clone(&receiver);
        let receiver_clone_2 = Arc::clone(&receiver);

        let tls_acceptor = tls_acceptor.clone();

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

            // TLS handshake
            let Ok(tls_socket) = tls_acceptor.accept(socket).await else {
                error!("error during tls handshake connection from {}. Use HTTP instead of HTTPS.", remote_addr);
                return;
            };

            let socket = TokioIo::new(tls_socket);

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

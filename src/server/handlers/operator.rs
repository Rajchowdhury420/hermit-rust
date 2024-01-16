use axum::extract::ws::{Message, WebSocket};
use log::{error, info};
use std::{
    net::SocketAddr,
    sync::Arc,
};
use tokio::sync::{Mutex, MutexGuard};

use crate::server::{
    db,
    operators::{format_all_operators, format_operator_details, Operator},
    server::Server
};

pub async fn handle_operator(
    args: Vec<String>,
    socket_lock: &mut MutexGuard<'_, WebSocket>,
    operator_addr: SocketAddr,
    server: Arc<Mutex<Server>>,
) {
    let mut server_lock = server.lock().await;
    let mut operators = db::get_all_operators(server_lock.db.path.to_string()).unwrap();

    match args[1].as_str() {
        "add" => {
            let operator_name = args[2].to_string();
            let operator = Operator::new(
                0, // Temporary ID
                operator_name,
                operator_addr.to_string(),
            );
            let result = db::add_operator(server_lock.db.path.to_string(), operator);
            match result {
                Ok(_) => {
                    info!("New operator added.");
                },
                Err(e) => {
                    error!("Could not add new operator: {:?}", e);
                }
            }

            // It does not have to send response.
        }
        "info" => {
            let o_name = args[2].to_owned();
            let operator = match db::get_operator(server_lock.db.path.to_string(), o_name.to_string()) {
                Ok(imp) => imp,
                Err(e) => {
                    let _ = socket_lock.send(
                        Message::Text(
                            format!("[operator:info:error] {}", e.to_string()))).await;
                    let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
                    return;
                }
            };

            let output = format_operator_details(operator);
            let _ = socket_lock.send(Message::Text(output)).await;
            let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
        }
        "list" => {
            let output = format_all_operators(&mut operators);

            if output == "" {
                let _ = socket_lock.send(
                    Message::Text("[operator:list:error] Operator not found.".to_owned())).await;
            } else {
                let _ = socket_lock.send(Message::Text(output)).await;
            }
            let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
        }
        _ => {
            let _ = socket_lock.send(Message::Text("Unknown arguments".to_owned())).await;
            let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
            return;
        }
    }
}
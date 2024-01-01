use axum::extract::ws::{Message, WebSocket};
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};

use crate::server::{
    agents::format_agents,
    db,
    server::Server,
};

pub async fn handle_agent(
    args: Vec<String>,
    socket_lock: &mut MutexGuard<'_, WebSocket>,
    server: Arc<Mutex<Server>>
) {
    let server_lock = server.lock().await;
    let mut agents = db::get_all_agents(server_lock.db.path.to_string()).unwrap();

    match args[1].as_str() {
        "use" => {
            let ag_name = args[2].to_string();

            let mut is_ok = false;
            for agent in agents {
                if agent.id.to_string() == ag_name || agent.name == ag_name {
                    let _ = socket_lock.send(
                        Message::Text(format!("[agent:use:ok] {} {}", agent.name, agent.os))).await;
                    let _ = socket_lock.send(
                        Message::Text("[done]".to_owned())
                    ).await;
                    is_ok = true;
                    break;
                }
            }

            if !is_ok {
                let _ = socket_lock.send(
                    Message::Text("[agent:use:error] Agent not found.".to_owned())
                ).await;
                let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
            }
        }
        "list" => {
            let output = format_agents(&mut agents);
            let _ = socket_lock.send(Message::Text(output)).await;
            let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
        }
        _ => {
            let _ = socket_lock.send(Message::Text("Unknown arguments".to_owned())).await;
            let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
        }
    }
}
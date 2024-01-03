use axum::extract::ws::{Message, WebSocket};
use log::info;
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};

use crate::server::{
    db,
    implants::{
        generate::generate,
        implant::{format_implants, Implant},
    },
    server::Server
};

pub async fn handle_implant(
    message_text: String,
    args: Vec<String>,
    socket_lock: &mut MutexGuard<'_, WebSocket>,
    server: Arc<Mutex<Server>>,
) {
    let server_lock = server.lock().await;
    let mut implants = db::get_all_implants(server_lock.db.path.to_string()).unwrap();

    match args[1].as_str() {
        "gen" => {
            let i_name = args[2].to_owned();
            let i_listener_url = args[3].to_owned();
            let i_os = args[4].to_owned();
            let i_arch = args[5].to_owned();
            let i_format = args[6].to_owned();
            let i_sleep: u16 = args[7].to_owned().parse().unwrap();

            let implant = Implant::new(
                0, // Temporary ID
                i_name.to_owned(),
                i_listener_url.to_owned(),
                i_os.to_owned(),
                i_arch.to_owned(),
                i_format.to_owned(),
                i_sleep.to_owned(),
            );

            // Check duplicate
            let exists = db::exists_implant(server_lock.db.path.to_string(), implant.clone()).unwrap();
            if exists {
                let _ = socket_lock.send(
                    Message::Text(
                        "[implant:gen:error] Similar implant already exists. Please use it with `implant download`.".to_owned()
                    )).await;
                let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
                return;
            }

            // Generate an implant
            match generate(
                server_lock.db.path.to_string(),
                i_name.to_owned(),
                i_listener_url.to_owned(),
                i_os.to_owned(),
                i_arch.to_owned(),
                i_format.to_owned(),
                i_sleep,
            ) {
                Ok((output, mut buffer)) => {
                    let _ = send_implant_binary(
                        &mut buffer,
                        socket_lock,
                        output.to_owned(),
                    ).await;

                    // Add to the list (check duplicate again before adding it)
                    let exists = db::exists_implant(server_lock.db.path.to_string(), implant.clone()).unwrap();
                    if exists {
                        let _ = socket_lock.send(
                            Message::Text(
                                "[implant:gen:error] Similar implant already exists. Please use it with `implant download`.".to_owned()
                            )).await;
                        let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
                        return;
                    }
                    match db::add_implant(server_lock.db.path.to_string(), implant.clone()) {
                        Ok(_) => {}
                        Err(e) => {
                            let _ = socket_lock.send(
                                Message::Text(
                                    format!("[implant:gen:error] {}", e.to_string())
                                )).await;
                            let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
                            return;
                        }
                    }       
                },
                Err(e) => {
                    let _ = socket_lock.send(
                        Message::Text(format!("[implant:gen:error] Could not generate an imaplant: {e}"))
                    ).await;
                }
            }

            let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;

        }
        "download" => {
            let i_name = args[2].to_owned();
            // Get the implant
            let mut target_implant: Option<Implant> = None;
            for implant in implants {
                if implant.id.to_string() == i_name || implant.name == i_name {
                    target_implant = Some(implant.to_owned());
                    break;
                }
            }

            if let Some(imp) = target_implant {
                match generate(
                    server_lock.db.path.to_string(),
                    imp.name,
                    imp.listener_url,
                    imp.os,
                    imp.arch,
                    imp.format,
                    imp.sleep,
                ) {
                    Ok((output, mut buffer)) => {
                        let _ = send_implant_binary(
                            &mut buffer,
                            socket_lock,
                            output.to_owned(),
                        ).await;

                    },
                    Err(e) => {
                        let _ = socket_lock.send(
                            Message::Text(format!("[implant:gen:error] {e}"))
                        ).await;
                    }
                }
            } else {
                let _ = socket_lock.send(
                    Message::Text(
                        format!("[implant:gen:error] Implant not found."))).await;
            }

            let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
        }
        "delete" => {
            info!("Deleting the implant.");
            let i_name = args[2].to_owned();

            // Remove the implant from the list
            match db::delete_implant(server_lock.db.path.to_string(), i_name.to_string()) {
                Ok(_) => {
                    let _ = socket_lock.send(
                        Message::Text(
                            "[implant:delete:ok] Implant deleted successfully.".to_owned()
                        )).await;
                }
                Err(_) => {
                    let _ = socket_lock.send(
                        Message::Text(
                            "[implant:delete:error] Implant could not be delete.".to_owned()
                        )).await;
                }
            }
            let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
        }
        "list" => {
            let output = format_implants(&mut implants);
            let _ = socket_lock.send(Message::Text(output)).await;
            let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
        }
        _ => {
            let _ = socket_lock.send(Message::Text(format!("Unknown command: {message_text}"))).await;
            let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
        }
    }
}

async fn send_implant_binary(
    buffer: &mut Vec<u8>,
    socket_lock: &mut MutexGuard<'_, WebSocket>,
    output: String,
) {
    if buffer.len() > 10000000 {
                                                    
        // Split buffer
        let buffer2 = buffer.split_off(10000000);
        
        let _ = socket_lock.send(
            Message::Text("[implant:gen:ok:sending]".to_owned())).await;
        let _ = socket_lock.send(Message::Binary(buffer.to_vec())).await;

        let _ = socket_lock.send(
            Message::Text(format!(
                "[implant:gen:ok:complete] {}",
                output,
            ))).await;
        let _ = socket_lock.send(Message::Binary(buffer2.to_vec())).await;
    } else {
        let _ = socket_lock.send(
            Message::Text(format!(
                "[implant:gen:ok:complete] {}",
                output,
            ))).await;
        let _ = socket_lock.send(Message::Binary(buffer.to_vec())).await;
    }
}
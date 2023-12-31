use axum::extract::ws::{Message, WebSocket};
use log::info;
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};

use crate::{
    implants::{
        generate::generate,
        implant::{format_implants, Implant},
    },
    server::server::Server,
};

pub async fn handle_implant(
    message_text: String,
    args: Vec<String>,
    socket_lock: &mut MutexGuard<'_, WebSocket>,
    server: Arc<Mutex<Server>>,
) {
    let mut server_lock = server.lock().await;

    match args[1].as_str() {
        "gen" => {
            let i_name = args[2].to_owned();
            let i_listener_url = args[3].to_owned();
            let i_os = args[4].to_owned();
            let i_arch = args[5].to_owned();
            let i_format = args[6].to_owned();
            let i_sleep: u16 = args[7].to_owned().parse().unwrap();

            let mut implant = Implant::new(
                i_name.to_owned(),
                i_listener_url.to_owned(),
                i_os.to_owned(),
                i_arch.to_owned(),
                i_format.to_owned(),
                i_sleep.to_owned(),
            );

            // Check duplicate
            if server_lock.is_dupl_implant(&mut implant).await {
                let _ = socket_lock.send(
                    Message::Text(
                        "[implant:gen:error] Similar implant already exists. Please use it with `implant download`.".to_owned()
                    )).await;
                let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
                return;
            }

            // Generate an implant
            match generate(
                &server_lock.config,
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
                    if server_lock.is_dupl_implant(&mut implant).await {
                        let _ = socket_lock.send(
                            Message::Text(
                                "[implant:gen:error] Similar implant already exists. Please use it with `implant download`.".to_owned()
                            )).await;
                        let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
                        return;
                    }
                    if let Err(e) = server_lock.add_implant(&mut implant).await {
                        let _ = socket_lock.send(
                            Message::Text(format!("[implant:gen:error] {}", e.to_string()))).await;
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
            let implants = server_lock.implants.lock().await;
            for implant in implants.iter() {
                if implant.id.to_string() == i_name || implant.name == i_name {
                    target_implant = Some(implant.to_owned());
                    break;
                }
            }

            if let Some(imp) = target_implant {
                match generate(
                    &server_lock.config,
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
            let mut removed = false;
            let mut implants = server_lock.implants.lock().await;
            for (idx, implant) in implants.iter_mut().enumerate() {
                if implant.id.to_string() == i_name || implant.name == i_name {
                    implants.remove(idx);
                    removed = true;
                    break;
                }
            }

            if removed {
                let _ = socket_lock.send(
                    Message::Text(
                        "[implant:delete:ok] Implant deleted successfully.".to_owned()
                    )).await;
            } else {
                let _ = socket_lock.send(
                    Message::Text(
                        "[implant:delete:error] Implant could not be delete.".to_owned()
                    )).await;
            }
            let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
        }
        "list" => {
            let mut implants = server_lock.implants.lock().await;
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
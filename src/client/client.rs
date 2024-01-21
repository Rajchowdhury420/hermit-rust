use clap::{ArgMatches, Command};
use colored::Colorize;
use rustyline::{DefaultEditor, Result};
use rustyline::error::ReadlineError;
use std::process;

use super::{
    cli::cmd::create_cmd,
    handlers::{
        agent::{
            handle_agent_use,
            handle_agent_delete,
            handle_agent_info,
            handle_agent_list,
            handle_agent_task,
        },
        implant::{
            handle_implant_generate,
            handle_implant_download,
            handle_implant_delete,
            handle_implant_info,
            handle_implant_list,
        },
        listener::{
            handle_listener_add,
            handle_listener_delete,
            handle_listener_start,
            handle_listener_stop,
            handle_listener_info,
            handle_listener_list,
        },
        operator::{
            handle_operator_add,
            handle_operator_info,
            handle_operator_list,
        }
    },
    operations::{Operation, set_operations},
    options::options::Options,
    prompt::set_prompt,
};
use crate::{
    server::grpc,
    utils::fs::{write_file, get_app_dir},
};

const EXIT_SUCCESS: i32 = 0;
// const EXIT_FAILURE: i32 = 0;

#[derive(Debug)]
struct Commands {
    pub op: Operation,
    pub options: Options
}

impl Commands {
    fn new(op: Operation, options: Options) -> Self {
        Self {
            op,
            options,
        }
    }
}

pub enum Mode {
    Root,
    Agent(String, String), // Agent name, Agent OS
}

pub struct HermitClient {
    pub server_host: String,
    pub server_port: u16,

    pub operator_name: String,

    pub mode: Mode,
}

impl HermitClient {
    pub fn new(server_host: String, server_port: u16, operator_name: String) -> Self {
        Self {
            server_host,
            server_port,
            operator_name,
            mode: Mode::Root,
        }
    }

    // General CLI
    fn cli(&self) -> Command {
        create_cmd(self)
    }
    
    fn parse_args(&self, args: &[String]) -> clap::error::Result<Option<Commands>> {
        let matches = self.cli().try_get_matches_from(args)?;
        self.parse_matches(&matches)
    }
    
    fn parse_matches(&self, matches: &ArgMatches) -> clap::error::Result<Option<Commands>> {
        let (op, options) = set_operations(self, matches);
    
        Ok(Some(Commands::new(op, options)))
    }
    
    pub async fn run(&mut self) -> Result<()> {
        // Create a gRPC client and connect to the C2 server.
        let server_addr: tonic::transport::Uri = format!(
            "http://{}:{}",
            self.server_host,
            self.server_port
        ).parse().unwrap();
        let mut client = match grpc::pb_hermitrpc::hermit_rpc_client::HermitRpcClient::connect(server_addr.clone()).await {
            Ok(c) => c,
            Err(e) => {
                println!("{} Connection Error: {:?}", "[x]".red(), e);
                return Ok(());
            }
        };
        println!(
            "{} Connected the C2 server ({}://{}:{}) successfully.", "[+]".green(),
            server_addr.scheme().unwrap(), server_addr.host().unwrap(), server_addr.port().unwrap()
        );

        // Add the current operator
        let _ = handle_operator_add(&mut client, self.operator_name.to_string()).await;
    
        // Read a client command by reading lines.
        let mut rl = DefaultEditor::new()?;
        #[cfg(feature = "with-file-history")]
        if rl.load_history("history.txt").is_err() {
            println!("No previous history.");
        }
    
        loop {
            println!(""); // Add newline before the prompt for good appearance.
            let readline = rl.readline(
                set_prompt(&self.mode).as_str());
            match readline {
                Ok(line) => {
                    // Handle input
                    let _ = rl.add_history_entry(line.as_str());
                    let mut args = match shellwords::split(&line) {
                        Ok(args) => { args }
                        Err(err) => {
                            eprintln!("Can't parse command line: {err}");
                            vec!["".to_string()]
                        }
                    };
                    args.insert(0, "client".into());
                    // Parse options
                    let commands = match self.parse_args(&args) {
                        Ok(commands) => commands,
                        Err(err) => {
                            println!("{}", err);
                            continue;
                        }
                    };
    
                    if let Some(commands) = commands {
                        match &commands.op {
                            // Root operations
                            // Operator
                            Operation::InfoOperator => {
                                if let Some(operator_opt) = commands.options.operator_opt {
                                    if let Some(name) = operator_opt.name {
                                        let _ = handle_operator_info(&mut client, name).await;
                                    } else {
                                        println!("Specify an operator by ID or name.");
                                        continue;
                                    }
                                } else {
                                    continue;
                                }
                            }
                            Operation::ListOperators => {
                                let _ = handle_operator_list(&mut client).await;
                            }
                            // Listener
                            Operation::AddListener => {
                                if let Some(listener_opt) = commands.options.listener_opt {
                                    let _ = handle_listener_add(
                                        &mut client,
                                        listener_opt.name.unwrap(),
                                        listener_opt.domains.unwrap().join(","),
                                        listener_opt.proto.unwrap(),
                                        listener_opt.host.unwrap(),
                                        listener_opt.port.unwrap(),
                                    ).await;
                                } else {
                                    println!("Invalid command. Run `add help` for the usage.");
                                    continue;
                                }
                            }
                            Operation::DeleteListener => {
                                if let Some(listener_opt) = commands.options.listener_opt {
                                    if let Some(name) = listener_opt.name {
                                        let _ = handle_listener_delete(&mut client, name).await;
                                    } else {
                                        println!("Specify target listener by ID or name.");
                                        continue;
                                    }
                                } else {
                                    continue;
                                }
                            }
                            Operation::StartListener => {
                                if let Some(listener_opt) = commands.options.listener_opt {
                                    if let Some(name) = listener_opt.name {
                                        let _ = handle_listener_start(&mut client, name).await;
                                    } else {
                                        println!("Specify target listener by ID or name.");
                                        continue;
                                    }
                                } else {
                                    continue;
                                }
                            }
                            Operation::StopListener => {
                                if let Some(listener_opt) = commands.options.listener_opt {
                                    if let Some(name) = listener_opt.name {
                                        let _ = handle_listener_stop(&mut client, name).await;
                                    } else {
                                        println!("Specify target listener by ID or name.");
                                        continue;
                                    }
                                } else {
                                    continue;
                                }
                            }
                            Operation::InfoListener => {
                                if let Some(listener_opt) = commands.options.listener_opt {
                                    if let Some(name) = listener_opt.name {
                                        let _ = handle_listener_info(&mut client, name).await;
                                    } else {
                                        println!("Specify target listener by ID or name.");
                                        continue;
                                    }
                                } else {
                                    continue;
                                }
                            }
                            Operation::ListListeners => {
                                let _ = handle_listener_list(&mut client).await;
                            }
                            // Agent
                            Operation::UseAgent => {
                                if let Some(agent_opt) = commands.options.agent_opt {
                                    let ag_name = agent_opt.name;

                                    match handle_agent_use(&mut client, ag_name.to_string()).await {
                                        Ok(result) => {
                                            let res_split: Vec<String> = result
                                                .split(",")
                                                .map(|s| s.to_string()).collect();
                                            let agent_name = res_split[0].to_string();
                                            let agent_os = res_split[1].to_string();
                                            self.mode = Mode::Agent(agent_name, agent_os);
                                            println!("{} The agent found. Switch to the agent mode.", "[+]".green());
                                        }
                                        Err(e) => {
                                            println!("{} Error switching to the agent mode: {:?}", "[x]".red(), e);
                                        }
                                    }
                                }
                            }
                            Operation::DeleteAgent => {
                                if let Some(agent_opt) = commands.options.agent_opt {
                                    let ag_name = agent_opt.name;
                                    let _ = handle_agent_delete(&mut client, ag_name.to_string()).await;
                                }
                            }
                            Operation::InfoAgent => {
                                if let Some(agent_opt) = commands.options.agent_opt {
                                    let ag_name = agent_opt.name;
                                    let _ = handle_agent_info(&mut client, ag_name.to_string()).await;
                                }
                            }
                            Operation::ListAgents => {
                                let _ = handle_agent_list(&mut client).await;
                            }
                            // Implant
                            Operation::GenerateImplant => {
                                if let Some(implant_opt) = commands.options.implant_opt {
                                    let name = implant_opt.name.unwrap();
                                    let url = implant_opt.url.unwrap();
                                    let os = implant_opt.os.unwrap();
                                    let arch = implant_opt.arch.unwrap();
                                    let format = implant_opt.format.unwrap();
                                    let sleep = implant_opt.sleep.unwrap();
                                    let jitter = implant_opt.jitter.unwrap();
                                    let user_agent = implant_opt.user_agent.unwrap();

                                    let _ = handle_implant_generate(
                                        &mut client,
                                        name,
                                        url,
                                        os,
                                        arch,
                                        format,
                                        sleep,
                                        jitter,
                                        user_agent,
                                    ).await;
                                } else {
                                    continue;
                                }
                            }
                            Operation::DownloadImplant => {
                                if let Some(implant_opt) = commands.options.implant_opt {
                                    let name = implant_opt.name.unwrap();
                                    let _ = handle_implant_download(&mut client, name).await;
                                } else {
                                    continue;
                                }
                            }
                            Operation::DeleteImplant => {
                                if let Some(implant_opt) = commands.options.implant_opt {
                                    let name = implant_opt.name.unwrap();
                                    let _ = handle_implant_delete(&mut client, name).await;
                                } else {
                                    continue;
                                }
                            }
                            Operation::InfoImplant => {
                                if let Some(implant_opt) = commands.options.implant_opt {
                                    let name = implant_opt.name.unwrap();
                                    let _ = handle_implant_info(&mut client, name).await;
                                } else {
                                    continue;
                                }
                            }
                            Operation::ListImplants => {
                                let _ = handle_implant_list(&mut client).await;
                            }
                            // Misc
                            Operation::Empty => {
                                continue;
                            }
                            Operation::Error(e) => {
                                println!("Error: {}", e);
                                continue;
                            }
                            Operation::Exit => {
                                process::exit(EXIT_SUCCESS);
                            }
                            Operation::Unknown => {
                                println!("{} Unknown command. Run `help` for the usage.", "[!]".yellow());
                                continue;
                            }

                            // Agent operations
                            // Tasks
                            Operation::AgentTask(task) => {
                                let task_opt = commands.options.task_opt.unwrap();
                                let t_agent = task_opt.agent_name.unwrap();
                                let t_args = match task_opt.args {
                                    Some(a) => a,
                                    None => "".to_string(),
                                };

                                let _ = handle_agent_task(
                                    &mut client,
                                    t_agent.to_string(),
                                    task.to_string(),
                                    t_args.to_string()
                                ).await;
                            }
                            // Misc
                            Operation::AgentEmpty => {
                                continue;
                            }
                            Operation::AgentExit => {
                                println!("{} Exit the agent mode.", "[+]".green());
                                self.mode = Mode::Root;
                                continue;
                            }
                            Operation::AgentUnknown => {
                                println!("{} Unknown command. Run `help` for the usage.", "[!]".yellow());
                                continue;
                            }
                        }
                    }
                },
                Err(ReadlineError::Interrupted) => {
                    break
                },
                Err(ReadlineError::Eof) => {
                    break
                },
                Err(err) => {
                    println!("[x] {} {:?}", "Error: ", err);
                    continue;
                }
            }
        }
    
        #[cfg(feature = "with-file-history")]
        rl.save_history("history.txt");
    
        Ok(())
    }
}

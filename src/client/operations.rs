use clap::ArgMatches;

use super::{
    client::{Client, Mode},
    options::{
        agent::AgentOption,
        implant::ImplantOption,
        listener::ListenerOption,
        options::Options,
        task::TaskOption,
    },
};
use crate::utils::random::random_name;

#[derive(Debug)]
pub enum Operation {
    // Root operations
    // Common
    Empty,
    Exit,
    Unknown,
    // Listeners
    AddListener,
    DeleteListener,
    StartListener,
    StopListener,
    ListListeners,
    // Agents
    UseAgent,
    ListAgents,
    // Implants
    GenerateImplant,
    DownloadImplant,
    DeleteImplant,
    ListImplants,

    // Agent operations
    // Common
    AgentEmpty,
    AgentExit,
    AgentUnknown,
    // Tasks
    AgentTaskScreenshot,
    AgentTaskShell,
}

pub fn set_operations(client: &Client, matches: &ArgMatches) -> (Operation, Options) {
    let mut op = Operation::Empty;
    let mut options = Options::new();

    match &client.mode {
        Mode::Root => {
            match matches.subcommand() {
                // Listener
                Some(("listener", subm)) => {
                    match subm.subcommand() {
                        Some(("add", subm2)) => {
                            op = Operation::AddListener;
                            let name = match subm2.get_one::<String>("name") {
                                Some(n) => Some(n.to_owned()),
                                None => Some(random_name("listener".to_owned())),
                            };
                            let host = match subm2.get_one::<String>("host") {
                                Some(h) => Some(h.to_string()),
                                None => Some("127.0.0.1".to_string())
                            };
                            let listener_option = ListenerOption {
                                name,
                                proto: subm2.get_one::<String>("protocol").cloned(),
                                host,
                                port: subm2.get_one::<u16>("port").cloned(),
                            };
                            options.listener_opt = Some(listener_option);
                        }
                        Some(("delete", subm2)) => {
                            op = Operation::DeleteListener;
                            let target = match subm2.get_one::<String>("listener") {
                                Some(l) => { Some(l.to_owned()) },
                                None => { None },
                            };
        
                            options.listener_opt = Some(ListenerOption {
                                name: target,
                                proto: None,
                                host: None,
                                port: None,
                            });
                        }
                        Some(("start", subm2)) => {
                            op = Operation::StartListener;
                            let target = match subm2.get_one::<String>("listener") {
                                Some(l) => { Some(l.to_owned()) },
                                None => { None },
                            };
        
                            options.listener_opt = Some(ListenerOption {
                                name: target,
                                proto: None,
                                host: None,
                                port: None,
                            });
                        }
                        Some(("stop", subm2)) => {
                            op = Operation::StopListener;
                            let target = match subm2.get_one::<String>("listener") {
                                Some(l) => { Some(l.to_owned()) },
                                None => { None },
                            };
        
                            options.listener_opt = Some(ListenerOption {
                                name: target,
                                proto: None,
                                host: None,
                                port: None,
                            });
                        }
                        Some(("list", _)) => {
                            op = Operation::ListListeners;
                        }
                        _ => {
                            op = Operation::Unknown;
                        }
                    }
                }
                Some(("listeners", _)) => {
                    op = Operation::ListListeners;
                }
                // Agent
                Some(("agent", subm)) => {
                    match subm.subcommand() {
                        Some(("use", subm2)) => {
                            op = Operation::UseAgent;
                            let name = match subm2.get_one::<String>("name") {
                                Some(n) => { n.to_owned() },
                                None => { "0".to_owned() },
                            };
        
                            options.agent_opt = Some(AgentOption {
                                name,
                            });
                        }
                        Some(("list", _)) => {
                            op = Operation::ListAgents;
                        }
                        _ => {
                            op = Operation::Unknown;
                        }
                    }
                }
                Some(("agents", _)) => {
                    op = Operation::ListAgents;
                }
                // Implant
                Some(("implant", subm)) => {
                    match subm.subcommand() {
                        Some(("gen", subm2)) => {
                            op = Operation::GenerateImplant;
                            let name = match subm2.get_one::<String>("name") {
                                Some(n) => { n.to_owned() },
                                None => { random_name("implant".to_owned()) }
                            };
                            let listener_url = match subm2.get_one::<String>("listener") {
                                Some(n) => { n.to_owned() },
                                None => { "http://127.0.0.1:8000/".to_owned() }
                            };
                            let os = match subm2.get_one::<String>("os") {
                                Some(n) => { n.to_owned() },
                                None => { "windows".to_owned() }
                            };
                            let arch = match subm2.get_one::<String>("arch") {
                                Some(n) => { n.to_owned() },
                                None => { "amd64".to_owned() }
                            };
                            let format = match subm2.get_one::<String>("format") {
                                Some(n) => { n.to_owned() },
                                None => { "exe".to_owned() }
                            };
                            let sleep = match subm2.get_one::<u64>("sleep") {
                                Some(n) => { *n },
                                None => { 3 }
                            };
        
                            options.implant_opt = Some(ImplantOption {
                                name: Some(name),
                                listener_url: Some(listener_url),
                                os: Some(os),
                                arch: Some(arch),
                                format: Some(format),
                                sleep: Some(sleep),
                            });
                        }
                        Some(("download", subm2)) => {
                            op = Operation::DownloadImplant;
                            let name = match subm2.get_one::<String>("name") {
                                Some(n) => n.to_owned(),
                                None => "0".to_owned(),
                            };
        
                            options.implant_opt = Some(ImplantOption {
                                name: Some(name),
                                listener_url: None,
                                os: None,
                                arch: None,
                                format: None,
                                sleep: None,
                            });
                        }
                        Some(("delete", subm2)) => {
                            op = Operation::DeleteImplant;
                            let name = match subm2.get_one::<String>("name") {
                                Some(n) => n.to_owned(),
                                None => "0".to_owned(),
                            };

                            options.implant_opt = Some(ImplantOption {
                                name: Some(name),
                                listener_url: None,
                                os: None,
                                arch: None,
                                format: None,
                                sleep: None,
                            });
                        }
                        Some(("list", _)) => {
                            op = Operation::ListImplants;
                        }
                        _ => {
                            op = Operation::Unknown;
                        }
                    }
                }
                Some(("implants", _)) => {
                    op = Operation::ListImplants;
                }
                // Misc
                Some(("exit", _)) | Some(("quit", _)) => {
                    op = Operation::Exit;
                }
                None => {
                    op = Operation::Empty;
                }
                _ => {
                    op = Operation::Unknown;
                },
            }
        }
        Mode::Agent(agent_name) => {
            match matches.subcommand() {
                // Tasks
                Some(("screenshot", _)) => {
                    op = Operation::AgentTaskScreenshot;
                    options.task_opt = Some(TaskOption {
                        agent_name: Some(agent_name.to_owned()),
                        command: None,
                    });
                }
                Some(("shell", subm)) => {
                    op = Operation::AgentTaskShell;
                    let mut pre = "cmd";
                    if subm.get_flag("ps") {
                        pre = "powershell";
                    }
                    let command = match subm.get_one::<String>("command") {
                        Some(c) => { c.to_owned() }
                        None => { "".to_owned() }
                    };

                    options.task_opt = Some(TaskOption {
                        agent_name: Some(agent_name.to_owned()),
                        command: Some(pre.to_string() + " " + command.as_str()),
                    });
                }
                // Misc
                Some(("exit", _)) | Some(("quit", _)) => {
                    op = Operation::AgentExit;
                }
                None => {
                    op = Operation::AgentEmpty;
                }
                _ => {
                    op = Operation::AgentUnknown;
                }
            }
        }
    }


    (op, options)
}
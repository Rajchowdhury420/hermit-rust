use base64::prelude::*;
use clap::ArgMatches;

use super::{
    client::{HermitClient, Mode},
    options::{
        agent::AgentOption,
        implant::ImplantOption,
        listener::ListenerOption,
        options::Options,
        task::TaskOption, operator::OperatorOption,
    },
};
use crate::{
    server::listeners::https::generate_user_agent,
    utils::random::random_name,
};

#[derive(Debug)]
pub enum Operation {
    // Root operations
    // Common
    Empty,
    Error(String),
    Exit,
    Unknown,
    // Operators
    // AddOperator,
    // DeleteOperator,
    InfoOperator,
    ListOperators,
    // Listeners
    AddListener,
    DeleteListener,
    StartListener,
    StopListener,
    InfoListener,
    ListListeners,
    // Agents
    UseAgent,
    DeleteAgent,
    InfoAgent,
    ListAgents,
    // Implants
    GenerateImplant,
    DownloadImplant,
    DeleteImplant,
    InfoImplant,
    ListImplants,

    // Agent operations
    // Common
    AgentEmpty,
    AgentExit,
    AgentUnknown,
    AgentTask(String), // The argument is the task name
}

pub fn set_operations(client: &HermitClient, matches: &ArgMatches) -> (Operation, Options) {
    let mut op = Operation::Empty;
    let mut options = Options::new();

    match &client.mode {
        Mode::Root => {
            match matches.subcommand() {
                // Operator
                Some(("operator", subm)) => {
                    match subm.subcommand() {
                        // Some(("add", subm2)) => {
                        //     op = Operation::AddOperator;
                        //     let name = match subm2.get_one::<String>("name") {
                        //         Some(n) => Some(n.to_owned()),
                        //         None => Some(random_name("operator".to_owned())),
                        //     };

                        //     let operator_option = OperatorOption {
                        //         name,
                        //     };
                        //     options.operator_opt = Some(operator_option);
                        // }
                        // Some(("delete", subm2)) => {
                        //     op = Operation::DeleteOperator;
                        // }
                        Some(("info", subm2)) => {
                            op = Operation::InfoOperator;
                            let target = match subm2.get_one::<String>("operator") {
                                Some(n) => { Some(n.to_owned()) },
                                None => { None },
                            };

                            options.operator_opt = Some(OperatorOption {
                                name: target,
                            });
                        }
                        Some(("list", _)) => {
                            op = Operation::ListOperators;
                        }
                        _ => {
                            op = Operation::Unknown;
                        }
                    }
                }
                Some(("operators", _)) => {
                    op = Operation::ListOperators;
                }
                Some(("whoami", _)) => {
                    op = Operation::InfoOperator;
                    
                    options.operator_opt = Some(OperatorOption {
                        name: Some(client.operator_name.to_string()),
                    });
                }
                // Listener
                Some(("listener", subm)) => {
                    match subm.subcommand() {
                        Some(("add", subm2)) => {
                            op = Operation::AddListener;
                            let name = match subm2.get_one::<String>("name") {
                                Some(n) => Some(n.to_owned()),
                                None => Some(random_name("listener".to_owned())),
                            };
                            let domains: Option<Vec<String>> = match subm2.get_one::<String>("domains") {
                                Some(n) => {
                                    Some(n.split(",").map(|s| s.to_string()).collect())
                                }
                                None => Some(Vec::new()),
                            };

                            let listener_option = ListenerOption {
                                name,
                                domains,
                                proto: subm2.get_one::<String>("protocol").cloned(),
                                host: subm2.get_one::<String>("host").cloned(),
                                port: subm2.get_one::<u16>("port").cloned(),
                            };
                            options.listener_opt = Some(listener_option);
                        }
                        Some(("delete", subm2)) => {
                            op = Operation::DeleteListener;
                            let target = match subm2.get_one::<String>("listener") {
                                Some(n) => { Some(n.to_owned()) },
                                None => { None },
                            };
        
                            options.listener_opt = Some(ListenerOption {
                                name: target,
                                domains: None,
                                proto: None,
                                host: None,
                                port: None,
                            });
                        }
                        Some(("start", subm2)) => {
                            op = Operation::StartListener;
                            let target = match subm2.get_one::<String>("listener") {
                                Some(n) => { Some(n.to_owned()) },
                                None => { None },
                            };
        
                            options.listener_opt = Some(ListenerOption {
                                name: target,
                                domains: None,
                                proto: None,
                                host: None,
                                port: None,
                            });
                        }
                        Some(("stop", subm2)) => {
                            op = Operation::StopListener;
                            let target = match subm2.get_one::<String>("listener") {
                                Some(n) => { Some(n.to_owned()) },
                                None => { None },
                            };
        
                            options.listener_opt = Some(ListenerOption {
                                name: target,
                                domains: None,
                                proto: None,
                                host: None,
                                port: None,
                            });
                        }
                        Some(("info", subm2)) => {
                            op = Operation::InfoListener;
                            let target = match subm2.get_one::<String>("listener") {
                                Some(n) => { Some(n.to_owned()) },
                                None => { None },
                            };

                            options.listener_opt = Some(ListenerOption {
                                name: target,
                                domains: None,
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
                        Some(("delete", subm2)) => {
                            op = Operation::DeleteAgent;
                            let name = match subm2.get_one::<String>("name") {
                                Some(n) => { n.to_owned() },
                                None => { "0".to_owned() },
                            };

                            options.agent_opt = Some(AgentOption {
                                name,
                            });
                        }
                        Some(("info", subm2)) => {
                            op = Operation::InfoAgent;
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
                                Some(n) => n.to_string(),
                                None => { random_name("implant".to_owned()) }
                            };
                            let url = match subm2.get_one::<String>("url") {
                                Some(n) => n.to_string(),
                                None => { "http://localhost:4443/".to_owned() }
                            };
                            let os = match subm2.get_one::<String>("os") {
                                Some(n) => n.to_string(),
                                None => "windows".to_string(),
                            };
                            let arch = match subm2.get_one::<String>("arch") {
                                Some(n) => n.to_string(),
                                None => "amd64".to_string(),
                            };
                            let format = match subm2.get_one::<String>("format") {
                                Some(n) => n.to_string(),
                                None => "exe".to_string(),
                            };
                            let sleep = match subm2.get_one::<u64>("sleep") {
                                Some(n) => *n,
                                None => 3,
                            };
                            let jitter = match subm2.get_one::<u64>("jitter") {
                                Some(j) => *j,
                                None => 5,
                            };
                            let user_agent = match subm2.get_one::<String>("user-agent") {
                                Some(u) => u.to_string(),
                                None => generate_user_agent(os.to_string(), arch.to_string()),
                            };
                            let killdate = match subm2.get_one::<String>("killdate") {
                                Some(k) => k.to_string(),
                                None => "0".to_string(),
                            };
        
                            options.implant_opt = Some(ImplantOption {
                                name: Some(name),
                                url: Some(url),
                                os: Some(os),
                                arch: Some(arch),
                                format: Some(format),
                                sleep: Some(sleep),
                                jitter: Some(jitter),
                                user_agent: Some(user_agent),
                                killdate: Some(killdate),
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
                                url: None,
                                os: None,
                                arch: None,
                                format: None,
                                sleep: None,
                                jitter: None,
                                user_agent: None,
                                killdate: None,
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
                                url: None,
                                os: None,
                                arch: None,
                                format: None,
                                sleep: None,
                                jitter: None,
                                user_agent: None,
                                killdate: None,
                            });
                        }
                        Some(("info", subm2)) => {
                            op = Operation::InfoImplant;
                            let name = match subm2.get_one::<String>("name") {
                                Some(n) => n.to_owned(),
                                None => "0".to_owned(),
                            };

                            options.implant_opt = Some(ImplantOption {
                                name: Some(name),
                                url: None,
                                os: None,
                                arch: None,
                                format: None,
                                sleep: None,
                                jitter: None,
                                user_agent: None,
                                killdate: None,
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
        Mode::Agent(agent_name, agent_os) => {
            match matches.subcommand() {
                // Tasks
                Some(("cat", subm)) => {
                    op = Operation::AgentTask("cat".to_string());

                    let file = match subm.get_one::<String>("file") {
                        Some(f) => f.to_owned(),
                        None => "".to_owned(),
                    };

                    options.task_opt = Some(TaskOption {
                        agent_name: Some(agent_name.to_owned()),
                        args: Some(file),
                    });                    
                }
                Some(("cd", subm)) => {
                    op = Operation::AgentTask("cd".to_string());

                    let dir = match subm.get_one::<String>("directory") {
                        Some(d) => d.to_owned(),
                        None => "".to_owned(),
                    };

                    options.task_opt = Some(TaskOption {
                        agent_name: Some(agent_name.to_owned()),
                        args: Some(dir),
                    });
                }
                Some(("cp", subm)) => {
                    op = Operation::AgentTask("cp".to_string());

                    let src = match subm.get_one::<String>("source") {
                        Some(s) => s.to_owned(),
                        None => "".to_owned(),
                    };

                    let dest = match subm.get_one::<String>("dest") {
                        Some(d) => d.to_owned(),
                        None => "".to_owned(),
                    };

                    options.task_opt = Some(TaskOption {
                        agent_name: Some(agent_name.to_owned()),
                        args: Some(src + " " + dest.as_str()),
                    });
                }
                Some(("download", subm)) => {
                    op = Operation::AgentTask("download".to_string());

                    let file = match subm.get_one::<String>("file") {
                        Some(f) => f.to_owned(),
                        None => "".to_owned(),
                    };

                    options.task_opt = Some(TaskOption {
                        agent_name: Some(agent_name.to_owned()),
                        args: Some(file),
                    });
                }
                Some(("info", _)) => {
                    op = Operation::AgentTask("info".to_string());

                    options.task_opt = Some(TaskOption {
                        agent_name: Some(agent_name.to_owned()),
                        args: None,
                    });
                }
                Some(("ls", subm)) => {
                    op = Operation::AgentTask("ls".to_string());

                    let dir = match subm.get_one::<String>("directory") {
                        Some(d) => d.to_owned(),
                        None => "".to_owned(),
                    };

                    options.task_opt = Some(TaskOption {
                        agent_name: Some(agent_name.to_owned()),
                        args: Some(dir),
                    });
                }
                Some(("mkdir", subm)) => {
                    op = Operation::AgentTask("mkdir".to_string());

                    let dir = match subm.get_one::<String>("directory") {
                        Some(d) => d.to_owned(),
                        None => "".to_owned(),
                    };

                    options.task_opt = Some(TaskOption {
                        agent_name: Some(agent_name.to_owned()),
                        args: Some(dir),
                    });
                }
                Some(("net", subm)) => {
                    op = Operation::AgentTask("net".to_string());

                    options.task_opt = Some(TaskOption {
                        agent_name: Some(agent_name.to_owned()),
                        args: None,
                    });
                }
                Some(("ps", subm)) => {
                    op = Operation::AgentTask("ps".to_string());

                    match subm.subcommand() {
                        Some(("kill", subm2)) => {
                            let pid = match subm2.get_one::<u32>("pid") {
                                Some(p) => p.to_string(),
                                None => {
                                    op = Operation::Error("Process ID not specified.".to_string());
                                    "".to_string()
                                },
                            };

                            options.task_opt = Some(TaskOption {
                                agent_name: Some(agent_name.to_owned()),
                                args: Some("kill ".to_string() + pid.as_str()),
                            });
                        }
                        Some(("list", subm2)) => {
                            let filter = match subm2.get_one::<String>("filter") {
                                Some(f) => f.to_owned(),
                                None => "*".to_owned(),
                            };
        
                            let exclude = match subm2.get_one::<String>("exclude") {
                                Some(x) => x.to_owned(),
                                None => "".to_owned(),
                            };
        
                            options.task_opt = Some(TaskOption {
                                agent_name: Some(agent_name.to_owned()),
                                args: Some("list ".to_string() + filter.as_str() + ":" + exclude.as_str()),
                            });
                        }
                        _ => {
                            op = Operation::Error("Subcommand not specified.".to_string());
                        }
                    }

                }
                Some(("pwd", _)) => {
                    op = Operation::AgentTask("pwd".to_string());
                    options.task_opt = Some(TaskOption {
                        agent_name: Some(agent_name.to_owned()),
                        args: None,
                    });
                }
                Some(("rm", subm)) => {
                    op = Operation::AgentTask("rm".to_string());
                    let mut file = match subm.get_one::<String>("file") {
                        Some(f) => f.to_owned(),
                        None => "".to_owned(),
                    };

                    if subm.get_flag("recursive") {
                        file = file + " -r";
                    }

                    options.task_opt = Some(TaskOption {
                        agent_name: Some(agent_name.to_owned()),
                        args: Some(file),
                    });
                }
                Some(("screenshot", _)) => {
                    op = Operation::AgentTask("screenshot".to_string());
                    options.task_opt = Some(TaskOption {
                        agent_name: Some(agent_name.to_owned()),
                        args: None,
                    });
                }
                Some(("shell", subm)) => {
                    op = Operation::AgentTask("shell".to_string());

                    match agent_os.as_str() {
                        "linux" => {
                            let command = match subm.get_one::<String>("command") {
                                Some(c) => c.to_owned(),
                                None => "".to_owned(),
                            };

                            options.task_opt = Some(TaskOption {
                                agent_name: Some(agent_name.to_owned()),
                                args: Some(command),
                            });
                        }
                        "windows" | _ => {
                            let mut pre = "cmd";
                            if subm.get_flag("ps") {
                                pre = "powershell";
                            }
                            let command = match subm.get_one::<String>("command") {
                                Some(c) => c.to_owned(),
                                None => "".to_owned(),
                            };
        
                            options.task_opt = Some(TaskOption {
                                agent_name: Some(agent_name.to_owned()),
                                args: Some(pre.to_string() + " " + command.as_str()),
                            });
                        }
                    }

                }
                Some(("shellcode", subm)) => {
                    op = Operation::AgentTask("shellcode".to_string());

                    let pid = match subm.get_one::<u32>("pid") {
                        Some(p) => p.to_string(),
                        None => "".to_string(),
                    };

                    let proc_name = match subm.get_one::<String>("process") {
                        Some(p) => p.to_string(),
                        None => "".to_string(),
                    };

                    let mut args = String::new();

                    if pid != "".to_string() {
                        args = "pid ".to_owned() + pid.as_str();
                    } else if proc_name != "".to_string() {
                        args = "process ".to_owned() + proc_name.as_str();
                    } else {
                        args = "process svchost".to_string();
                    }

                    let shellcode = match subm.get_one::<String>("file") {
                        Some(f) => {
                            let shellcode = match std::fs::read(f.to_string()) {
                                Ok(s) => s,
                                Err(e) => {
                                    // println!("Error reading shellcode file: {}", e);
                                    op = Operation::Error(format!("Error reading shellcode file: {}", e));
                                    "nop".as_bytes().to_vec()
                                }
                            };

                            base64::prelude::BASE64_STANDARD.encode(shellcode)
                        },
                        None => {
                            op = Operation::Error("Shellcode file not specified.".to_string());
                            base64::prelude::BASE64_STANDARD.encode("nop".as_bytes().to_vec())
                        },
                    };

                    args = args + " " + shellcode.as_str();

                    options.task_opt = Some(TaskOption {
                        agent_name: Some(agent_name.to_owned()),
                        args: Some(args),
                    });
                }
                Some(("sleep", subm)) => {
                    op = Operation::AgentTask("sleep".to_string());

                    let sleeptime = match subm.get_one::<u64>("time") {
                        Some(t) => *t,
                        None => 3 as u64,
                    };

                    options.task_opt = Some(TaskOption {
                        agent_name: Some(agent_name.to_owned()),
                        args: Some(sleeptime.to_string()),
                    });
                }
                Some(("upload", subm)) => {
                    op = Operation::AgentTask("upload".to_string());

                    let uploaded_file = match subm.get_one::<String>("file") {
                        Some(f) => f.to_string(),
                        None => "".to_string(),
                    };
                    let dest = match subm.get_one::<String>("dest") {
                        Some(d) => d.to_string(),
                        None => "".to_string(),
                    };

                    options.task_opt = Some(TaskOption {
                        agent_name: Some(agent_name.to_owned()),
                        args: Some(uploaded_file + " " + dest.as_str()),
                    });
                }
                Some(("whoami", _)) => {
                    op = Operation::AgentTask("whoami".to_string());
                    options.task_opt = Some(TaskOption {
                        agent_name: Some(agent_name.to_owned()),
                        args: None,
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
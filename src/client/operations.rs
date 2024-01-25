use base64::prelude::*;
use clap::ArgMatches;

use super::client::{HermitClient, Mode};
use crate::{
    server::listeners::https::generate_user_agent,
    utils::random::random_name,
};

#[derive(Debug)]
pub enum RootOperation {
    Exit,

    // Operator
    // OperatorAdd,
    // OperatorDelete,
    OperatorInfo { name: String },
    OperatorList,

    // Listener
    ListenerAdd {
        name: String,
        domains: Vec<String>,
        proto: String,
        host: String,
        port: u16,
    },
    ListenerDelete { name: String },
    ListenerStart { name: String },
    ListenerStop { name: String },
    ListenerInfo { name: String },
    ListenerList,

    // Agent
    AgentUse { name: String },
    AgentDelete { name: String },
    AgentInfo { name: String },
    AgentList,

    // Implant
    ImplantGenerate {
        name: String,
        url: String,
        os: String,
        arch: String,
        format: String,
        sleep: u64,
        jitter: u64,
        user_agent: String,
        killdate: String,
    },
    ImplantDownload { name: String },
    ImplantDelete { name: String },
    ImplantInfo { name: String },
    ImplantList,
}

#[derive(Debug)]
pub enum AgentOperation {
    Exit,
    Task {
        agent: String,  // An agent name
        task: String,   // A task name e.g. "cat"
        args: String,   // Task arguments e.g. a specified file name
    },
}

#[derive(Debug)]
pub enum Operation {
    Root(RootOperation),
    Agent(AgentOperation),

    Empty,
    Error { message: String },
    Unknown,
}

pub fn set_operation(client: &HermitClient, matches: &ArgMatches) -> Operation {
    // let mut op = Operation::Root(RootOperation::Empty);

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
                            let target = match subm2.get_one::<String>("operator") {
                                Some(n) => n.to_owned(),
                                None => return Operation::Error { message: "Invalid argument.".to_string() },
                            };
                            return Operation::Root(RootOperation::OperatorInfo { name: target });
                        }
                        Some(("list", _)) => {
                            return Operation::Root(RootOperation::OperatorList);
                        }
                        _ => {
                            return Operation::Unknown;
                        }
                    }
                }
                Some(("operators", _)) => {
                    return Operation::Root(RootOperation::OperatorList);
                }
                Some(("whoami", _)) => {
                    return Operation::Root(
                        RootOperation::OperatorInfo {
                            name: client.operator_name.to_string()
                        });
                }
                // Listener
                Some(("listener", subm)) => {
                    match subm.subcommand() {
                        Some(("add", subm2)) => {
                            let name = match subm2.get_one::<String>("name") {
                                Some(n) => n.to_owned(),
                                None => random_name("listener".to_owned()),
                            };
                            let domains: Vec<String> = match subm2.get_one::<String>("domains") {
                                Some(n) => n.split(",").map(|s| s.to_string()).collect(),
                                None => Vec::new(),
                            };
                            return Operation::Root(RootOperation::ListenerAdd {
                                name,
                                domains,
                                proto: subm2.get_one::<String>("protocol").unwrap().to_string(),
                                host: subm2.get_one::<String>("host").unwrap().to_string(),
                                port: *subm2.get_one::<u16>("port").unwrap(),
                            })
                        }
                        Some(("delete", subm2)) => {
                            let target = match subm2.get_one::<String>("listener") {
                                Some(n) => n.to_owned(),
                                None => return Operation::Error { message: "Invalid argument.".to_string() },
                            };
                            return Operation::Root(RootOperation::ListenerDelete { name: target });
                        }
                        Some(("start", subm2)) => {
                            let target = match subm2.get_one::<String>("listener") {
                                Some(n) => n.to_owned(),
                                None => return Operation::Error { message: "Invalid argument.".to_string() },
                            };
                            return Operation::Root(RootOperation::ListenerStart { name: target });
                        }
                        Some(("stop", subm2)) => {
                            let target = match subm2.get_one::<String>("listener") {
                                Some(n) => n.to_owned(),
                                None => return Operation::Error { message: "Invalid argument.".to_string() },
                            };
                            return Operation::Root(RootOperation::ListenerStop { name: target });
                        }
                        Some(("info", subm2)) => {
                            let target = match subm2.get_one::<String>("listener") {
                                Some(n) => n.to_owned(),
                                None => return Operation::Error { message: "Invalid argument.".to_string() },
                            };
                            return Operation::Root(RootOperation::ListenerInfo { name: target });
                        }
                        Some(("list", _)) => {
                            return Operation::Root(RootOperation::ListenerList);
                        }
                        _ => {
                            return Operation::Unknown;
                        }
                    }
                }
                Some(("listeners", _)) => {
                    return Operation::Root(RootOperation::ListenerList);
                }
                // Agent
                Some(("agent", subm)) => {
                    match subm.subcommand() {
                        Some(("use", subm2)) => {
                            let name = match subm2.get_one::<String>("name") {
                                Some(n) => n.to_owned(),
                                None => "0".to_owned(),
                            };
                            return Operation::Root(RootOperation::AgentUse { name });
                        }
                        Some(("delete", subm2)) => {
                            let name = match subm2.get_one::<String>("name") {
                                Some(n) => { n.to_owned() },
                                None => { "0".to_owned() },
                            };
                            return Operation::Root(RootOperation::AgentDelete { name });
                        }
                        Some(("info", subm2)) => {
                            let name = match subm2.get_one::<String>("name") {
                                Some(n) => n.to_owned(),
                                None => "0".to_owned(),
                            };
                            return Operation::Root(RootOperation::AgentInfo { name });
                        }
                        Some(("list", _)) => {
                            return Operation::Root(RootOperation::AgentList);
                        }
                        _ => {
                            return Operation::Unknown;
                        }
                    }
                }
                Some(("agents", _)) => {
                    return Operation::Root(RootOperation::AgentList);
                }
                // Implant
                Some(("implant", subm)) => {
                    match subm.subcommand() {
                        Some(("gen", subm2)) => {
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
                            return Operation::Root(RootOperation::ImplantGenerate {
                                name,
                                url,
                                os,
                                arch,
                                format,
                                sleep,
                                jitter,
                                user_agent,
                                killdate,
                            });
                        }
                        Some(("download", subm2)) => {
                            let name = match subm2.get_one::<String>("name") {
                                Some(n) => n.to_owned(),
                                None => "0".to_owned(),
                            };
                            return Operation::Root(RootOperation::ImplantDownload { name });
                        }
                        Some(("delete", subm2)) => {
                            let name = match subm2.get_one::<String>("name") {
                                Some(n) => n.to_owned(),
                                None => "0".to_owned(),
                            };
                            return Operation::Root(RootOperation::ImplantDelete { name });
                        }
                        Some(("info", subm2)) => {
                            let name = match subm2.get_one::<String>("name") {
                                Some(n) => n.to_owned(),
                                None => "0".to_owned(),
                            };
                            return Operation::Root(RootOperation::ImplantInfo { name });
                        }
                        Some(("list", _)) => {
                            return Operation::Root(RootOperation::ImplantList);
                        }
                        _ => {
                            return Operation::Unknown;
                        }
                    }
                }
                Some(("implants", _)) => {
                    return Operation::Root(RootOperation::ImplantList);
                }
                // Misc
                Some(("exit", _)) | Some(("quit", _)) => {
                    return Operation::Root(RootOperation::Exit);
                }
                None => {
                    return Operation::Empty;
                }
                _ => {
                    return Operation::Unknown;
                },
            }
        }
        Mode::Agent(agent_name, agent_os) => {
            match matches.subcommand() {
                // Tasks
                Some(("cat", subm)) => {
                    let file = match subm.get_one::<String>("file") {
                        Some(f) => f.to_owned(),
                        None => "".to_owned(),
                    };
                    return Operation::Agent(AgentOperation::Task {
                        agent: agent_name.to_string(),
                        task: "cat".to_string(),
                        args: file,
                    });
                }
                Some(("cd", subm)) => {
                    let dir = match subm.get_one::<String>("directory") {
                        Some(d) => d.to_owned(),
                        None => "".to_owned(),
                    };
                    return Operation::Agent(AgentOperation::Task {
                        agent: agent_name.to_string(),
                        task: "cd".to_string(),
                        args: dir,
                    });
                }
                Some(("cp", subm)) => {
                    let src = match subm.get_one::<String>("source") {
                        Some(s) => s.to_owned(),
                        None => "".to_owned(),
                    };
                    let dest = match subm.get_one::<String>("dest") {
                        Some(d) => d.to_owned(),
                        None => "".to_owned(),
                    };
                    return Operation::Agent(AgentOperation::Task {
                        agent: agent_name.to_string(),
                        task: "cp".to_string(),
                        args: src + " " + dest.as_str(),
                    });
                }
                Some(("download", subm)) => {
                    let file = match subm.get_one::<String>("file") {
                        Some(f) => f.to_owned(),
                        None => "".to_owned(),
                    };
                    return Operation::Agent(AgentOperation::Task {
                        agent: agent_name.to_string(),
                        task: "download".to_string(),
                        args: file,
                    });
                }
                Some(("info", _)) => {
                    return Operation::Agent(AgentOperation::Task {
                        agent: agent_name.to_string(),
                        task: "info".to_string(),
                        args: "".to_string(),
                    });
                }
                Some(("ls", subm)) => {
                    let dir = match subm.get_one::<String>("directory") {
                        Some(d) => d.to_owned(),
                        None => ".".to_owned(),
                    };
                    return Operation::Agent(AgentOperation::Task {
                        agent: agent_name.to_string(),
                        task: "ls".to_string(),
                        args: dir,
                    });
                }
                Some(("mkdir", subm)) => {
                    let dir = match subm.get_one::<String>("directory") {
                        Some(d) => d.to_owned(),
                        None => {
                            return Operation::Error {
                                message: "Specify a directory to create.".to_string()
                            };
                        },
                    };
                    return Operation::Agent(AgentOperation::Task {
                        agent: agent_name.to_string(),
                        task: "mkdir".to_string(),
                        args: dir,
                    });
                }
                Some(("net", subm)) => {
                    // TODO: Implement `net status <interface>`, `net stop <interface>`, etc.
                    return Operation::Agent(AgentOperation::Task {
                        agent: agent_name.to_string(),
                        task: "net".to_string(),
                        args: "".to_string(),
                    })
                }
                Some(("ps", subm)) => {
                    match subm.subcommand() {
                        Some(("kill", subm2)) => {
                            let pid = match subm2.get_one::<u32>("pid") {
                                Some(p) => p.to_string(),
                                None => {
                                    return Operation::Error {
                                        message: "Process ID not specified.".to_string(),
                                    };
                                },
                            };
                            return Operation::Agent(AgentOperation::Task {
                                agent: agent_name.to_string(),
                                task: "ps".to_string(),
                                args: "kill ".to_string() + pid.as_str(),
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
                            return Operation::Agent(AgentOperation::Task {
                                agent: agent_name.to_string(),
                                task: "ps".to_string(),
                                args: "list ".to_string() + filter.as_str() + ":" + exclude.as_str(),
                            });
                        }
                        _ => {
                            return Operation::Error {
                                message: "Subcommand not specified.".to_string(),
                            };
                        }
                    }
                }
                Some(("pwd", _)) => {
                    return Operation::Agent(AgentOperation::Task {
                        agent: agent_name.to_string(),
                        task: "pwd".to_string(),
                        args: "".to_string(),
                    });
                }
                Some(("rm", subm)) => {
                    let mut file = match subm.get_one::<String>("file") {
                        Some(f) => f.to_owned(),
                        None => "".to_owned(),
                    };

                    if subm.get_flag("recursive") {
                        file = file + " -r";
                    }

                    return Operation::Agent(AgentOperation::Task {
                        agent: agent_name.to_string(),
                        task: "rm".to_string(),
                        args: file,
                    });
                }
                Some(("screenshot", _)) => {
                    return Operation::Agent(AgentOperation::Task {
                        agent: agent_name.to_string(),
                        task: "screenshot".to_string(),
                        args: "".to_string(),
                    });
                }
                Some(("shell", subm)) => {
                    let mut args = String::new();
                    match agent_os.as_str() {
                        "linux" => {
                            args = match subm.get_one::<String>("command") {
                                Some(c) => c.to_owned(),
                                None => "".to_owned(),
                            };
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

                            args = pre.to_string() + " " + command.as_str();
                        }
                    }

                    return Operation::Agent(AgentOperation::Task {
                        agent: agent_name.to_string(),
                        task: "shell".to_string(),
                        args,
                    });
                }
                Some(("shellcode", subm)) => {
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
                                    return Operation::Error {
                                        message: format!("Error reading shellcode file: {}", e),
                                    };
                                }
                            };

                            base64::prelude::BASE64_STANDARD.encode(shellcode)
                        },
                        None => {
                            return Operation::Error {
                                message: "Shellcode file not specified.".to_string(),
                            };
                        },
                    };

                    args = args + " " + shellcode.as_str();

                    return Operation::Agent(AgentOperation::Task {
                        agent: agent_name.to_string(),
                        task: "shellcode".to_string(),
                        args,
                    })
                }
                Some(("sleep", subm)) => {
                    let sleeptime = match subm.get_one::<u64>("time") {
                        Some(t) => t.to_string(),
                        None => "3".to_string(),
                    };
                    return Operation::Agent(AgentOperation::Task {
                        agent: agent_name.to_string(),
                        task: "sleep".to_string(),
                        args: sleeptime,
                    });
                }
                Some(("upload", subm)) => {
                    let uploaded_file = match subm.get_one::<String>("file") {
                        Some(f) => f.to_string(),
                        None => "".to_string(),
                    };
                    let dest = match subm.get_one::<String>("dest") {
                        Some(d) => d.to_string(),
                        None => "".to_string(),
                    };

                    return Operation::Agent(AgentOperation::Task {
                        agent: agent_name.to_string(),
                        task: "upload".to_string(),
                        args: uploaded_file + " " + dest.as_str(),
                    });
                }
                Some(("whoami", _)) => {
                    return Operation::Agent(AgentOperation::Task {
                        agent: agent_name.to_string(),
                        task: "whoami".to_string(),
                        args: "".to_string(),
                    });
                }
                // Misc
                Some(("exit", _)) | Some(("quit", _)) => {
                    return Operation::Agent(AgentOperation::Exit);
                }
                None => {
                    return Operation::Empty;
                }
                _ => {
                    return Operation::Unknown;
                }
            }
        }
    }
}
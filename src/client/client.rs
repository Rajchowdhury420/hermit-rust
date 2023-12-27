use clap::{Arg, ArgMatches, value_parser, Command};
use colored::Colorize;
use futures_util::{SinkExt, StreamExt};
use rustyline::{DefaultEditor, Result};
use rustyline::error::ReadlineError;
use spinners::{Spinner, Spinners};
use std::fs;
use std::process;
use std::sync::{Arc, Mutex};
use tokio_tungstenite::{
    connect_async,
    tungstenite::protocol::Message,
};

use super::options::{
    agent::AgentOption,
    implant::ImplantOption,
    listener::ListenerOption,
    options::Options
};
use super::prompt::set_prompt;
use crate::utils::random::random_name;

const EXIT_SUCCESS: i32 = 0;
const EXIT_FAILURE: i32 = 0;

#[derive(Debug)]
pub enum Operation {
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
    ListImplants,

    // Instruction,
}

#[derive(Debug)]
struct Commands {
    pub op: Operation,
    pub options: Options
    // pub cmd: String,
}

impl Commands {
    fn new(op: Operation, options: Options) -> Self {
        Self {
            op,
            options,
            // cmd,
        }
    }
}

pub struct Client {
    server_host: String,
    server_port: u16,

    mode: String,
}

impl Client {
    pub fn new(server_host: String, server_port: u16) -> Self {
        Self {
            server_host,
            server_port,
            mode: String::new(),
        }
    }

    // General CLI
    fn cli(&self) -> Command {
        Command::new("client")
            .about("Hermit C2 client")
            .allow_external_subcommands(true)
            .subcommand(Command::new("exit")
                .about("Close the connection and exit.")
            )
            // Listeners
            .subcommand(Command::new("listener")
                .about("Manage listeners.")
                .subcommand(Command::new("add")
                    .about("Add a new listener.")
                    .args([
                        Arg::new("protocol")
                            .help("Protocol")
                            .default_value("http")
                            .value_parser(value_parser!(String)),
                        Arg::new("host")
                            .short('H')
                            .long("host")
                            .help(format!("Host [default: {}]", self.server_host.to_string()))
                            .value_parser(value_parser!(String)),
                        Arg::new("port")
                            .short('P')
                            .long("port")
                            .help("Port")
                            .required(true)
                            .value_parser(value_parser!(u16)),
                        Arg::new("name")
                            .short('n')
                            .long("name")
                            .help("Specify the name of a listener")
                            .value_parser(value_parser!(String)),
                    ])
                )
                .subcommand(Command::new("delete")
                    .about("Delete a listener.")
                    .arg(Arg::new("listener")
                        .help("Listener ID or name to delete")
                        .required(true)
                        .value_parser(value_parser!(String))
                    )                    
                )
                .subcommand(Command::new("start")
                    .about("Start a listener.")
                    .arg(Arg::new("listener")
                        .help("Listener ID or name to start")
                        .required(true)
                        .value_parser(value_parser!(String))
                    )
                )
                .subcommand(Command::new("stop")
                    .about("Stop a listener.")
                    .arg(Arg::new("listener")
                        .help("Listener ID or name to stop")
                        .required(true)
                        .value_parser(value_parser!(String))
                    )
                )
                .subcommand(Command::new("list")
                    .about("List listeners.")
                )
            )
            .subcommand(Command::new("listeners")
                .about("List listeners.")
            )
            // Agents
            .subcommand(Command::new("agent")
                .about("Manage agents.")
                .subcommand(Command::new("use")
                    .about("Switch to specified agent shell.")
                    .arg(Arg::new("name")
                        .help("Agent ID or name")
                        .required(true)
                        .value_parser(value_parser!(String)))
                )
                .subcommand(Command::new("list")
                    .about("List agents.")
                )
            )
            .subcommand(Command::new("agents")
                .about("List agents.")
            )
            // Implants
            .subcommand(Command::new("implant")
                .about("Manage implants.")
                .subcommand(Command::new("gen")
                    .about("Generate a new implant.")
                    .args([
                        Arg::new("name")
                            .short('n')
                            .long("name")
                            .help("Set an implant name")
                            .value_parser(value_parser!(String)),
                        Arg::new("listener")
                            .short('l')
                            .long("listener")
                            .help("Listener URL that an agent connect to")
                            .default_value("http://127.0.0.1:8000/")
                            .value_parser(value_parser!(String)),
                        Arg::new("os")
                            .short('o')
                            .long("os")
                            .help("Target OS")
                            .default_value("windows")
                            .value_parser(value_parser!(String)),
                        Arg::new("arch")
                            .short('a')
                            .long("arch")
                            .help("Target architecture")
                            .default_value("amd64")
                            .value_parser(value_parser!(String)),
                        Arg::new("format")
                            .short('f')
                            .long("format")
                            .help("File format to be generated")
                            .default_value("exe")
                            .value_parser(value_parser!(String)),
                        Arg::new("sleep")
                            .short('s')
                            .long("sleep")
                            .help("Sleep time for each request to listener")
                            .default_value("3")
                            .value_parser(value_parser!(u64)),
                    ])
                )
                .subcommand(Command::new("download")
                    .about("Download specified implant.")
                    .arg(Arg::new("implant")
                        .help("Implant ID or name."))
                )
                .subcommand(Command::new("list")
                    .about("List implants generated.")
                )
            )
            .subcommand(Command::new("implants")
                .about("List implants generated.")
            )
            // Instructions
            // .subcommand("download")
    }
    
    fn parse_args(&self, args: &[String]) -> clap::error::Result<Option<Commands>> {
        let matches = self.cli().try_get_matches_from(args)?;
        self.parse_matches(&matches)
    }
    
    fn parse_matches(&self, matches: &ArgMatches) -> clap::error::Result<Option<Commands>> {
        let mut op = Operation::Empty;
        // let mut cmd = "".to_string();
        let mut options = Options::new();
    
        match matches.subcommand() {
            // Listener
            Some(("listener", subm)) => {
                match subm.subcommand() {
                    Some(("add", subm2)) => {
                        op = Operation::AddListener;
                        let name = match subm2.get_one::<String>("name") {
                            Some(n) => Some(n.to_string()),
                            None => Some(random_name("listener".to_string())),
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
                            Some(l) => { Some(l.to_string()) },
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
                            Some(l) => { Some(l.to_string()) },
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
                            Some(l) => { Some(l.to_string()) },
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
                            Some(n) => { n.to_string() },
                            None => { "0".to_string() },
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
                            Some(n) => { n.to_string() },
                            None => { random_name("implant".to_string()) }
                        };
                        let listener_url = match subm2.get_one::<String>("listener") {
                            Some(n) => { n.to_string() },
                            None => { "http://127.0.0.1:8000/".to_string() }
                        };
                        let os = match subm2.get_one::<String>("os") {
                            Some(n) => { n.to_string() },
                            None => { "windows".to_string() }
                        };
                        let arch = match subm2.get_one::<String>("arch") {
                            Some(n) => { n.to_string() },
                            None => { "amd64".to_string() }
                        };
                        let format = match subm2.get_one::<String>("format") {
                            Some(n) => { n.to_string() },
                            None => { "exe".to_string() }
                        };
                        let sleep = match subm2.get_one::<u64>("sleep") {
                            Some(n) => { *n },
                            None => { 3 }
                        };

                        options.implant_opt = Some(ImplantOption {
                            name,
                            listener_url,
                            os,
                            arch,
                            format,
                            sleep,
                        });
                    }
                    Some(("download", subm2)) => {
                        op = Operation::DownloadImplant;
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
            None => {
                op = Operation::Empty;
            }
            Some(("exit", _)) | Some(("quit", _)) => {
                op = Operation::Exit;
            }
            _ => {
                op = Operation::Unknown;
            },
        }
    
        Ok(Some(Commands::new(op, options)))
    }
    
    pub async fn run(&self) -> Result<()> {
        // Connect to C2 server.
        let server_url = format!(
            "ws://{}:{}/hermit",
            self.server_host.to_owned(),
            self.server_port.to_owned());
    
        let ws_stream = match connect_async(server_url.to_string()).await {
            Ok((stream, _response)) => {
                println!("{} Handshake has been completed.", "[+]".green().bold());
                stream
            }
            Err(e) => {
                println!("{} WebSocket handshake failed: {}", "[x]".red().bold(), e.to_string());
                return Ok(());
            }
        };
    
        println!(
            "{} Connected to C2 server ({}) successfully.",
            "[+]".green().bold(), server_url.to_string());
    
        let (mut sender, receiver) = ws_stream.split();
    
        // Client commands
        let mut rl = DefaultEditor::new()?;
        #[cfg(feature = "with-file-history")]
        if rl.load_history("history.txt").is_err() {
            println!("No previous history.");
        }
    
        let receiver = Arc::new(Mutex::new(receiver));
    
        loop {
            let readline = rl.readline(
                set_prompt(self.mode.to_string()).as_str());
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
    
                    let mut message = Message::Text("".to_owned());
    
                    if let Some(commands) = commands {
                        match &commands.op {
                            // Listener
                            Operation::AddListener => {
                                if let Some(listener_opt) = commands.options.listener_opt {
                                    message = Message::Text(format!("listener add {} {}://{}:{}/",
                                        listener_opt.name.unwrap(),
                                        listener_opt.proto.unwrap(),
                                        listener_opt.host.unwrap(),
                                        listener_opt.port.unwrap()));
                                } else {
                                    println!("Invalid command. Run `add help` for the usage.");
                                    continue;
                                }
                            }
                            Operation::DeleteListener => {
                                if let Some(listener_opt) = commands.options.listener_opt {
                                    if let Some(name) = listener_opt.name {
                                        message = Message::Text(format!("listener delete {}", name));
                                    } else {
                                        println!("Specify target listener by ID or name.");
                                    }
                                } else {
                                    continue;
                                }
                            }
                            Operation::StartListener => {
                                if let Some(listener_opt) = commands.options.listener_opt {
                                    if let Some(name) = listener_opt.name {
                                        message = Message::Text(format!("listener start {}", name));
                                    } else {
                                        println!("Specify target listener by ID or name.");
                                    }
                                } else {
                                    continue;
                                }
                            }
                            Operation::StopListener => {
                                if let Some(listener_opt) = commands.options.listener_opt {
                                    if let Some(name) = listener_opt.name {
                                        message = Message::Text(format!("listener stop {}", name));
                                    } else {
                                        println!("Specify target listener by ID or name.");
                                        continue;
                                    }
                                } else {
                                    continue;
                                }
                            }
                            Operation::ListListeners => {
                                message = Message::Text("listener list".to_string());
                            }
                            // Agent
                            Operation::UseAgent => {
                                if let Some(agent_opt) = commands.options.agent_opt {
                                    let name = agent_opt.name;
                                }
                                continue;
                            }
                            Operation::ListAgents => {
                                message = Message::Text("agent list".to_string());
                            }
                            // Implant
                            Operation::GenerateImplant => {
                                if let Some(implant_opt) = commands.options.implant_opt {
                                    let name = implant_opt.name;
                                    let listener_url = implant_opt.listener_url;
                                    let os = implant_opt.os;
                                    let arch = implant_opt.arch;
                                    let format = implant_opt.format;
                                    let sleep = implant_opt.sleep;

                                    message = Message::Text(
                                        format!("implant gen {} {} {} {} {} {}",
                                            name, listener_url, os, arch, format, sleep));
                                } else {
                                    continue;
                                }

                            }
                            Operation::DownloadImplant => {
                                println!("Download an implant.");
                                continue;
                            }
                            Operation::ListImplants => {
                                message = Message::Text("implant list".to_string());
                            }
                            // Misc
                            Operation::Empty => {
                                continue;
                            }
                            Operation::Exit => {
                                // message = Message::Close(None);
                                process::exit(EXIT_SUCCESS);
                            }
                            Operation::Unknown => {
                                println!("Unknown command. Run `help` command for the usage.");
                                continue;
                            }
                        }
                    }
    
                    // Send command
                    // sender.send(Message::Text(line.to_owned())).await.expect("Can not send.");
                    sender.send(message.to_owned()).await.expect("Can not send.");

                    // Spinner while waiting for responses
                    let mut spin: Option<Spinner> = None;
                    if message.to_string().starts_with("implant gen") {
                        spin = Some(Spinner::new(
                            Spinners::Dots8,
                            "Generating an implant...".into()));
                    }
                    
                    // Receive responses
                    let mut receiver_lock = receiver.lock().unwrap();
                    let mut flag = String::new();
                    // let mut 
                    while let Some(Ok(msg)) = receiver_lock.next().await {
                        match msg {
                            Message::Text(text) => {
                                if text == "done" {
                                    break
                                }
                                println!("{text}");

                                // Set flag
                                flag = text;
                            }
                            Message::Binary(bytes) => {
                                // Parse flag
                                let args = match shellwords::split(&flag) {
                                    Ok(args) => { args }
                                    Err(err) => {
                                        eprintln!("Can't parse command line: {err}");
                                        vec!["".to_string()]
                                    }
                                };

                                match (args[0].as_str(), args[1].as_str()) {
                                    ("Implant",  "generated") => {
                                        let outfile = args[2].to_string();
                                        fs::write(outfile, &bytes).expect("Unable to write file");
                                    }
                                    _ => {}
                                }
                            }
                            Message::Close(c) => {
                                if let Some(cf) = c {
                                    println!(
                                        "Close with code {} and reason `{}`",
                                        cf.code, cf.reason
                                    );
                                } else {
                                    println!("Somehow got close message without CloseFrame");
                                }
                                process::exit(EXIT_SUCCESS);
                            }
                            Message::Frame(_) => {
                                unreachable!("This is never supposed to happen")
                            }
                            _ => { break }
                        }
                    }
    
                    if let Some(mut spin) = spin {
                        spin.stop();
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
                }
            }
        }
    
        #[cfg(feature = "with-file-history")]
        rl.save_history("history.txt");
    
        Ok(())
    }
}

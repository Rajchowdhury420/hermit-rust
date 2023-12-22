use clap::{Arg, ArgMatches, value_parser, Command};
use colored::Colorize;
use futures_util::{SinkExt, StreamExt};
use rustyline::{DefaultEditor, Result};
use rustyline::error::ReadlineError;
use std::fs;
use std::io::Write;
use std::process;
use std::sync::{Arc, Mutex};
use tokio_tungstenite::{
    connect_async,
    tungstenite::protocol::Message,
};

use super::options::{
    implant::ImplantOption,
    listener::ListenerOption,
    options::Options
};
use crate::utils::random::random_name;

const EXIT_SUCCESS: i32 = 0;
const EXIT_FAILURE: i32 = 0;

#[derive(Debug)]
pub enum Mode {
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
    ListAgents,
    
    // Implants
    GenerateImplant,
    ListImplants,

    Instruction,
}

#[derive(Debug)]
struct Commands {
    pub mode: Mode,
    pub options: Options
    // pub cmd: String,
}

impl Commands {
    fn new(mode: Mode, options: Options) -> Self {
        Self {
            mode,
            options,
            // cmd,
        }
    }
}

pub struct Client {
    server_host: String,
    server_port: u16,
}

impl Client {
    pub fn new(server_host: String, server_port: u16) -> Self {
        Self {
            server_host,
            server_port,
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
                        .help(format!("Host [default: {}]", self.server_host))
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
            .subcommand(Command::new("listeners")
                .about("List listeners.")
            )
            // Agents
            .subcommand(Command::new("agents")
                .about("List agents who are connected to listeners.")
            )
            // Implants
            .subcommand(Command::new("generate")
                .about("Generate an implant.")
                .arg(Arg::new("name")
                    .short('n')
                    .long("name")
                    .help("Set an implant name")
                    .value_parser(value_parser!(String))
                )
                .arg(Arg::new("listener")
                    .short('l')
                    .long("listener")
                    .help("Listener URL that an agent connect to")
                    .default_value("http://127.0.0.1:8000/")
                    .value_parser(value_parser!(String))
                )
                .arg(Arg::new("os")
                    .short('o')
                    .long("os")
                    .help("Target OS")
                    .default_value("windows")
                    .value_parser(value_parser!(String))
                )
                .arg(Arg::new("arch")
                    .short('a')
                    .long("arch")
                    .help("Target architecture")
                    .default_value("amd64")
                    .value_parser(value_parser!(String))
                )
                .arg(Arg::new("format")
                    .short('f')
                    .long("format")
                    .help("File format to be generated")
                    .default_value("exe")
                    .value_parser(value_parser!(String))
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
        let mut mode = Mode::Empty;
        let mut cmd = "".to_string();
        let mut options = Options::new();
    
        match matches.subcommand() {
            // Listeners
            Some(("add", submatches)) => {
                mode = Mode::AddListener;
                let name = match submatches.get_one::<String>("name") {
                    Some(n) => { Some(n.to_string()) },
                    None => { Some(random_name("listener".to_string())) }
                };
                let host = match submatches.get_one::<String>("host") {
                    Some(h) => { Some(h.to_string()) },
                    None => { Some(self.server_host.to_string()) }
                };
                let listener_option = ListenerOption {
                    name,
                    proto: submatches.get_one::<String>("protocol").cloned(),
                    host,
                    port: submatches.get_one::<u16>("port").cloned(),
                };
                options.listener_opt = Some(listener_option);
            }
            Some(("delete", submatches)) => {
                mode = Mode::DeleteListener;
                let target = match submatches.get_one::<String>("listener") {
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
            Some(("start", submatches)) => {
                mode = Mode::StartListener;
                let target = match submatches.get_one::<String>("listener") {
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
            Some(("stop", submatches)) => {
                mode = Mode::StopListener;
                let target = match submatches.get_one::<String>("listener") {
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
            Some(("listeners", _)) => {
                mode = Mode::ListListeners;
            }
            // Agents
            Some(("agents", _)) => {
                mode = Mode::ListAgents;
            }
            // Implants
            Some(("generate", submatches)) => {
                mode = Mode::GenerateImplant;
                let name = match submatches.get_one::<String>("name") {
                    Some(n) => { n.to_string() },
                    None => { random_name("implant".to_string()) }
                };
                let listener_url = match submatches.get_one::<String>("listener") {
                    Some(n) => { n.to_string() },
                    None => { "http://127.0.0.1:8000/".to_string() }
                };
                let os = match submatches.get_one::<String>("os") {
                    Some(n) => { n.to_string() },
                    None => { "windows".to_string() }
                };
                let arch = match submatches.get_one::<String>("arch") {
                    Some(n) => { n.to_string() },
                    None => { "amd64".to_string() }
                };
                let format = match submatches.get_one::<String>("format") {
                    Some(n) => { n.to_string() },
                    None => { "exe".to_string() }
                };

                options.implant_opt = Some(ImplantOption {
                    name,
                    listener_url,
                    os,
                    arch,
                    format,
                });
            }
            Some(("implants", _)) => {
                mode = Mode::ListImplants;
            }
            // Instructions
            Some(("download", _)) => {
                mode = Mode::Empty;
            }
            // Misc
            None => {
                mode = Mode::Empty;
            }
            Some(("exit", _)) | Some(("quit", _)) => {
                mode = Mode::Exit;
            }
            _ => {
                mode = Mode::Unknown;
            },
        }
    
        Ok(Some(Commands::new(mode, options)))
    }
    
    pub async fn run(&self) -> Result<()> {
        // Connect to C2 server.
        let server_url = format!(
            "ws://{}:{}/hermit",
            self.server_host.to_owned(),
            self.server_port.to_owned());
    
        let ws_stream = match connect_async(server_url.to_string()).await {
            Ok((stream, _response)) => {
                println!("{}", "[+] Handshake has been completed.".green().bold());
                // println!("[+] Server response: {response:?}");
                stream
            }
            Err(e) => {
                println!("{} {}", "[x] WebSocket handshake failed:".red().bold(), e.to_string().red().bold());
                return Ok(());
            }
        };
    
        println!(
            "{}",
            format!(
                "[+] Connected to C2 server ({}) successfully.",
                server_url.to_string().cyan().bold()
            ).green().bold());
    
        let (mut sender, mut receiver) = ws_stream.split();
    
        // Client commands
        let mut rl = DefaultEditor::new()?;
        #[cfg(feature = "with-file-history")]
        if rl.load_history("history.txt").is_err() {
            println!("No previous history.");
        }
    
        let receiver = Arc::new(Mutex::new(receiver));
    
        loop {
            let readline = rl.readline(Self::set_prompt().as_str());
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
                        match &commands.mode {
                            // Listeners
                            Mode::AddListener => {
                                if let Some(listener_opt) = commands.options.listener_opt {
                                    message = Message::Text(format!("add {} {}://{}:{}/",
                                        listener_opt.name.unwrap(),
                                        listener_opt.proto.unwrap(),
                                        listener_opt.host.unwrap(),
                                        listener_opt.port.unwrap()));
                                } else {
                                    println!("Invalid command. Run `add help` for the usage.");
                                    continue;
                                }
                            }
                            Mode::DeleteListener => {
                                if let Some(listener_opt) = commands.options.listener_opt {
                                    if let Some(name) = listener_opt.name {
                                        message = Message::Text(format!("delete {}", name));
                                    } else {
                                        println!("Specify target listener by ID or name.");
                                    }
                                } else {
                                    continue;
                                }
                            }
                            Mode::StartListener => {
                                if let Some(listener_opt) = commands.options.listener_opt {
                                    if let Some(name) = listener_opt.name {
                                        message = Message::Text(format!("start {}", name));
                                    } else {
                                        println!("Specify target listener by ID or name.");
                                    }
                                } else {
                                    continue;
                                }
                            }
                            Mode::StopListener => {
                                if let Some(listener_opt) = commands.options.listener_opt {
                                    if let Some(name) = listener_opt.name {
                                        message = Message::Text(format!("stop {}", name));
                                    } else {
                                        println!("Specify target listener by ID or name.");
                                    }
                                } else {
                                    continue;
                                }
                            }
                            Mode::ListListeners => {
                                message = Message::Text("listeners".to_string());
                            }
                            // Agents
                            Mode::ListAgents => {
                                message = Message::Text("agents".to_string());
                            }
                            // Implants
                            Mode::GenerateImplant => {
                                if let Some(implant_opt) = commands.options.implant_opt {
                                    let name = implant_opt.name;
                                    let listener_url = implant_opt.listener_url;
                                    let os = implant_opt.os;
                                    let arch = implant_opt.arch;
                                    let format = implant_opt.format;

                                    message = Message::Text(
                                        format!("generate {} {} {} {} {}",
                                            name, listener_url, os, arch, format));
                                } else {
                                    continue;
                                }

                            }
                            Mode::ListImplants => {
                                message = Message::Text("implants".to_string());
                            }
                            // Misc
                            Mode::Empty => {
                                continue;
                            }
                            Mode::Exit => {
                                // message = Message::Close(None);
                                process::exit(EXIT_SUCCESS);
                            }
                            _ => {
                                println!("Unknown command. Run `help` command for the usage.");
                                continue;
                            }
                        }
                    }
    
                    // Send command
                    // sender.send(Message::Text(line.to_owned())).await.expect("Can not send.");
                    sender.send(message).await.expect("Can not send.");
                    
                    // Receive responses
                    let mut receiver_lock = receiver.lock().unwrap();
                    let mut recv_state = String::new();
                    // let mut 
                    while let Some(Ok(msg)) = receiver_lock.next().await {
                        match msg {
                            Message::Text(text) => {
                                if text == "done" {
                                    break
                                }
                                println!("{text}");

                                // Set state
                                recv_state = text;
                            }
                            Message::Binary(bytes) => {
                                // Parse the state
                                let args = match shellwords::split(&recv_state) {
                                    Ok(args) => { args }
                                    Err(err) => {
                                        eprintln!("Can't parse command line: {err}");
                                        vec!["".to_string()]
                                    }
                                };

                                match args[0].as_str() {
                                    "generated" => {
                                        let outfile = args[1].to_string();
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
    
    fn set_prompt() -> String {
        let name = "Hermit";
        format!("{} > ", name.bright_cyan())
    }
}

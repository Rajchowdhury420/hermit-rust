use clap::{Arg, ArgMatches, value_parser, Command};
use colored::Colorize;
use futures_util::{SinkExt, StreamExt};
use rustyline::{DefaultEditor, Result};
use rustyline::error::ReadlineError;
use std::process;
use std::sync::{Arc, Mutex};
use tokio_tungstenite::{
    connect_async,
    tungstenite::protocol::Message,
};

use crate::utils::random::random_name;

const EXIT_SUCCESS: i32 = 0;
const EXIT_FAILURE: i32 = 0;

#[derive(Debug)]
pub enum Mode {
    Empty,
    Exit,
    Unknown,

    AddListener,
    DeleteListener,
    StartListener,
    StopListener,
    ListListeners,
    
    GeneratePayload,
    ListPayloads,

    Instruction,
}

#[derive(Debug)]
struct ListenerOption {
    pub id: Option<u32>,
    pub name: Option<String>,
    pub proto: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
}

#[derive(Debug)]
struct Options {
    pub listener_opt: Option<ListenerOption>,
}

impl Options {
    pub fn new() -> Self {
        Self {
            listener_opt: None,
        }
    }
}

#[derive(Debug)]
struct Commands {
    pub mode: Mode,
    pub cmd: String,
    pub options: Options
}

impl Commands {
    fn new(mode: Mode, cmd: String, options: Options) -> Self {
        Self {
            mode,
            cmd,
            options,
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
                // .subcommand(Command::new("http")
                //     .about("Add a new HTTP listener.")
                // )
                .args([
                    Arg::new("name")
                        .short('n')
                        .long("name")
                        .help("Specify the name of a listener")
                        .value_parser(value_parser!(String)),
                    Arg::new("protocol")
                        .long("proto")
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
                        .value_parser(value_parser!(u16))
                ])
            )
            .subcommand(Command::new("delete")
                .about("Delete a listener.")
                .arg(Arg::new("id")
                    .short('i')
                    .long("id")
                    .help("Listener ID")
                    .required(true)
                    .value_parser(value_parser!(u32))
                )
            )
            .subcommand(Command::new("start")
                .about("Start a listener.")
                .arg(Arg::new("id")
                    .short('i')
                    .long("id")
                    .required(true)
                    .value_parser(value_parser!(u32))
                )
            )
            .subcommand(Command::new("stop")
                .about("Stop a listener.")
                .arg(Arg::new("id")
                    .short('i')
                    .long("id")
                    .help("Listener ID")
                    .required(true)
                    .value_parser(value_parser!(u32))
                )
            )
            .subcommand(Command::new("listeners")
                .about("List listeners.")
            )
            // Implants
            .subcommand(Command::new("generate")
                .about("Generate implants.")
            )
            .subcommand(Command::new("implants")
                .about("List implants.")
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
                    id: None,
                    name,
                    proto: submatches.get_one::<String>("protocol").cloned(),
                    host,
                    port: submatches.get_one::<u16>("port").cloned(),
                };
                options.listener_opt = Some(listener_option);
            }
            Some(("start", submatches)) => {
                mode = Mode::StartListener;
                let listener_id = submatches.get_one::<u32>("id")
                    .map(|s| s.to_owned()).unwrap_or(0);
                let listener_option = ListenerOption {
                    id: Some(listener_id),
                    name: None,
                    proto: None,
                    host: None,
                    port: None,
                };
                options.listener_opt = Some(listener_option);
            }
            Some(("stop", submatches)) => {
                mode = Mode::StopListener;
                let listener_id = submatches.get_one::<u32>("id")
                    .map(|s| s.to_owned()).unwrap_or(0);
                let listener_option = ListenerOption {
                    id: Some(listener_id),
                    name: None,
                    proto: None,
                    host: None,
                    port: None,
                };
                options.listener_opt = Some(listener_option);
            }
            Some(("listeners", _)) => {
                mode = Mode::ListListeners;
            }
            // Payloads
            Some(("payloads", _)) => {
                mode = Mode::ListPayloads;
            }
            // Instructions
            Some(("download", _)) => {
    
            }
            // Misc
            None => {
                mode = Mode::Empty;
            }
            Some(("exit", _)) => {
                mode = Mode::Exit;
            }
            _ => {
                mode = Mode::Unknown;
            },
        }
    
        Ok(Some(Commands::new(mode, cmd, options)))
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
                        Err(err) => { eprintln!("Can't parse command line: {err}"); vec!["".to_string()] }
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
                                    message = Message::Text(format!("delete {}", listener_opt.id.unwrap()));
                                }
                            }
                            Mode::StartListener => {
                                if let Some(listener_opt) = commands.options.listener_opt {
                                    message = Message::Text(format!("start {}", listener_opt.id.unwrap()));
                                }
                            }
                            Mode::StopListener => {
                                println!("Stop a listener.");
                                continue;
                            }
                            Mode::ListListeners => {
                                message = Message::Text("listeners".to_string());
                            }
                            // Payloads
                            Mode::GeneratePayload => {
                                message = Message::Text("generate".to_string());
                            }
                            Mode::ListPayloads => {
                                message = Message::Text("payloads".to_string());
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
                    while let Some(Ok(msg)) = receiver_lock.next().await {
                        match msg {
                            Message::Text(text) => {
                                println!("{text}");
                            }
                            Message::Binary(d) => {
                                println!("Got {} bytes: {:?}", d.len(), d);
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
                            _ => {}
                        }
                        break
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

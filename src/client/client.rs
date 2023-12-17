use clap::{Arg, ArgMatches, ArgAction, value_parser, Command};
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

const SERVER: &str = "ws://127.0.0.1:9999/hermit";

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
}

#[derive(Debug)]
struct ListenerOption {
    pub id: Option<u32>,
    pub channel: Option<String>,
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

pub struct Client {}

impl Client {
    pub fn new() -> Self {
        Self {}
    }

    fn cli() -> Command {
        Command::new("client")
            .about("Hermit C2 client")
            // .subcommand_required(true)
            // .arg_required_else_help(true)
            .allow_external_subcommands(true)
            .subcommand(Command::new("exit")
                .about("Close the connection and exit.")
            )
            // Listeners
            .subcommand(Command::new("add")
                .about("Add a new listener.")
                .subcommand(Command::new("http")
                    .about("Add a new HTTP listener.")
                )
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
            // Payloads
            .subcommand(Command::new("generate")
                .about("Generate a payload.")
            )
            .subcommand(Command::new("payloads")
                .about("List payloads (implants).")
            )
    }

    fn parse_args(args: &[String]) -> clap::error::Result<Option<Commands>> {
        let matches = Self::cli().try_get_matches_from(args)?;
        Self::parse_matches(&matches)
    }

    fn parse_matches(matches: &ArgMatches) -> clap::error::Result<Option<Commands>> {
        let mut mode = Mode::Empty;
        let mut cmd = "".to_string();
        let mut options = Options::new();

        match matches.subcommand() {
            
            // Listeners
            Some(("add", submatches)) => {
                mode = Mode::AddListener;
                match submatches.subcommand() {
                    Some(("http", _submatches_2)) => {
                        let listener_option = ListenerOption {
                            id: None,
                            channel: Some("http".to_string()),
                            host: Some("127.0.0.1".to_string()),
                            port: Some(8888),
                        };
                        options.listener_opt = Some(listener_option);
                    }
                    _ => {}
                }
            }
            Some(("start", submatches)) => {
                mode = Mode::StartListener;
                let listener_id = submatches.get_one::<u32>("id")
                    .map(|s| s.to_owned()).unwrap_or(0);
                let listener_option = ListenerOption {
                    id: Some(listener_id),
                    channel: None,
                    host: None,
                    port: None,
                };
            }
            Some(("stop", submatches)) => {
                mode = Mode::StopListener;
                let listener_id = submatches.get_one::<u32>("id")
                    .map(|s| s.to_owned()).unwrap_or(0);
                let listener_option = ListenerOption {
                    id: Some(listener_id),
                    channel: None,
                    host: None,
                    port: None,
                };
            }
            Some(("listeners", _submatches)) => {
                mode = Mode::ListListeners;
            }
            // Payloads
            Some(("payloads", _submatches)) => {
                mode = Mode::ListPayloads;
            }
            // Misc
            Some(("exit", _submatches)) => {
                mode = Mode::Exit;
            }
            _ => {},
        }

        Ok(Some(Commands::new(mode, cmd, options)))
    }

    pub async fn run(&self) -> Result<()> {
        // Connect to C2 server.
        let ws_stream = match connect_async(SERVER).await {
            Ok((stream, response)) => {
                println!("[i] Handshake has been completed.");
                println!("[i] Server response: {response:?}");
                stream
            }
            Err(e) => {
                println!("[x] {}", "WebSocket handshake failed.".bright_red());
                return Ok(());
            }
        };
    
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
                    let commands = match Self::parse_args(&args) {
                        Ok(commands) => commands,
                        Err(err) => {
                            println!("{}", err);
                            continue;
                        }
                    };

                    let mut message = Message::Text("".to_owned());

                    if let Some(commands) = commands {
                        match &commands.mode {
                            Mode::AddListener => {
                                if let Some(listener_opt) = commands.options.listener_opt {
                                    message = Message::Text(format!("add {}://{}:{}/",
                                        listener_opt.channel.unwrap(),
                                        listener_opt.host.unwrap(),
                                        listener_opt.port.unwrap()));
                                } else {
                                    println!("Listener option not found.");
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
                            Mode::GeneratePayload => {
                                message = Message::Text("generate".to_string());
                            }
                            Mode::ListPayloads => {
                                message = Message::Text("payloads".to_string());
                            }
                            Mode::Empty => {
                                continue;
                            }
                            Mode::Exit => {
                                message = Message::Close(None);
                            }
                            _ => {
                                println!("Unknown command.");
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
use clap::{ArgMatches, Command};
use colored::Colorize;
use rustyline::{
    Cmd,
    Completer,
    completion::FilenameCompleter,
    CompletionType,
    DefaultEditor,
    EditMode,
    Editor,
    error::ReadlineError,
    Helper,
    highlight::{Highlighter, MatchingBracketHighlighter},
    hint::HistoryHinter,
    Hinter,
    KeyEvent,
    Result,
    validate::MatchingBracketValidator,
    Validator,
};
use std::{
    borrow::Cow::{self, Borrowed, Owned},
    process
};

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
            handle_operator_delete,
            handle_operator_info,
            handle_operator_list,
        }
    },
    operations::{AgentOperation, Operation, RootOperation, set_operation},
    prompt::set_prompt,
};
use crate::server::grpc::pb_hermitrpc::hermit_rpc_client::HermitRpcClient;

const EXIT_SUCCESS: i32 = 0;
// const EXIT_FAILURE: i32 = 0;

#[derive(Helper, Completer, Hinter, Validator)]
pub struct RustylineHelper {
    #[rustyline(Completer)]
    completer: FilenameCompleter,
    highlighter: MatchingBracketHighlighter,
    #[rustyline(Validator)]
    validator: MatchingBracketValidator,
    pub colored_prompt: String,
}

impl Highlighter for RustylineHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        if default {
            Borrowed(&self.colored_prompt)
        } else {
            Borrowed(prompt)
        }
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned("\x1b[1m".to_owned() + hint + "\x1b[m")
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize, forced: bool) -> bool {
        self.highlighter.highlight_char(line, pos, forced)
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
    
    fn parse_args(&self, args: &[String]) -> clap::error::Result<Operation> {
        let matches = self.cli().try_get_matches_from(args)?;
        self.parse_matches(&matches)
    }
    
    fn parse_matches(&self, matches: &ArgMatches) -> clap::error::Result<Operation> {
        Ok(set_operation(self, matches))
    }
    
    pub async fn run(&mut self) -> Result<()> {
        let server_addr: tonic::transport::Uri = format!(
            "http://{}:{}",
            self.server_host,
            self.server_port
        ).parse().unwrap();
        let mut client = match HermitRpcClient::connect(server_addr.clone()).await {
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
    
        // Initialize the rustyline
        let rl_config = rustyline::config::Config::builder()
            .history_ignore_space(true)
            .completion_type(CompletionType::List)
            .edit_mode(EditMode::Emacs)
            .build();
        let rl_helper = RustylineHelper {
            completer: FilenameCompleter::new(),
            highlighter: MatchingBracketHighlighter::new(),
            colored_prompt: "".to_owned(),
            validator: MatchingBracketValidator::new(),
        };
        let mut rl = Editor::with_config(rl_config)?;
        rl.set_helper(Some(rl_helper));
        rl.bind_sequence(KeyEvent::alt('h'), Cmd::HistorySearchForward);
        rl.bind_sequence(KeyEvent::alt('p'), Cmd::HistorySearchBackward);
        if rl.load_history("history.txt").is_err() {
            println!("No previous history.");
        }
        
        loop {
            println!(""); // Add newline above the prompt for good appearance.
            let p = set_prompt(&mut rl, &self.mode);
            let readline = rl.readline(&p);
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
                    let op = match self.parse_args(&args) {
                        Ok(o) => o,
                        Err(err) => {
                            println!("{}", err);
                            continue;
                        }
                    };

                    match op {
                        Operation::Root(RootOperation::OperatorInfo { name}) => {
                            let _ = handle_operator_info(&mut client, name).await;
                        }
                        Operation::Root(RootOperation::OperatorList) => {
                            let _ = handle_operator_list(&mut client).await;
                        }
                        Operation::Root(RootOperation::ListenerAdd {
                            name,
                            domains,
                            proto,
                            host,
                            port,
                        }) => {
                            let _ = handle_listener_add(
                                &mut client,
                                name,
                                domains.join(","),
                                proto,
                                host,
                                port,
                            ).await;
                        }
                        Operation::Root(RootOperation::ListenerDelete { name }) => {
                            let _ = handle_listener_delete(&mut client, name).await;
                        }
                        Operation::Root(RootOperation::ListenerStart { name }) => {
                            let _ = handle_listener_start(&mut client, name).await;
                        }
                        Operation::Root(RootOperation::ListenerStop { name }) => {
                            let _ = handle_listener_stop(&mut client, name).await;
                        }
                        Operation::Root(RootOperation::ListenerInfo { name }) => {
                            let _ = handle_listener_info(&mut client, name).await;
                        }
                        Operation::Root(RootOperation::ListenerList) => {
                            let _ = handle_listener_list(&mut client).await;
                        }
                        Operation::Root(RootOperation::AgentUse { name }) => {
                            match handle_agent_use(&mut client, name.to_string()).await {
                                Ok(result) => {
                                    let res_split: Vec<String> = result
                                        .split(",")
                                        .map(|s| s.to_string()).collect();
                                    let agent_name = res_split[0].to_string();
                                    let agent_os = res_split[1].to_string();
                                    self.mode = Mode::Agent(agent_name, agent_os);
                                    println!("{} Switch to the agent mode.", "[+]".green());
                                }
                                Err(e) => {
                                    println!("{} Error switching to the agent mode: {:?}", "[x]".red(), e);
                                }
                            }
                        }
                        Operation::Root(RootOperation::AgentDelete { name }) => {
                            let _ = handle_agent_delete(&mut client, name.to_string()).await;
                        }
                        Operation::Root(RootOperation::AgentInfo { name }) => {
                            let _ = handle_agent_info(&mut client, name.to_string()).await;
                        }
                        Operation::Root(RootOperation::AgentList) => {
                            let _ = handle_agent_list(&mut client).await;
                        }
                        Operation::Root(RootOperation::ImplantGenerate {
                            name,
                            url,
                            os,
                            arch,
                            format,
                            sleep,
                            jitter,
                            user_agent,
                            killdate,
                        }) => {
                            let _ = handle_implant_generate(
                                &mut client, name, url, os, arch, format, sleep, jitter, user_agent, killdate
                            ).await;
                        }
                        Operation::Root(RootOperation::ImplantDownload { name }) => {
                            let _ = handle_implant_download(&mut client, name).await;
                        }
                        Operation::Root(RootOperation::ImplantDelete { name }) => {
                            let _ = handle_implant_delete(&mut client, name).await;
                        }
                        Operation::Root(RootOperation::ImplantInfo { name }) => {
                            let _ = handle_implant_info(&mut client, name).await;
                        }
                        Operation::Root(RootOperation::ImplantList) => {
                            let _ = handle_implant_list(&mut client).await;
                        }
                        Operation::Root(RootOperation::Exit) => {
                            let _ = handle_operator_delete(
                                &mut client,
                                self.operator_name.to_string()
                            ).await;
                            break;
                            // process::exit(EXIT_SUCCESS);
                        }
                        Operation::Agent(AgentOperation::Task { agent, task, args }) => {
                            let _ = handle_agent_task(
                                &mut client,
                                agent,
                                task,
                                args,
                            ).await;
                        }
                        Operation::Agent(AgentOperation::Exit) => {
                            println!("{} Exit the agent mode.", "[+]".green());
                            self.mode = Mode::Root;
                            continue;
                        }
                        Operation::Empty => {
                            continue;
                        }
                        Operation::Error { message } => {
                            println!("Error: {}", message);
                            continue;
                        }
                        Operation::Unknown => {
                            println!("{} Unknown command. Run `help` for the usage.", "[!]".yellow());
                            continue;
                        }
                    }
                },
                Err(ReadlineError::Interrupted) => {
                    println!("Interrupted");
                    let _ = handle_operator_delete(
                        &mut client,
                        self.operator_name.to_string()
                    ).await;
                    break
                },
                Err(ReadlineError::Eof) => {
                    let _ = handle_operator_delete(
                        &mut client,
                        self.operator_name.to_string()
                    ).await;
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

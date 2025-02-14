use clap::{Arg, Command, value_parser, ArgAction};

use crate::HermitClient;
use crate::client::client::Mode;

pub fn create_cmd(client: &HermitClient) -> Command {
    let cmd = Command::new("client")
        .about("Hermit C2 client")
        .allow_external_subcommands(true);

    match client.mode {
        Mode::Root => {
            cmd
                .subcommand(Command::new("exit")
                    .about("Close the connection and exit.")
                )
                // Operators
                .subcommand(Command::new("operator")
                    .about("Manage operators.")
                    .subcommand(Command::new("info")
                        .about("Print the detailed information of specified operator.")
                        .arg(Arg::new("operator")
                            .help("Operator ID or name.")
                            .required(true)
                            .value_parser(value_parser!(String)))
                    )
                    .subcommand(Command::new("list")
                        .about("List operators.")
                    )
                )
                .subcommand(Command::new("operators")
                    .about("List operators.")
                )
                .subcommand(Command::new("whoami")
                    .about("Print your information as operator.")
                )
                // Listeners
                .subcommand(Command::new("listener")
                    .about("Manage listeners.")
                    .subcommand(Command::new("add")
                        .about("Add a new listener.")
                        .args([
                            Arg::new("protocol")
                                .help("Protocol")
                                .default_value("https")
                                .value_parser(value_parser!(String)),
                            Arg::new("host")
                                .short('H')
                                .long("host")
                                .help("A listener host")
                                .default_value("0.0.0.0")
                                .value_parser(value_parser!(String)),
                            Arg::new("port")
                                .short('P')
                                .long("port")
                                .help("A listener port")
                                .default_value("4443")
                                .value_parser(value_parser!(u16)),
                            Arg::new("name")
                                .short('n')
                                .long("name")
                                .help("The name of a listener")
                                .value_parser(value_parser!(String)),
                            Arg::new("domains")
                                .short('d')
                                .long("domains")
                                .help("Domains for the HTTPS certificates")
                                .default_value("localhost")
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
                    .subcommand(Command::new("info")
                        .about("Print the detailed information of a specified listener.")
                        .arg(Arg::new("listener")
                            .help("Listener ID or name to print the information")
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
                        .about("Interact with the specified agent.")
                        .arg(Arg::new("name")
                            .help("Agent ID or name")
                            .required(true)
                            .value_parser(value_parser!(String)))
                    )
                    .subcommand(Command::new("delete")
                        .about("Delete the specified agent.")
                        .arg(Arg::new("name")
                            .help("Agent ID or name")
                            .required(true)
                            .value_parser(value_parser!(String))
                        )
                    )
                    .subcommand(Command::new("info")
                        .about("Print the detail information of a specified agent. (under development)")
                        .arg(Arg::new("name")
                            .help("Agent ID or name")
                            .required(true)
                            .value_parser(value_parser!(String))
                        )
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
                            Arg::new("url")
                                .short('u')
                                .long("url")
                                .help("Listener URL that an agent connect to")
                                .default_value("https://127.0.0.1:4443/")
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
                            Arg::new("jitter")
                                .short('j')
                                .long("jitter")
                                .help("Jitter")
                                .default_value("5")
                                .value_parser(value_parser!(u64)),
                            Arg::new("user-agent")
                                .long("user-agent")
                                .help("Custom User-Agent")
                                .value_parser(value_parser!(String)),
                            Arg::new("killdate")
                                .short('k')
                                .long("killdate")
                                .help("Specify the kill date (format: 'yyyy-mm-dd hh:mm:ss', example: '2024-01-01 00:00:00') to stop processing when a specified date is reached. The kill date is not specified by default.")
                                .default_value("0")
                                .value_parser(value_parser!(String)),
                        ])
                    )
                    .subcommand(Command::new("download")
                        .about("Download specified implant.")
                        .arg(Arg::new("name")
                            .help("Implant ID or name.")
                            .required(true)
                            .value_parser(value_parser!(String)))
                    )
                    .subcommand(Command::new("delete")
                        .about("Delete an implant from the list.")
                        .arg(Arg::new("name")
                            .help("Implant ID or name.")
                            .required(true)
                            .value_parser(value_parser!(String)))
                    )
                    .subcommand(Command::new("info")
                        .about("Print the detailed information of a specified implant.")
                        .arg(Arg::new("name")
                            .help("Implant ID or name.")
                            .required(true)
                            .value_parser(value_parser!(String)))
                    )
                    .subcommand(Command::new("list")
                        .about("List implants generated.")
                    )
                )
                .subcommand(Command::new("implants")
                    .about("List implants generated.")
                )
        }
        Mode::Agent(_, _) => {
            cmd
                .subcommand(Command::new("exit")
                    .about("Exit the agent mode.")
                )
                // Tasks
                .subcommand(Command::new("cat")
                    .about("Read a file content.")
                    .arg(Arg::new("file")
                        .help("Specified file to read the content.")
                        .required(true)
                        .value_parser(value_parser!(String)))
                )
                .subcommand(Command::new("cd")
                    .about("Change the current directory.")
                    .arg(Arg::new("directory")
                        .help("Specified directory that you want to set as the current directory.")
                        .required(true)
                        .value_parser(value_parser!(String)))
                )
                .subcommand(Command::new("cp")
                    .about("Copy file to another location.")
                    .args([
                        Arg::new("source")
                            .help("Source")
                            .required(true)
                            .value_parser(value_parser!(String)),
                        Arg::new("dest")
                            .help("Destination")
                            .required(true)
                            .value_parser(value_parser!(String)),
                    ])
                )
                .subcommand(Command::new("download")
                    .about("Download a file from target computer.")
                    .arg(Arg::new("file")
                        .help("Specified file to download.")
                        .required(true)
                        .value_parser(value_parser!(String)))
                )
                .subcommand(Command::new("info")
                    .about("Get the detailed information of the system.")
                )
                .subcommand(Command::new("ls")
                    .about("List files in a specified directory.")
                    .args([
                        Arg::new("directory")
                            .help("Specified directory")
                            .default_value(".")
                            .value_parser(value_parser!(String))
                    ])
                )
                .subcommand(Command::new("loot")
                    .about("Show the looted information for the agent that you've obtained so far. (Under development)")
                )
                .subcommand(Command::new("mkdir")
                    .about("Create a new directory.")
                    .arg(Arg::new("directory")
                            .help("Specified directory to create")
                            .required(true)
                            .value_parser(value_parser!(String)))
                )
                .subcommand(Command::new("net")
                    .about("Get the network information.")
                )
                .subcommand(Command::new("ps")
                    .about("Process management.")
                    .subcommand(Command::new("kill")
                        .about("Kill the process.")
                        .arg(Arg::new("pid")
                            .help("Process ID")
                            .required(true)
                            .value_parser(value_parser!(u32)))
                    )
                    .subcommand(Command::new("list")
                        .about("List the running processes.")
                        .args([
                            Arg::new("filter")
                                .help("Filter by specified string")
                                .short('f')
                                .long("filter")
                                .value_parser(value_parser!(String)),
                            Arg::new("exclude")
                                .help("Exclude by specified string")
                                .short('x')
                                .long("exclude")
                                .value_parser(value_parser!(String)),
                        ])
                    )
                )
                .subcommand(Command::new("pwd")
                    .about("Get the current directory.")
                )
                .subcommand(Command::new("rm")
                    .about("Remove file or directory.")
                    .args([
                        Arg::new("recursive")
                            .help("Recursively remove a directory.")
                            .short('r')
                            .long("recursive")
                            .action(ArgAction::SetTrue),
                        Arg::new("file")
                            .help("Specified file or directory to remove.")
                            .value_parser(value_parser!(String))
                    ])
                )
                .subcommand(Command::new("screenshot")
                    .about("Take a screenshot for target machine.")
                )
                .subcommand(Command::new("shell")
                    .about("Execute shell command for target machine.")
                    .args([
                        Arg::new("cmd")
                            .help("Use Command Prompt.")
                            .long("cmd")
                            .action(ArgAction::SetTrue),
                        Arg::new("ps")
                            .help("Use PowerShell.")
                            .long("ps")
                            .action(ArgAction::SetTrue),
                        Arg::new("command")
                            .help("Specified command.")
                            .required(true)
                            .value_parser(value_parser!(String)),
                    ])
                )
                .subcommand(Command::new("shellcode")
                    // Shellcode generation example with Msfvenom:
                    //   `msfvenom -p windows/x64/exec CMD="calc.exe" -f hex -o shellcode.txt`
                    .about("Shellcode injection")
                    .args([
                        Arg::new("pid")
                            .long("pid")
                            .help("Specified process ID which is currently running to inject shellcode.")
                            .value_parser(value_parser!(u32)),
                        Arg::new("process")
                            .short('p')
                            .long("process")
                            .help("Create a new process with specified process (e.g. 'svchost') and inject shellcode to it.")
                            .value_parser(value_parser!(String)),
                        Arg::new("file")
                            .short('f')
                            .long("file")
                            .help("Path to shellcode file e.g. 'shellcode.txt'.")
                            .required(true)
                            .value_parser(value_parser!(String))
                    ])
                )
                .subcommand(Command::new("sleep")
                    .about("Change sleep time for sending requests to C2 server.")
                    .arg(Arg::new("time")
                            .help("Specified time seconds.")
                            .required(true)
                            .value_parser(value_parser!(u64)))
                )
                .subcommand(Command::new("upload")
                    .about("Upload a file to target computer.")
                    .args([
                        Arg::new("file")
                            .help("File path to upload.")
                            .required(true)
                            .value_parser(value_parser!(String)),
                        Arg::new("dest")
                            .help("Destination path")
                            .default_value(".")
                            .value_parser(value_parser!(String)),
                    ])
                )
                .subcommand(Command::new("whoami")
                    .about("Get current username.")
                )
        }
    }
}
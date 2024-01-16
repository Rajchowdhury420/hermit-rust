use clap::{Parser, Subcommand};
use env_logger::Env;
use log::warn;

pub mod banner;
pub mod client;
pub mod config;
pub mod server;
pub mod utils;

use crate::{
    banner::banner,
    client::client::Client,
    config::Config,
    server::server::run as run_server,
    utils::{fs::mkdir, random::random_name},
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// C2 client
    Client {
        /// Host to connect to C2 server
        #[arg(short = 'H', long)]
        host: String,

        /// Port to connect to C2 server
        #[arg(short = 'P', long)]
        port: u16,

        /// Operator name
        #[arg(short = 'n', long, default_value_t = random_name("operator".to_string()))]
        name: String,
    },

    /// C2 server
    Server {
        /// Port for C2 server
        #[arg(short = 'P', long, default_value_t = 9999)]
        port: u16,
    },
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let mut config = Config::new();

    // Get app directory.
    match home::home_dir() {
        Some(path) if !path.as_os_str().is_empty() => {
            config.app_dir = format!("{}/.hermit", path.display()).into();
        },
        _ => warn!("Unable to get your home dir. "),
    }

    let cli = Cli::parse();

    mkdir("agents".to_owned()).unwrap();
    mkdir("implants".to_owned()).unwrap();
    mkdir("tmp".to_owned()).unwrap();

    match &cli.command {
        Some(Commands::Client { host, port, name }) => {
            mkdir("client".to_string()).unwrap();
            
            banner("client");
            let _ = Client::new(
                host.to_owned(),
                port.to_owned(),
                name.to_owned()
            ).run().await;
        },
        Some(Commands::Server { port }) => {
            mkdir("server".to_string()).unwrap();

            banner("server");
            let _ = run_server(config, *port).await;
        },
        _ => {
            println!("Not enough argument. Run `hermit help` for the usage.")
        },
    }
}

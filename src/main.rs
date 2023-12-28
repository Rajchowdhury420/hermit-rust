use clap::{Parser, Subcommand};
use env_logger::Env;
use log::warn;

pub mod banner;
pub mod client;
pub mod config;
pub mod implants;
pub mod server;
pub mod utils;

use crate::banner::banner;
use crate::client::client::Client;
use crate::config::Config;
use crate::server::server::run as run_server;

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
    },

    /// C2 server
    Server {},
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

    config.mkdir("agents".to_owned()).unwrap();
    config.mkdir("implants".to_owned()).unwrap();
    config.mkdir("tmp".to_owned()).unwrap();

    match &cli.command {
        Some(Commands::Client { host, port }) => {
            config.mkdir("client".to_string()).unwrap();

            banner("client");
            let _ = Client::new(host.to_owned(), port.to_owned()).run().await;
        },
        Some(Commands::Server {}) => {
            config.mkdir("server".to_string()).unwrap();

            banner("server");
            let _ = run_server(config).await;
        },
        _ => {
            println!("Not enough argument. Run `hermit help` for the usage.")
        },
    }
}

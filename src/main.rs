use clap::{Parser, Subcommand};
use env_logger::Env;
use log::info;

pub mod client;
pub mod server;
pub mod utils;

use crate::client::client::Client;
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

    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Client { host, port }) => {
            println!("Starting C2 client...");
            let _ = Client::new(host.to_owned(), port.to_owned()).run().await;
        },
        Some(Commands::Server {}) => {
            info!("Starting C2 server...");
            let _ = run_server().await;
        },
        _ => {},
    }
}

use clap::{Parser, Subcommand};
use env_logger::Env;
use log::info;

pub mod client;
pub mod server;

use crate::client::client::Client;
use crate::server::server::Server;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// C2 client
    Client {},

    /// C2 server
    Server {},
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Client {}) => {
            println!("Starting C2 client...");
            let _ = Client::new().run().await;
        },
        Some(Commands::Server {}) => {
            info!("Starting C2 server...");
            let _ = Server::new().run().await;
        },
        _ => {},
    }
}

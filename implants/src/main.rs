pub mod agents;
pub mod core;
pub mod config;
pub mod handlers;
pub mod systeminfo;

use core::run;
use config::config::Config;

include!(concat!(env!("OUT_DIR"), "/init.rs"));

#[tokio::main]
async fn main() {
    let (proto, host, port, sleep) = init();
    let config = Config::new(proto.to_string(), host.to_string(), port, sleep);
    run(config).await.unwrap()
}

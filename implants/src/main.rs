pub mod core;
pub mod config;
pub mod utils;

#[cfg(target_os = "linux")]
use core::run_linux::run;
#[cfg(target_os = "windows")]
use core::run_windows::run;

use config::config::Config;

include!(concat!(env!("OUT_DIR"), "/init.rs"));

#[tokio::main]
async fn main() {
    let (proto, host, port, sleep) = init();
    let config = Config::new(proto.to_string(), host.to_string(), port, sleep);
    run(config).await.unwrap()
}

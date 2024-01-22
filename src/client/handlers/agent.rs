use colored::Colorize;
use spinners::{Spinner, Spinners};
use std::io::{Error, ErrorKind};
use tonic::transport::Channel;

use crate::{
    server::grpc::{
        pb_agenttasks,
        pb_common,
        pb_hermitrpc::{hermit_rpc_client::HermitRpcClient, hermit_rpc_server::HermitRpc},
        pb_operations,
    },
    utils::fs::{get_app_dir, write_file},
};
use super::common::print_response;

pub async fn handle_agent_use(
    client: &mut HermitRpcClient<Channel>,
    agent: String, // Agent ID or name
) -> Result<String, Error> {

    let request = tonic::Request::new(pb_operations::Target {
        id_or_name: agent,
    });
    let response = match client.use_agent(request).await {
        Ok(r) => r,
        Err(e) => {
            println!("\n{} Cannot switch the agent mode: {}", "[x]".red(), e.message());
            return Err(Error::new(ErrorKind::Other, e.to_string()));
        }
    };
    // print_response(response);

    let result = response.into_inner();
    if result.success {
        return Ok(result.message); // Return the agent name.
    } else {
        println!("\n{} Cannot switch to the agent mode.", "[x]".red());
        return Err(Error::new(ErrorKind::Other, "Cannot switch to the agent mode."));
    }
}

pub async fn handle_agent_delete(
    client: &mut HermitRpcClient<Channel>,
    agent: String, // Agent ID or name
) -> Result<(), Error> {

    let request = tonic::Request::new(pb_operations::Target {
        id_or_name: agent,
    });
    let response = match client.delete_agent(request).await {
        Ok(r) => r,
        Err(e) => {
            println!("\n{} The agent cannot be deleted: {}", "[x]".red(), e.message());
            return Ok(());
        }
    };
    print_response(response);

    Ok(())
}

pub async fn handle_agent_info(
    client: &mut HermitRpcClient<Channel>,
    agent: String, // Agent ID or name
) -> Result<(), Error> {

    let request = tonic::Request::new(pb_operations::Target {
        id_or_name: agent,
    });
    let response = match client.info_agent(request).await {
        Ok(r) => r,
        Err(e) => {
            println!("\n{} Cannot display the agent info: {}", "[x]".red(), e.message());
            return Ok(());
        }
    };
    print_response(response);

    Ok(())
}

pub async fn handle_agent_list(
    client: &mut HermitRpcClient<Channel>,
) -> Result<(), Error> {
    
    let request = tonic::Request::new(pb_common::Empty {});
    let response = match client.list_agents(request).await {
        Ok(r) => r,
        Err(e) => {
            println!("\n{} Cannot list agents: {}", "[x]".red(), e.message());
            return Ok(());
        }
    };
    print_response(response);

    Ok(())
}

pub async fn handle_agent_task(
    client: &mut HermitRpcClient<Channel>,
    agent: String,      // Agent name
    task: String,       // Task name
    args: String,       // Arguments
) -> Result<(), Error> {

    let mut spin = Spinner::new(
        Spinners::Dots8,
        "Executing the task...".to_string(),
    );

    let request = tonic::Request::new(pb_agenttasks::Task {
        agent,
        task,
        args,
    });
    let response = match client.agent_task(request).await {
        Ok(r) => r,
        Err(e) => {
            spin.stop();
            println!("\n{} {}", "[x]".red(), e.message());
            return Ok(());
        }
    };

    spin.stop();
    println!(""); // Insert newline for better appearance.

    let mut read_data: Vec<u8> = Vec::new();
    let mut resp_stream = response.into_inner();
    let mut outfile = String::new();
    while let Some(res) = resp_stream.message().await.unwrap() {
        read_data.extend(res.data);

        if res.message != "" && outfile == "" {
            outfile = res.message;
        }
    }

    if outfile != "" {
        write_file(outfile.to_string(), &read_data).unwrap();
        println!(
            "{} File saved at {}",
            "[+]".green(),
            format!("{}/{}", get_app_dir(), outfile.to_string()).cyan());
    } else {
        println!("{} {}", "[+]".green(), String::from_utf8(read_data).unwrap());
    }

    Ok(())
}
use colored::Colorize;
use spinners::{Spinner, Spinners};
use std::io::Error;
use tonic::transport::Channel;

use crate::{
    server::grpc::{
        self,
        pb_common,
        pb_hermitrpc::{hermit_rpc_client::HermitRpcClient, hermit_rpc_server::HermitRpc},
        pb_operations,
    },
    utils::fs::{get_app_dir, write_file},
};
use super::common::print_response;

pub async fn handle_implant_generate(
    client: &mut HermitRpcClient<Channel>,
    name: String,
    url: String,
    os: String,
    arch: String,
    format: String,
    sleep: u64,
    jitter: u64,
    user_agent: String,
    killdate: String,
) -> Result<(), Error> {

    let mut spin = Spinner::new(
        Spinners::Dots8,
        "Generating implant...".to_string(),
    );

    let request = tonic::Request::new(pb_operations::NewImplant {
        name: name.to_string(),
        url: url.to_string(),
        os: os.to_string(),
        arch: arch.to_string(),
        format: format.to_string(),
        sleep: sleep as i64,
        jitter: jitter as i64,
        user_agent: user_agent.to_string(),
        killdate: killdate.to_string(),
    });
    let response = match client.generate_implant(request).await {
        Ok(r) => r,
        Err(e) => {
            spin.stop();
            println!("\n{} {}", "[x]".red(), e.message());
            return Ok(());
        }
    };

    // Collect the implant data
    let mut implant_data: Vec<u8> = Vec::new();
    let mut resp_stream = response.into_inner();
    let mut outfile = String::new();
    while let Some(res) = resp_stream.message().await.unwrap() {
        implant_data.extend(res.data);

        if outfile == "" {
            outfile = res.message;
        }
    }

    // Save the implant
    let outfile_names: Vec<String> = outfile.split("/").map(|s| s.to_string()).collect();
    let outfile = format!(
        "implants/{}/{}",
        outfile_names[outfile_names.len() - 4],
        outfile_names.last().unwrap(),
    );
    write_file(outfile.to_string(), &implant_data).unwrap();

    spin.stop();

    println!(
        "{} Implant generated at {}",
        "[+]".green(),
        format!("{}/{}", get_app_dir(), outfile.to_string()).cyan());
    println!(
        "{} Transfer this file to target machine and execute it to interact with our C2 server.",
        "[i]".green());

    Ok(())
}

pub async fn handle_implant_download(
    client: &mut HermitRpcClient<Channel>,
    implant: String, // Implant ID or name
) -> Result<(), Error> {

    let mut spin = Spinner::new(
        Spinners::Dots8,
        "Generating implant...".to_string(),
    );

    let request = tonic::Request::new(pb_operations::Target {
        id_or_name: implant,
    });
    let response = match client.download_implant(request).await {
        Ok(r) => r,
        Err(e) => {
            spin.stop();
            println!("\n{} {}", "[x]".red(), e.message());
            return Ok(());
        }
    };

    // Collect the implant data
    let mut implant_data: Vec<u8> = Vec::new();
    let mut resp_stream = response.into_inner();
    let mut outfile = String::new();
    while let Some(res) = resp_stream.message().await.unwrap() {
        implant_data.extend(res.data);

        if outfile == "" {
            outfile = res.message;
        }
    }

    // Save the implant
    let outfile_names: Vec<String> = outfile.split("/").map(|s| s.to_string()).collect();
    let outfile = format!(
        "implants/{}/{}",
        outfile_names[outfile_names.len() - 4],
        outfile_names.last().unwrap()
    );
    write_file(outfile.to_string(), &implant_data).unwrap();

    spin.stop();

    println!(
        "{} Implant generated at {}",
        "[+]".green(),
        format!("{}/{}", get_app_dir(), outfile.to_string()).cyan());
    println!(
        "{} Transfer this file to target machine and execute it to interact with our C2 server.",
        "[i]".green());

    Ok(())
}

pub async fn handle_implant_delete(
    client: &mut HermitRpcClient<Channel>,
    implant: String,
) -> Result<(), Error> {

    let request = tonic::Request::new(pb_operations::Target {
        id_or_name: implant,
    });
    let response = match client.delete_implant(request).await {
        Ok(r) => r,
        Err(e) => {
            println!("\n{} Implant cannot be deleted: {}", "[x]".red(), e.message());
            return Ok(());
        }
    };
    print_response(response);

    Ok(())
}

pub async fn handle_implant_info(
    client: &mut HermitRpcClient<Channel>,
    implant: String,
) -> Result<(), Error> {
    let request = tonic::Request::new(pb_operations::Target {
        id_or_name: implant,
    });
    let response = match client.info_implant(request).await {
        Ok(r) => r,
        Err(e) => {
            println!("\n{} Implant info not found: {:?}", "[x]".red(), e.message());
            return Ok(());
        }
    };
    print_response(response);

    Ok(())
}

pub async fn handle_implant_list(
    client: &mut HermitRpcClient<Channel>,
) -> Result<(), Error> {
    let request = tonic::Request::new(pb_common::Empty {});
    let response = match client.list_implants(request).await {
        Ok(r) => r,
        Err(e) => {
            println!("\n{} Implant info not found: {}", "[x]".red(), e.message());
            return Ok(());
        }
    };
    print_response(response);

    Ok(())
}
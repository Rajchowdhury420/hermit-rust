use colored::Colorize;
use std::io::Error;
use tonic::transport::Channel;

use crate::server::{
    grpc::{
        self,
        pb_common,
        pb_hermitrpc::{hermit_rpc_client::HermitRpcClient, hermit_rpc_server::HermitRpc},
        pb_operations,
    },
    listeners::listener::Listener,
};
use super::common::print_response;

pub async fn handle_listener_add(
    client: &mut HermitRpcClient<Channel>,
    name: String,       // Listener name
    domains: String,    // Listener domains
    protocol: String,   // Listener protocol
    host: String,       // Listener host
    port: u16,          // Listener port
) -> Result<(), Error> {

    let request = tonic::Request::new(pb_operations::NewListener {
        name,
        domains,
        protocol,
        host,
        port: port.to_string(),
    });
    let response = match client.add_listener(request).await {
        Ok(r) => r,
        Err(e) => {
            println!("{} The listener cannot be added: {:?}", "[x]".red(), e);
            return Ok(());
        }
    };
    print_response(response);
    Ok(())
}

pub async fn handle_listener_delete(
    client: &mut HermitRpcClient<Channel>,
    listener: String, // Listener ID or name
) -> Result<(), Error> {

    let request = tonic::Request::new(pb_operations::Target {
        id_or_name: listener,
    });
    let response = match client.delete_listener(request).await {
        Ok(r) => r,
        Err(e) => {
            println!("{} The listener cannot be deleted: {:?}", "[x]".red(), e);
            return Ok(());
        }
    };
    print_response(response);
    Ok(())
}

pub async fn handle_listener_start(
    client: &mut HermitRpcClient<Channel>,
    listener: String, // Listener ID or name
) -> Result<(), Error> {

    let request = tonic::Request::new(pb_operations::Target {
        id_or_name: listener,
    });
    let response = match client.start_listener(request).await {
        Ok(r) => r,
        Err(e) => {
            println!("{} The listener cannot be started: {:?}", "[x]".red(), e);
            return Ok(());
        }
    };
    print_response(response);
    Ok(())
}

pub async fn handle_listener_stop(
    client: &mut HermitRpcClient<Channel>,
    listener: String, // Listener ID or name
) -> Result<(), Error> {

    let request = tonic::Request::new(pb_operations::Target {
        id_or_name: listener,
    });
    let response = match client.stop_listener(request).await {
        Ok(r) => r,
        Err(e) => {
            println!("{} The listener cannot be stopped: {:?}", "[x]".red(), e);
            return Ok(());
        }
    };
    print_response(response);
    Ok(())
}

pub async fn handle_listener_info(
    client: &mut HermitRpcClient<Channel>,
    listener: String, // Listener ID or name
) -> Result<(), Error> {

    let request = tonic::Request::new(pb_operations::Target {
        id_or_name: listener,
    });
    let response = match client.info_listener(request).await {
        Ok(r) => r,
        Err(e) => {
            println!("{} Could not display the listener info: {:?}", "[x]".red(), e);
            return Ok(());
        }
    };
    print_response(response);
    Ok(())
}

pub async fn handle_listener_list(
    client: &mut HermitRpcClient<Channel>,
) -> Result<(), Error> {

    let request = tonic::Request::new(pb_common::Empty {});
    let response = match client.list_listeners(request).await {
        Ok(r) => r,
        Err(e) => {
            println!("{} List listeners failed: {:?}", "[x]".red(), e);
            return Ok(());
        }
    };
    print_response(response);
    Ok(())
}
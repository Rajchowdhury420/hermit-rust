use colored::Colorize;
use std::io::Error;
use tonic::transport::Channel;

use crate::server::grpc::{
    self,
    pb_agenttasks,
    pb_common,
    pb_hermitrpc::{hermit_rpc_client::HermitRpcClient, hermit_rpc_server::HermitRpc},
    pb_operations,
};
use super::common::print_response;

pub async fn handle_operator_add(
    client: &mut HermitRpcClient<Channel>,
    operator_name: String
) -> Result<(), Error> {

    let request = tonic::Request::new(pb_operations::NewOperator {
        name: operator_name.to_string(),
    });
    let response = match client.add_operator(request).await {
        Ok(r) => r,
        Err(e) => {
            println!("{} The operator cannot be added: {:?}", "[x]".red(), e);
            return Ok(());
        }
    };
    print_response(response);

    Ok(())
}

pub async fn handle_operator_info(
    client: &mut HermitRpcClient<Channel>,
    operator: String,
) -> Result<(), Error> {

    let request = tonic::Request::new(pb_operations::Target {
        id_or_name: operator,
    });
    let response = match client.info_operator(request).await {
        Ok(o) => o,
        Err(e) => {
            println!("{} Could not get the operator info: {:?}", "[x]".red(), e);
            return Ok(());
        }
    };
    print_response(response);

    Ok(())
}

pub async fn handle_operator_list(
    client: &mut HermitRpcClient<Channel>,
) -> Result<(), Error> {

    let request = tonic::Request::new(pb_common::Empty {});
    let response = match client.list_operators(request).await {
        Ok(o) => o,
        Err(e) => {
            println!("{} Could not list operators: {:?}", "[x]".red(), e);
            return Ok(());
        }
    };
    print_response(response);

    Ok(())
}
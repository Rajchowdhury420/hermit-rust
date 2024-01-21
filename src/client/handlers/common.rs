use colored::Colorize;
use tonic::{Response, Status};

use crate::server::grpc::pb_common::Result;

pub fn print_response(response: Response<Result>) {
    let response = response.get_ref();
    if response.success {
        println!("{} {}", "[+]".green(), response.message);
    } else {
        println!("{} {}", "[x]".red(), response.message);
    }
}

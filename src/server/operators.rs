use log::info;

use crate::utils::str::truncated_format;

#[derive(Clone, Debug)]
pub struct Operator {
    pub id: u32,
    pub name: String,
    pub address: String,
}

impl Operator {
    pub fn new(id: u32, name: String, address: String) -> Self {
        Self {
            id,
            name,
            address,
        }
    }
}

pub fn format_operator_details(operator: Operator) -> String {
    info!("Getting operator details...");

    let mut output = String::new();
    output = output + "\n";
    output = output + format!("{:<10} : {:<20}\n", "ID", operator.id).as_str();
    output = output + format!("{:<10} : {:<20}\n", "NAME", operator.name).as_str();
    output = output + format!("{:<10} : {:<20}\n", "FORMAT", operator.address).as_str();
    output
}

pub fn format_all_operators(operators: &Vec<Operator>) -> String {
    info!("Getting operators information...");
    if operators.len() == 0 {
        return String::new();
    }

    let mut output = String::new();
    output = output + "\n";
    output = output + format!(
        "{:>5} | {:<20} | {:<20}\n",
        "ID", "NAME", "ADDRESS",
    ).as_str();
    let output_len = output.len();
    output = output + "-".repeat(output_len).as_str() + "\n";

    for operator in operators {
        output = output + format!(
            "{:>5} | {:<20} | {:<20}\n",
            operator.id.to_owned(),
            truncated_format(operator.name.to_owned(), 17),
            truncated_format(operator.address.to_owned(), 17),
        ).as_str();
    }

    return output;
}

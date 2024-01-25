use log::info;
use pad::Alignment;

use crate::utils::str::{
    table_format,
    TableItem,
};

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

    let mut output = String::from("\n\n");
    output = output + format!("{:<8} : {}\n", "ID", operator.id).as_str();
    output = output + format!("{:<8} : {}\n", "NAME", operator.name).as_str();
    output = output + format!("{:<8} : {}", "FORMAT", operator.address).as_str();
    output
}

pub fn format_all_operators(operators: &Vec<Operator>) -> String {
    info!("Getting operators information...");
    if operators.len() == 0 {
        return String::new();
    }

    let columns = vec![
        TableItem::new("ID".to_string(), 3, Alignment::Right, None),
        TableItem::new("NAME".to_string(), 14, Alignment::Left, None),
        TableItem::new("ADDRESS".to_string(), 16, Alignment::Left, None),
    ];
    let mut rows: Vec<Vec<TableItem>> = Vec::new();
    for operator in operators {
        let row = vec![
            TableItem::new(operator.id.to_string(), 3, Alignment::Right, None),
            TableItem::new(operator.name.to_string(), 14, Alignment::Left, None),
            TableItem::new(operator.address.to_string(), 16, Alignment::Left, None),
        ];
        rows.push(row);
    }
    table_format(columns, rows)
}

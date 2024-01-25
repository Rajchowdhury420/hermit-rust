use colored::Colorize;
use pad::{Alignment, PadStr};

pub fn truncated_format(txt: String, n: usize) -> String {
    let mut truncated_txt = txt.to_string();

    if txt.len() < 3 {
        return txt;
    }

    truncated_txt.truncate(n);

    if truncated_txt.len() >= n {
        truncated_txt + "..."
    } else {
        truncated_txt
    }
}

pub struct TableItem {
    item: String,
    padding: usize,
    align: Alignment,
    color: Option<String>,
}

impl TableItem {
    pub fn new(item: String, padding: usize, align: Alignment, color: Option<String>) -> Self {
        Self { item, padding, align, color }
    }
}

pub fn table_format(columns: Vec<TableItem>, rows: Vec<Vec<TableItem>>) -> String {
    let mut header = String::from("| ");
    for col in columns {
        header = header + format!(
            " {} |",
            col.item.pad_to_width_with_alignment(col.padding, col.align),
        ).as_str();
    }

    let width = header.len() - 2;

    let border = "+".to_string() + "-".repeat(width).as_str() + "+";

    let mut body = String::new();

    for row in rows {
        let mut line = String::from("| ");
        for r in row {
            let truncated = truncated_format(r.item, r.padding - 3)
                .pad_to_width_with_alignment(r.padding, r.align);
            let colored_txt = if let Some(color) = r.color {
                match color.as_str() {
                    "green" => truncated.green(),
                    "red" => truncated.red(),
                    _ => truncated.normal(),
                }
            } else {
                truncated.normal()
            };
            line = line + format!(" {} |", colored_txt).as_str();
        }
        line = line + "\n";
        body = body + line.as_str();
    }

    "\n".to_string() +
    border.as_str() + "\n" +
    header.as_str() + "\n" +
    border.as_str() + "\n" +
    body.as_str() +
    border.as_str()
}
pub fn truncated_format(txt: String, n: usize) -> String {
    let mut truncated_txt = txt.to_string();
    truncated_txt.truncate(n);

    if truncated_txt.len() >= n {
        truncated_txt + "..."
    } else {
        truncated_txt
    }
}
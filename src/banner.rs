const LOGO: &'static str = "
        ┓┏┏┓┳┓┳┳┓┳┏┳┓
        ┣┫┣ ┣┫┃┃┃┃ ┃ 
        ┛┗┗┛┛┗┛ ┗┻ ┻ ";

pub fn banner(mode: &str) {
    let mut output = String::from("\n");
    output = output + format!("{}", LOGO).as_str();
    output = output + "\n";
    output = output + match mode {
        "server" => "           C2 SERVER\n",
        "client" => "           C2 CLIENT\n",
        _ => "",
    };
    output = output + "      +++++++++++++++++\n";
    output = output + "      DEVELOPED BY HDKS\n";
    output = output + "\n";

    println!("\x1b[38;5;101m{}\x1b[0m", output);
}
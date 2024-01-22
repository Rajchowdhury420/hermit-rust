use colored::{Colorize, CustomColor};

use super::client::Mode;

pub fn set_prompt(mode: &Mode) -> String {
    let name = "Hermit";
    let mark = "$";
    let color_gray = CustomColor::new(150, 150, 150);

    match mode {
        Mode::Root => {
            return format!("{} {} ", name.custom_color(color_gray), mark.red());
        }
        Mode::Agent(agent_name, _) => {
            return format!("{} [{}] {} ", name.custom_color(color_gray), agent_name.cyan(), mark.red());
        }
    }
}
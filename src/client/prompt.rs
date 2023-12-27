use colored::{Colorize, CustomColor};

pub fn set_prompt(mode: String) -> String {
    let name = "Hermit";
    let mark = "$";
    let color_gray = CustomColor::new(150, 150, 150);

    if mode == "" {
        format!("{} {} ", name.custom_color(color_gray), mark.red())
    } else {
        format!("{} [{}] {} ", name.custom_color(color_gray), mode.cyan(), mark.red())
    }
}
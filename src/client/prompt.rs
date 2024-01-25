use rustyline::{
    Editor,
    history::FileHistory
};

use super::client::{
    Mode,
    RustylineHelper,
};

pub fn set_prompt(rl: &mut Editor<RustylineHelper, FileHistory>, mode: &Mode) -> String {
    let p_hermit = "\x1b[38;5;101mHermit\x1b[0m";
    let p_dollar = "\x1b[38;5;88m$\x1b[0m";

    match mode {
        Mode::Root => {
            rl.helper_mut().expect("No helper").colored_prompt = format!(
                "{} {} ",
                p_hermit, p_dollar,
            );
            format!("Hermit $ ")
        },
        Mode::Agent(agent_name, _) => {
            let p_agent = format!("[\x1b[38;5;24m{agent_name}\x1b[0m]");
            rl.helper_mut().expect("No helper").colored_prompt = format!(
                "{} {} {} ",
                p_hermit, p_agent, p_dollar,
            );
            format!("Hermit [{}] $ ", agent_name)
        }
    }
}
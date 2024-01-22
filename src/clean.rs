use rustyline::{DefaultEditor, Result};
use std::fs::remove_dir_all;

use crate::utils::fs::get_app_dir;

pub fn clean() -> Result<()> {
    let mut rl = DefaultEditor::new()?;
    
    let line = rl.readline("Do you really want to clean all saved data? [y/N]: ")?;
    if line.to_lowercase() == "y" {
        // Delete the app directory
        let app_dir = get_app_dir();

        match remove_dir_all(app_dir) {
            Ok(_) => {
                println!("Cleaned data successfully.");
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
    }

    Ok(())
}
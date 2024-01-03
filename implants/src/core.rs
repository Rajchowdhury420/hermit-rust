#[cfg(target_os = "linux")]
pub mod run_linux;
#[cfg(target_os = "windows")]
pub mod run_windows;

pub mod handlers;
pub mod postdata;
pub mod systeminfo;
pub mod tasks;
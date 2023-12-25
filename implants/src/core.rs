#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "linux")]
pub use self::linux::*;

// #[cfg(target_os = "windows")]
pub mod win;
// #[cfg(target_os = "windows")]
pub use self::win::*;
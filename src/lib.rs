// In rust we code
// In code we trust
// AgarthaSoftware - 2024

pub mod config;

pub mod commands;

#[cfg(target_os = "windows")]
pub const INSTANCE_PATH: &str = "%APPDATA%/local/wormhole";

#[cfg(target_os = "linux")]
pub const INSTANCE_PATH: &'static str = "/usr/local/share/wormhole/";

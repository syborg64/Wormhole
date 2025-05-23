// In rust we code
// In code we trust
// AgarthaSoftware - 2024
use std::fs;

use tokio::runtime::Runtime;

use crate::commands::cli_commands::{Cli, GetHostsArgs};

use super::cli_messager;

pub fn get_hosts(ip: &str, args: GetHostsArgs) -> Result<(), Box<dyn std::error::Error>> {
    let rt = Runtime::new().unwrap();
    rt.block_on(cli_messager(
        ip,
        Cli::GetHosts(args),
    ))
}

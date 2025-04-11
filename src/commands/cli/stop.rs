use tokio::runtime::Runtime;

use crate::commands::cli_commands::{Cli::{self}, StatusPodArgs};

use super::cli_messager;

pub fn stop(ip: &str, stop_args: StatusPodArgs) -> Result<(), Box<dyn std::error::Error>> {
    let rt = Runtime::new().unwrap();
    rt.block_on(cli_messager(ip, Cli::Stop(stop_args)))?;
    Ok(())
}

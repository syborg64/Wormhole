use std::env;

use tokio::runtime::Runtime;

use crate::{commands::cli_commands::{Cli::{self}, StatusPodArgs}, pods::whpath::WhPath};

use super::cli_messager;

pub fn stop(ip: &str, mut stop_args: StatusPodArgs) -> Result<(), Box<dyn std::error::Error>> {
    if stop_args.name == None && stop_args.path == None {
        let path = env::current_dir()?;
        stop_args.path = Some(WhPath::from(&path.display().to_string()));
    }

    let rt = Runtime::new().unwrap();
    rt.block_on(cli_messager(ip, Cli::Stop(stop_args)))?;
    Ok(())
}

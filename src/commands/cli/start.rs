use std::env;

use tokio::runtime::Runtime;

use crate::{
    commands::cli_commands::{
        Cli::{self},
        StatusPodArgs,
    },
    pods::whpath::WhPath,
};

use super::cli_messager;

pub fn start(ip: &str, mut start_args: StatusPodArgs) -> Result<(), Box<dyn std::error::Error>> {
    if start_args.name == None && start_args.path == None {
        let path = env::current_dir()?;
        start_args.path = Some(WhPath::from(&path.display().to_string()));
    }

    let rt = Runtime::new().unwrap();
    rt.block_on(cli_messager(ip, Cli::Start(start_args)))?;
    Ok(())
}

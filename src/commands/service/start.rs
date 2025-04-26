use tokio::sync::mpsc;

use crate::{
    commands::{cli_commands::StatusPodArgs, PodCommand},
    error::{CliResult, CliSuccess},
};

pub async fn start(start_args: StatusPodArgs) -> CliResult {
    let name = start_args.name.clone().unwrap_or("default".to_string());
    Ok(CliSuccess::WithData { message: String::from("Pod start: "), data: name })
}

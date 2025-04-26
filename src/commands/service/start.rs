use tokio::sync::mpsc;

use crate::{
    commands::{cli_commands::StatusPodArgs, PodCommand},
    error::{CliResult, CliSuccess},
};

pub async fn start(tx: mpsc::UnboundedSender<PodCommand>, start_args: StatusPodArgs) -> CliResult {
    let name = start_args.name.clone();

    tx.send(PodCommand::StartPod(start_args))
        .expect("Cli feedback channel is closed");
    Ok(CliSuccess::PodCreated {
        pod_id: name.unwrap_or("None".to_string()),
    })
}

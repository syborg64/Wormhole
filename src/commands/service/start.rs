use tokio::sync::mpsc;

use crate::{
    commands::{cli_commands::StatusPodArgs, PodCommand},
    error::{CliError, CliResult, CliSuccess},
};

pub async fn start(tx: mpsc::UnboundedSender<PodCommand>, start_args: StatusPodArgs) -> CliResult {
    let name = start_args.name.clone();
    match tx.send(PodCommand::StartPod(start_args)) {
        Ok(_) => Ok(CliSuccess::PodCreated {
            pod_id: name.unwrap_or("None".to_string()),
        }),
        Err(e) => Err(CliError::SendCommandFailed {
            reason: e.to_string(),
        }),
    }
}

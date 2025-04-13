use tokio::sync::mpsc;

use crate::{
    commands::{cli_commands::StatusPodArgs, PodCommand},
    error::{CliError, CliResult, CliSuccess},
};

pub async fn stop(tx: mpsc::UnboundedSender<PodCommand>, stop_args: StatusPodArgs) -> CliResult {
    let name = stop_args.name.clone();
    match tx.send(PodCommand::StopPod(stop_args)) {
        Ok(_) => Ok(CliSuccess::PodCreated {
            pod_id: name.unwrap_or("None".to_string()),
        }),
        Err(e) => Err(CliError::SendCommandFailed {
            reason: e.to_string(),
        }),
    }
}

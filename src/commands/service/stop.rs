use tokio::sync::mpsc;

use crate::commands::{cli_commands::StatusPodArgs, PodCommand};

pub async fn stop(
    tx: mpsc::UnboundedSender<PodCommand>,
    stop_args: StatusPodArgs,
) -> Result<String, String> {
    match tx.send(PodCommand::StopPod(stop_args)) {
        Ok(_) => Ok("Pod stopped successfully".to_string()),
        Err(e) => Err(format!("PodCommand send error: {}", e)),
    }
}

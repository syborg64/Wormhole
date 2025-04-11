use tokio::sync::mpsc;

use crate::commands::{cli_commands::StatusPodArgs, PodCommand};

pub async fn start(tx: mpsc::UnboundedSender<PodCommand>, start_args: StatusPodArgs) -> Result<String, String> {
    match tx.send(PodCommand::StartPod(start_args)) {
      Ok(_) => Ok("Pod started successfully".to_string()),
      Err(e) => Err(format!("PodCommand send error: {}", e)),
    }
}

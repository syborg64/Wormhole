use tokio::sync::{mpsc, oneshot::channel};

use crate::{
    commands::{cli_commands::StatusPodArgs, PodCommand},
    error::{CliError, CliSuccess},
    pods::pod::PodStopError,
};

pub async fn stop(
    tx: mpsc::UnboundedSender<PodCommand>,
    stop_args: StatusPodArgs,
) -> Result<CliSuccess, CliError> {
    let (result_tx, result_rx) = channel::<Result<String, PodStopError>>();

    tx.send(PodCommand::StopPod(stop_args, result_tx))
        .expect("Cli feedback channel is closed");

    // REVIEW -
    result_rx
        .await
        .expect("pod_stop: channel closed")
        .map_or_else(
            |e| {
                Err(CliError::PodRemovalFailed {
                    reason: e.to_string(),
                })
            },
            |a| Ok(CliSuccess::Message(a)),
        )
}

use tokio::sync::{mpsc, oneshot::channel};

use crate::{
    commands::{cli_commands::StatusPodArgs, PodCommand},
    pods::pod::PodStopError,
};

pub async fn stop(
    tx: mpsc::UnboundedSender<PodCommand>,
    stop_args: StatusPodArgs,
) -> Result<String, String> {
    let (result_tx, result_rx) = channel::<Result<String, PodStopError>>();

    if let Err(e) = tx.send(PodCommand::StopPod(stop_args, result_tx)) {
        log::error!("Cli: PodCommand tx is closed: {e}");
        return Err(format!("ERROR: Cli feedback channel is closed: {e}").to_string());
    }

    // NOTE - converts custom_error into `Result<String, String>` asked for the cli
    // should be updated when the cli is updated
    result_rx.await.map_or_else(
        |e| {
            log::error!("Cli feedback channel is closed: {e}");
            Err(format!("ERROR: Cli feedback channel is closed: {e}").to_string())
        },
        |a| a.map_err(|e| e.to_string()),
    )
}

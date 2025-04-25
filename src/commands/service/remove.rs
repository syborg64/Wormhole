use tokio::sync::mpsc;

use crate::{
    commands::{cli_commands::RemoveArgs, PodCommand},
    error::{CliResult, CliSuccess},
};

pub async fn remove(tx: mpsc::UnboundedSender<PodCommand>, args: RemoveArgs) -> CliResult {
    tx.send(PodCommand::RemovePod(args))
        .expect("Cli feedback channel is closed");
    Ok(CliSuccess::Message(
        "Pod delete successfully and cleaned".to_string(),
    ))
}

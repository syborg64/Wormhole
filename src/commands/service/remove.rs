use tokio::{runtime::Runtime, sync::mpsc};

use crate::{
    commands::{
        cli::cli_messager,
        cli_commands::{Mode, RemoveArgs},
        PodCommand,
    },
    error::{CliError, CliResult, CliSuccess},
};

pub async fn remove(tx: mpsc::UnboundedSender<PodCommand>, args: RemoveArgs) -> CliResult {
    match tx.send(PodCommand::RemovePod(args)) {
        Ok(_) => Ok(CliSuccess::Message(
            "Pod delete successfully and cleaned".to_string(),
        )),
        Err(e) => Err(CliError::SendCommandFailed {
            reason: e.to_string(),
        }),
    }
}

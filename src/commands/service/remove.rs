use tokio::{runtime::Runtime, sync::mpsc};

use crate::commands::{
    cli::cli_messager,
    cli_commands::{Mode, RemoveArgs},
    PodCommand,
};

pub async fn remove(
    tx: mpsc::UnboundedSender<PodCommand>,
    args: RemoveArgs,
) -> Result<String, String> {
    match args.mode {
        Mode::Simple => todo!(),
        Mode::Clone => todo!(),
        Mode::Take => todo!(),
        Mode::Clean => match tx.send(PodCommand::RemovePod(args)) {
            Ok(_) => Ok("Pod removed successfully".to_string()),
            Err(e) => Err(format!("PodCommand send error: {}", e)),
        },
    }
}

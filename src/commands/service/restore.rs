use crate::{commands::cli_commands::RestoreConf, error::{CliResult, CliSuccess}};

pub async fn restore(_args: RestoreConf) -> CliResult {
    Ok(CliSuccess::Message("bread".to_owned()))
}
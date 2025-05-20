use std::sync::Arc;

use crate::{commands::cli_commands::RestoreConf,
    config::{types::Config, GlobalConfig, LocalConfig},
    error::{CliError, CliResult, CliSuccess},
    pods::arbo::{GLOBAL_CONFIG_FNAME, LOCAL_CONFIG_FNAME}
};

pub async fn restore(local_config: Arc<LocalConfig>, global_config: Arc<GlobalConfig>, args: RestoreConf) -> CliResult {
    for file in args.files.clone() {
        match file.as_str() {
            LOCAL_CONFIG_FNAME => {
                if let Err(e) = local_config.write(args.path.join(LOCAL_CONFIG_FNAME).inner) {
                    return Err(CliError::BoxError { arg: e });
                }
            },
            GLOBAL_CONFIG_FNAME => {
                if let Err(e) = global_config.write(args.path.join(GLOBAL_CONFIG_FNAME).inner) {
                    return Err(CliError::BoxError { arg: e });
                }
            },
            _ => {return Err(CliError::InvalidArgument { arg: file });}
        }
    }
    Ok(CliSuccess::Message("bread".to_owned()))
}
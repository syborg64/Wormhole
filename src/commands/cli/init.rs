use std::fs;

use tokio::runtime::Runtime;

use crate::{
    commands::{
        cli::message::cli_messager,
        cli_commands::{Cli, PodArgs},
    },
    pods::whpath::WhPath,
};

fn check_config_file(
    path: &WhPath,
    files_name: Vec<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    for file_name in files_name {
        println!("file_name: {}", path.clone().push(file_name).inner);
        if fs::metadata(path.clone().push(file_name).inner.clone()).is_err() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("The file config {} does not exist", file_name),
            )));
        }
    }
    Ok(())
}

pub fn init(ip: &str, path: &WhPath) -> Result<(), Box<dyn std::error::Error>> {
    fs::read_dir(&path.inner).map(|_| ())?;
    let files_name = vec![".local_config.toml", ".global_config.toml"];
    check_config_file(&path, files_name)?;
    let rt = Runtime::new().unwrap();
    rt.block_on(cli_messager(
        ip,
        Cli::Init(PodArgs {
            path: Some(path.clone()),
        }),
    ))?;
    Ok(())
}

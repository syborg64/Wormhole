use std::{
    os::{fd::AsFd, unix::net::UnixStream},
    path::PathBuf,
};

use assert_fs::TempDir;
use std::process::Stdio;
use tokio::process::Command;

pub struct Service {
    pub instance: tokio::process::Child,
    #[allow(dead_code)]
    dir: TempDir,
    #[allow(dead_code)]
    stdin: UnixStream,
    pub path: PathBuf,
}

pub struct EnvironnementManager {
    pub services: Vec<Service>,
}

impl EnvironnementManager {
    pub fn new() -> Self {
        return EnvironnementManager {
            services: Vec::new(),
        };
    }

    fn generate_pipe(pipe_output: bool) -> Stdio {
        if pipe_output {
            Stdio::inherit()
        } else {
            Stdio::null()
        }
    }

    pub fn add_service(&mut self, pipe_output: bool) -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = assert_fs::TempDir::new()?;

        let new_path = temp_dir.path().to_string_lossy().to_string();
        let new_index = self.services.len();
        let snd_index = (new_index + 1) % 3;
        let third_index = (new_index + 2) % 3;

        let mut command = Command::new("cargo");

        // GHActions does'nt pipe a stdin so we have to make one ourselves from a FD
        let (write, read) = std::os::unix::net::UnixStream::pair()?;

        let stdio = Stdio::from(read.as_fd().try_clone_to_owned().unwrap());
        command.kill_on_drop(true);

        let instance = command
            .args(&[
                "run".to_string(),
                "--bin".to_string(),
                "wormhole-service".to_string(),
                new_path,
                format!("127.0.0.{}:8081", new_index + 100),
                format!("127.0.0.{}:8081", snd_index + 100),
                format!("127.0.0.{}:8081", third_index + 100),
            ])
            .stdout(Self::generate_pipe(pipe_output))
            .stderr(Self::generate_pipe(pipe_output))
            .stdin(stdio)
            .spawn()?;

        self.services.push(Service {
            instance,
            path: temp_dir.path().to_owned(),
            dir: temp_dir,
            stdin: write,
        });

        Ok(())
    }
}

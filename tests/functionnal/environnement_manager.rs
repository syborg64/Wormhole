use std::path::PathBuf;

use assert_fs::TempDir;
use tokio::process::Command;

pub struct EnvironnementManager {
    pub service_instances: Vec<tokio::process::Child>,
    temp_dirs: Vec<TempDir>,
    pub paths: Vec<PathBuf>,
}

impl EnvironnementManager {
    pub fn new() -> Self {
        return EnvironnementManager {
            service_instances: Vec::new(),
            temp_dirs: Vec::new(),
            paths: Vec::new(),
        };
    }

    pub fn add_service(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = assert_fs::TempDir::new()?;

        let new_path = temp_dir.path().to_string_lossy().to_string();
        let new_index = self.paths.len();
        let snd_index = (new_index + 1) % 3;
        let third_index = (new_index + 2) % 3;

        self.paths.push(temp_dir.path().to_owned());
        self.temp_dirs.push(temp_dir);
        let mut command = Command::new("cargo");
        command.kill_on_drop(true);
        self.service_instances.push(
            command
                .args(&[
                    "run".to_string(),
                    "--bin".to_string(),
                    "service".to_string(),
                    format!("127.0.0.{new_index}:8080"),
                    format!("ws://127.0.0.{snd_index}:8080"),
                    format!("ws://127.0.0.{third_index}:8080"),
                    new_path.clone(),
                    new_path,
                ])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()?,
        );
        return Ok(());
    }
}

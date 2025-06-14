pub mod environment_manager;
pub mod test_sync;
pub mod test_transfer;

use std::path::PathBuf;

pub use environment_manager::EnvironmentManager;

#[tokio::test]
async fn start_log() {
    env_logger::init();
}

fn append_to_path(p: &PathBuf, s: &str) -> PathBuf {
    let mut p = p.as_os_str().to_owned();
    p.push(s);
    p.into()
}

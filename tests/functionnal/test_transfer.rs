use crate::functionnal::{append_to_path, environnement_manager::SLEEP_TIME};

use super::environnement_manager;

pub use environnement_manager::EnvironmentManager;
use serial_test::serial;

#[serial]
#[tokio::test]
async fn basic_text_file_transfer() {
    let mut env = EnvironmentManager::new();
    env.add_service(false).unwrap();
    env.add_service(false).unwrap();
    env.add_service(false).unwrap();
    std::thread::sleep(*SLEEP_TIME);
    env.create_network("default".to_string(), false).unwrap();

    std::thread::sleep(*SLEEP_TIME);
    let file_path = append_to_path(&env.services[0].pods[0].2.path().to_owned(), "/foo.txt");
    std::fs::write(&file_path, "Hello world!").unwrap();
    std::thread::sleep(*SLEEP_TIME);

    for paths in [
        &env.services[0].pods[0].2.path().to_owned(),
        &env.services[1].pods[0].2.path().to_owned(),
        &env.services[2].pods[0].2.path().to_owned(),
    ] {
        match std::fs::read_to_string(append_to_path(paths, "/foo.txt")) {
            Ok(content) => assert!(content == "Hello world!", "File content is incorrect"),
            Err(_) => assert!(false, "File doesn't exist"),
        }
    }
    std::thread::sleep(*SLEEP_TIME);
}

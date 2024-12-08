use crate::functionnal::append_to_path;

use super::environnement_manager;

pub use environnement_manager::EnvironnementManager;
use serial_test::serial;

#[serial]
#[tokio::test]
async fn basic_text_file_transfer() {
    let mut env = EnvironnementManager::new();
    env.add_service(false).unwrap();
    env.add_service(false).unwrap();
    env.add_service(false).unwrap();

    std::thread::sleep(std::time::Duration::from_secs_f32(0.5));
    let file_path = append_to_path(&env.services[0].path, "/foo.txt");
    std::fs::write(&file_path, "Hello world!").unwrap();
    std::thread::sleep(std::time::Duration::from_secs_f32(0.3));

    for paths in [&env.services[1].path, &env.services[2].path] {
        match std::fs::read_to_string(append_to_path(paths, "/foo.txt")) {
            Err(_) => assert!(false, "File doesn't exist"),
            Ok(content) => assert!(content == "Hello world!", "File content is incorrect"),
        }
    }
}

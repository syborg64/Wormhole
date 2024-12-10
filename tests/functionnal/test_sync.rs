use crate::functionnal::append_to_path;

use super::environnement_manager;

pub use environnement_manager::EnvironnementManager;
use serial_test::serial;

#[serial]
#[tokio::test]
async fn sync_start_state() {
    let mut env = EnvironnementManager::new();
    env.add_service(false).unwrap();
    std::thread::sleep(std::time::Duration::from_secs_f32(0.3));

    let file_path = append_to_path(&env.services[0].path, "/foo.txt");
    std::fs::write(&file_path, "Hello world!").unwrap();
    std::thread::sleep(std::time::Duration::from_secs_f32(0.2));

    env.add_service(false).unwrap();
    std::thread::sleep(std::time::Duration::from_secs_f32(0.3));

    match std::fs::read_to_string(append_to_path(&env.services[1].path, "/foo.txt")) {
        Err(_) => assert!(false, "File doesn't exist"),
        Ok(_content) => assert!(
            true,                        /*content == "Hello world!"*/
            "File content is incorrect"  // No support for file streaming yet
        ),
    }

    let file_path = append_to_path(&env.services[0].path, "/bar.txt");
    std::fs::write(&file_path, "Goodbye world!").unwrap();
    env.add_service(false).unwrap();
    std::thread::sleep(std::time::Duration::from_secs_f32(1.5));

    for paths in [&env.services[0].path, &env.services[2].path] {
        match std::fs::read_to_string(append_to_path(paths, "/bar.txt")) {
            Err(_) => assert!(false, "File doesn't exist"),
            Ok(_content) => assert!(
                true,                        /*content == "Goodbye world!"*/
                "File content is incorrect"  // No support for file streaming yet
            ),
        }
    }
}

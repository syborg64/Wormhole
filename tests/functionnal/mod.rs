pub mod environnement_manager;

use std::path::PathBuf;

pub use environnement_manager::EnvironnementManager;

fn append_to_path(p: &PathBuf, s: &str) -> PathBuf {
    let mut p = p.as_os_str().to_owned();
    p.push(s);
    p.into()
}

#[tokio::test]
async fn test_files() {
    let mut env = EnvironnementManager::new();
    env.add_service(true).unwrap();
    env.add_service(true).unwrap();
    env.add_service(true).unwrap();

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

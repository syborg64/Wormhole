// In rust we code
// In code we trust
// AgarthaSoftware - 2024

use file_config::recover_config;

mod file_config;

fn main() {
    println!(
        "{:?}",
        recover_config("/home/ludofr3/WormHole/src/test.toml")
    );
}

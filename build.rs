use std::process::Command;

fn main() {
    #[cfg(target_os = "windows")]
    {
        winfsp::build::winfsp_link_delayload();
    }
    let git_hash: String = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output().ok().and_then(|output| output.stdout.try_into().ok()).unwrap_or("none".into());
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
}

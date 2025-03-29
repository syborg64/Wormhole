use std::fs;
pub fn init(path: &std::path::PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    fs::read_dir(path).map(|_| ())?;
    Ok(())
}

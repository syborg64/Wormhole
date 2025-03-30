use std::fs;

use crate::pods::whpath::WhPath;
pub fn init(path: &WhPath) -> Result<(), Box<dyn std::error::Error>> {
    fs::read_dir(path.inner.clone()).map(|_| ())?;
    Ok(())
}

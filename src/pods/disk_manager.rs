use std::{fs::File, path::PathBuf};

use openat::Dir;
use tokio::io;

pub struct DiskManager {
    handle: Dir,
    mount_point: PathBuf, // mountpoint on linux and mirror mountpoint on windows
}


/// always takes a WhPath and infers the real disk path
impl DiskManager {
    pub fn new_file(&self, path: &PathBuf) -> io::Result<File> {
        let path = self.mount_point.join(path);
        self.handle.new_file(&path, 0o644) // TODO look more in c mode_t value
    }

    pub fn remove_file(&self, path: &PathBuf) -> io::Result<()> {
        let path = self.mount_point.join(path);
        self.handle.remove_file(&path) // TODO look more in c mode_t value
    }

    pub fn new_dir(&self, path: &PathBuf) -> io::Result<()> {
        let path = self.mount_point.join(path);
        self.handle.create_dir(&path, 0o644) // TODO look more in c mode_t value
    }
}
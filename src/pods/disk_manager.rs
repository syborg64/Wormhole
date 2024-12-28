use std::fs::File;

use openat::Dir;
use tokio::io;

use super::whpath::WhPath;

pub struct DiskManager {
    handle: Dir,
    mount_point: WhPath, // mountpoint on linux and mirror mountpoint on windows
}

/// always takes a WhPath and infers the real disk path
impl DiskManager {
    pub fn new(mount_point: WhPath) -> io::Result<Self> {
        Ok(Self {
            handle: Dir::open(mount_point.clone())?,
            mount_point,
        })
    }

    pub fn new_file(&self, path: &WhPath) -> io::Result<File> {
        let path: WhPath = self.mount_point.join(path);
        self.handle.new_file(path, 0o644) // TODO look more in c mode_t value
    }

    pub fn remove_file(&self, path: &WhPath) -> io::Result<()> {
        let path = self.mount_point.join(path);
        self.handle.remove_file(path) // TODO look more in c mode_t value
    }

    pub fn new_dir(&self, path: &WhPath) -> io::Result<()> {
        let path = self.mount_point.join(path);
        self.handle.create_dir(path, 0o644) // TODO look more in c mode_t value
    }
}

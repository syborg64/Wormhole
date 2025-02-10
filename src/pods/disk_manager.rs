use std::{ffi::OsStr, fs::File, os::unix::fs::FileExt};

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

    pub fn new_file(&self, path: WhPath) -> io::Result<File> {
        self.handle.new_file(path.set_relative(), 0o644) // TODO look more in c mode_t value
    }

    pub fn remove_file(&self, path: WhPath) -> io::Result<()> {
        self.handle.remove_file(path.set_relative())
    }

    pub fn remove_dir(&self, path: WhPath) -> io::Result<()> {
        self.handle.remove_dir(path.set_relative())
    }

    pub fn write_file(&self, path: WhPath, binary: Vec<u8>, offset: u64) -> io::Result<u64> {
        let file = self.handle.write_file(path.set_relative(), 0o600)?;
        Ok(file.write_at(&binary, offset)? as u64) // NOTE - used "as" because into() is not supported
    }

    pub fn mv_file(&self, path: WhPath, new_path: WhPath) -> io::Result<()> {
        // let mut original_path = path.clone(); // NOTE - Would be better if rename was non mutable
        // original_path.rename(new_name);
        log::error!("disk rename {} {}", path, new_path);
        self.handle
            .local_rename(path.set_relative(), new_path.set_relative())
    }

    pub fn read_file(&self, path: WhPath, offset: u64, len: u64) -> io::Result<Vec<u8>> {
        let file = self.handle.open_file(path.set_relative())?;
        let mut buf = Vec::with_capacity(
            len.try_into()
                .expect("disk_manager::read_file: can't convert u64 to usize"),
        );

        file.read_exact_at(&mut buf, offset)?;
        Ok(buf)
    }

    pub fn new_dir(&self, path: WhPath) -> io::Result<()> {
        self.handle.create_dir(path.set_relative(), 0o644) // TODO look more in c mode_t value
    }
}

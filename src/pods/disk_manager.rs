use std::{fs::File, io::Read, os::unix::fs::FileExt};

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
        let file = self.handle.append_file(path.set_relative(), 0o600)?;
        Ok(file.write_at(&binary, offset)? as u64) // NOTE - used "as" because into() is not supported
    }

    pub fn set_file_size(&self, path: WhPath, size: u64) -> io::Result<()> {
        let file = self.handle.write_file(path.set_relative(), 0o600)?;
        file.set_len(size)
    }
    
    pub fn read_file(&self, path: WhPath, offset: u64, len: u64) -> io::Result<Vec<u8>> {
        let file = self.handle.open_file(path.set_relative())?;
        let mut buf = Vec::<u8>::new();
        buf.splice(
            0..0,
            file.bytes()
                .skip(offset as usize)
                .take(len as usize)
                .map_while(|b| b.ok()),
        );

        Ok(buf)
    }

    pub fn new_dir(&self, path: WhPath) -> io::Result<()> {
        self.handle.create_dir(path.set_relative(), 0o644) // TODO look more in c mode_t value
    }
}

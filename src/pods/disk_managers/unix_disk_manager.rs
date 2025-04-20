use std::{ffi::CString, fs::File, io::Read, os::unix::fs::FileExt};

use std::fs::File;

use openat::{AsPath, Dir};
use tokio::io;

use crate::pods::whpath::WhPath;

use super::DiskManager;

impl AsPath for WhPath {
    type Buffer = CString;

    fn to_path(self) -> Option<Self::Buffer> {
        CString::new(self.inner).ok()
    }
}

pub struct UnixDiskManager {
    handle: Dir,
    mount_point: WhPath, // mountpoint on linux and mirror mountpoint on windows
}

impl UnixDiskManager {
    pub fn new(mount_point: &WhPath) -> io::Result<Self> {
        // /!\
        // /!\

        unsafe { libc::umask(0o000) }; //TODO: Remove when handling permissions

        // /!\
        // /!\

        Ok(Self {
            handle: Dir::open(mount_point.clone())?,
            mount_point,
        })
    }
}

/// always takes a WhPath and infers the real disk path
impl DiskManager for UnixDiskManager {
    // Very simple util to log the content of a folder locally
    fn log_arbo(&self, path: &WhPath) -> io::Result<()> {
        let dirs = self.handle.list_dir(path.clone().set_relative())?;
        for dir in dirs {
            match dir {
                Ok(entry) => log::debug!("|{:?} => {:?}|", entry.file_name(), entry.simple_type()),
                Err(err) => return Err(err),
            }
        }
        Ok(())
    }

    fn new_file(&self, path: &WhPath) -> io::Result<()> {
        self.handle.new_file(path.clone().set_relative(), 0o644)?;  // TODO look more in c mode_t value
        Ok(())
    }

    fn remove_file(&self, path: &WhPath) -> io::Result<()> {
        self.handle.remove_file(path.clone().set_relative())
    }

    fn remove_dir(&self, path: &WhPath) -> io::Result<()> {
        self.handle.remove_dir(path.clone().set_relative())
    }

    fn write_file(&self, path: &WhPath, binary: &[u8], offset: usize) -> io::Result<usize> {
        let file = self.handle.append_file(path.clone().set_relative(), 0o600)?;
        Ok(file.write_at(&binary, offset)?) // NOTE - used "as" because into() is not supported
    }

    fn set_file_size(&self, path: &WhPath, size: usize) -> io::Result<()> {
        let file = self.handle.write_file(path.clone().set_relative(), 0o600)?;
        file.set_len(size)
    }

    fn mv_file(&self, path: &WhPath, new_path: &WhPath) -> io::Result<()> {
        // let mut original_path = path.clone(); // NOTE - Would be better if rename was non mutable
        // original_path.rename(new_name);
        self.handle
            .local_rename(path.clone().set_relative(), new_path.clone().set_relative())
    }

    fn read_file(&self, path: &WhPath, offset: usize, buf: &mut [u8]) -> io::Result<usize> {
        let file = self.handle.open_file(path.clone().set_relative())?;
        Ok(buf.splice(
            0..0,
            file.bytes()
                .skip(offset)
                .take(len)
                .map_while(|b| b.ok()),
        ).len())
    }

    fn new_dir(&self, path: &WhPath) -> io::Result<()> {
        self.handle.create_dir(path.clone().set_relative(), 0o644) // TODO look more in c mode_t value
    }

    fn free_size(&self) -> std::io::Result<usize> {
        todo!() // TODO: implement in linux
    }

    fn size(&self) ->std::io::Result<usize> {
        todo!() // TODO: implement in linux
    }
}

use std::{
    ffi::CString,
    fs::Permissions,
    io::Read,
    os::unix::fs::{FileExt, PermissionsExt},
};

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
            mount_point: mount_point.clone(),
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

    fn new_file(&self, path: &WhPath, mode: u16) -> io::Result<()> {
        self.handle
            .new_file(path.clone().set_relative(), mode.into())?; // TODO look more in c mode_t value
        Ok(())
    }

    fn remove_file(&self, path: &WhPath) -> io::Result<()> {
        self.handle.remove_file(path.clone().set_relative())
    }

    fn remove_dir(&self, path: &WhPath) -> io::Result<()> {
        self.handle.remove_dir(path.clone().set_relative())
    }

    fn write_file(&self, path: &WhPath, binary: &[u8], offset: usize) -> io::Result<usize> {
        let file = self
            .handle
            .append_file(path.clone().set_relative(), 0o600)?;
        Ok(file.write_at(&binary, offset as u64)?) // NOTE - used "as" because into() is not supported
    }

    fn set_file_size(&self, path: &WhPath, size: usize) -> io::Result<()> {
        let file = self
            .handle
            .append_file(path.clone().set_relative(), 0o600)?;
        file.set_len(size as u64)
    }

    fn mv_file(&self, path: &WhPath, new_path: &WhPath) -> io::Result<()> {
        // let mut original_path = path.clone(); // NOTE - Would be better if rename was non mutable
        // original_path.rename(new_name);
        self.handle
            .local_rename(path.clone().set_relative(), new_path.clone().set_relative())
    }

    fn read_file(&self, path: &WhPath, offset: usize, buf: &mut [u8]) -> io::Result<usize> {
        let file = self.handle.open_file(path.clone().set_relative())?;
        let mut read = file
            .bytes()
            .skip(offset)
            .take(buf.len())
            .map_while(|b| b.ok())
            .collect::<Vec<u8>>();
        let read_len = read.len();
        buf[0..read_len].swap_with_slice(&mut read);
        Ok(read_len)
    }

    fn new_dir(&self, path: &WhPath, permissions: u16) -> io::Result<()> {
        self.handle
            .create_dir(path.clone().set_relative(), permissions.into()) // TODO look more in c mode_t value
    }

    fn set_permisions(&self, path: &WhPath, permissions: u16) -> std::io::Result<()> {
        self.handle
            .open_file(path.clone().set_relative())?
            .set_permissions(Permissions::from_mode(permissions as u32))?;
        Ok(())
    }

    fn size_info(&self) -> std::io::Result<super::DiskSizeInfo> {
        todo!()
    }
}

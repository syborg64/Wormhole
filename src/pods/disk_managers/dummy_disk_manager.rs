use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use libc::ENODEV;
use tokio::io;

use crate::pods::whpath::WhPath;

use super::{DiskManager, DiskSizeInfo};

#[derive(PartialEq, Debug)]
pub enum VirtualFile {
    File(Vec<u8>),
    Folder(Vec<WhPath>),
}

pub struct DummyDiskManager {
    files: Arc<RwLock<HashMap<WhPath, VirtualFile>>>,
    size: Arc<RwLock<usize>>,
    mount_point: WhPath, // mountpoint on linux and mirror mountpoint on windows
}

impl DummyDiskManager {
    pub fn new(mount_point: &WhPath) -> io::Result<Self> {
        let mut folders = HashMap::new();
        folders.insert(WhPath::from("."), VirtualFile::Folder(vec![]));
        folders.insert(WhPath::from(""), VirtualFile::Folder(vec![]));
        Ok(Self {
            files: Arc::new(RwLock::new(folders)),
            mount_point: mount_point.clone(),
            size: Arc::new(RwLock::new(0)),
        })
    }

    fn mv_recurse(&self, old_path: &WhPath, new_path: &WhPath) {
        let removed = self
            .files
            .write()
            .expect("VirtDisk::tree rwLock")
            .remove(old_path); // if let does not drop temporaries
        if let Some(file) = removed {
            if let VirtualFile::Folder(entries) = &file {
                entries.iter().for_each(|name| {
                    let mut old_path = new_path.clone();
                    old_path.push(&name.get_end());
                    let mut new_path = new_path.clone();
                    new_path.push(&name.get_end());
                    self.mv_recurse(&old_path, &new_path);
                });
            }
            self.files
                .try_write()
                .expect("VirtDisk::tree rwLock")
                .insert(new_path.clone(), file);
        }
    }
}

/// always takes a WhPath and infers the real disk path
impl DiskManager for DummyDiskManager {
    fn new_file(&self, path: &WhPath, _permissions: u16) -> io::Result<()> {
        let (f_path, name) = path.split_folder_file();
        log::error!("new file path: {}: {} , {}", &path.clone().set_relative().inner, f_path, name);
        self.files
            .write()
            .expect("VirtDisk::new_file rwLock")
            .insert(path.clone().set_relative(), VirtualFile::File(Vec::new()));
        Ok(())
    }

    fn remove_file(&self, path: &WhPath) -> io::Result<()> {
        *self.size.write().expect("VirtDisk::remove_file rwLock") -= match self.files
            .write()
            .expect("VirtDisk::remove_file rwLock")
            .remove(&path.clone().set_relative()) {
                Some(VirtualFile::File(vec)) => vec.len(),
                _ => 0,
            };
        Ok(())
    }

    fn mv_file(&self, old_path: &WhPath, new_path: &WhPath) -> io::Result<()> {
        self.mv_recurse(&old_path.clone().set_relative(), &new_path.clone().set_relative());
        Ok(())
    }

    fn remove_dir(&self, path: &WhPath) -> io::Result<()> {
        self.files
            .write()
            .expect("VirtDisk::remove_dir rwLock")
            .remove(&path.clone().set_relative());
        Ok(())
    }

    fn write_file(&self, path: &WhPath, binary: &[u8], offset: usize) -> io::Result<usize> {
        if let Some(VirtualFile::File(file)) = self
            .files
            .write()
            .expect("VirtDisk::write_file rwLock")
            .get_mut(&path.clone().set_relative())
        {
            let len = binary.len();
            let grow = std::cmp::max(0usize, offset + len - file.len());
            *self.size.write().expect("VirtDisk::write_file rwLock") += grow;
            file.splice((offset)..(offset), binary.iter().cloned());
            Ok(len)
        } else {
            Err(io::Error::from_raw_os_error(ENODEV))
        }
    }

    fn set_file_size(&self, path: &WhPath, size: usize) -> io::Result<()> {
        if let Some(VirtualFile::File(file)) = self
            .files
            .write()
            .expect("VirtDisk::write_file rwLock")
            .get_mut(&path.clone().set_relative())
        {
            let grow = size - file.len();
            *self.size.write().expect("VirtDisk::write_file rwLock") += grow;

            file.resize(size, 0);
            Ok(())
        } else {
            Err(io::Error::from_raw_os_error(ENODEV))
        }
    }

    fn read_file(&self, path: &WhPath, offset: usize, buf: &mut [u8]) -> io::Result<usize> {
        if let Some(VirtualFile::File(file)) = self
            .files
            .read()
            .expect("VirtDisk::read_file rwLock")
            .get(&path.clone().set_relative())
        {
            let len = std::cmp::min(buf.len(), file.len() - offset);
            buf[0..len].copy_from_slice(&file[(offset)..(offset + len)]);
            Ok(len)
        } else {
            Err(io::Error::from_raw_os_error(ENODEV))
        }
    }

    fn new_dir(&self, path: &WhPath, _permissions: u16) -> io::Result<()> {
        self.files
            .write()
            .expect("VirtDisk::new_dir rwLock")
            .insert(path.clone().set_relative(), VirtualFile::Folder(vec![]));
        Ok(())
    }

    fn size_info(&self) -> io::Result<DiskSizeInfo> {
        let s = sysinfo::System::new_all();
        Ok(DiskSizeInfo {
            free_size: s.available_memory() as usize,
            total_size: self.size.read().map(|s|*s).map_err(|_| io::Error::new(io::ErrorKind::Other.into(), "poison error"))?,
        })
    }

    fn log_arbo(&self, path: String) -> std::io::Result<()> {
        todo!()
    }

    fn set_permisions(&self, _path: &WhPath, _permissions: u16) -> io::Result<()> {
        Ok(())
    }
}

// use std::{fs::File, io::Write, os::unix::fs::FileExt};
use std::fs::File;

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use libc::ENODEV;
use log::debug;
// use openat::Dir;
use tokio::io;

use super::whpath::WhPath;

#[derive(PartialEq)]
pub enum Type {
    File,
    Folder,
}

pub struct DiskManager {
    folders: Arc<RwLock<HashMap<String, HashMap<String, Type>>>>,
    files: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    // handle: Dir,
    mount_point: WhPath, // mountpoint on linux and mirror mountpoint on windows
}

/// always takes a WhPath and infers the real disk path
impl DiskManager {
    pub fn new(mount_point: WhPath) -> io::Result<Self> {
        let mut folders = HashMap::new();
        folders.insert(".".to_string(), HashMap::new());
        folders.insert("".to_string(), HashMap::new());
        Ok(Self {
            // handle: Dir::open(mount_point.clone())?,
            files: Arc::new(RwLock::new(HashMap::new())),
            folders: Arc::new(RwLock::new(folders)),
            mount_point,
        })
    }

    pub fn new_file(&self, mut path: WhPath) -> io::Result<()> {
        path = path.set_relative();
        let (f_path, name) = path.split_folder_file();
        log::error!("new file path: {}: {} , {}", &path.inner, f_path, name);
        if let Some(entries) = self.folders.write().expect("VirtDisk::new_file rwLock").get_mut(&f_path) {
            entries.insert(name, Type::File);
            self.files.write().expect("VirtDisk::new_file rwLock").insert(path.inner.clone(), Vec::new());
            Ok(())
        } else {
            log::error!("new file FAILED");

            Err(std::io::Error::from_raw_os_error(ENODEV))
            // Err(std::io::ErrorKind::NotFound.into())
        }
    }

    pub fn remove_file(&self, mut path: WhPath) -> io::Result<()> {
        path = path.set_relative();
        let (f_path, name) = path.split_folder_file();
        if let Some(entries) = self.folders.write().expect("VirtDisk::remove_file rwLock").get_mut(&f_path) {
            if entries.get(&name) != Some(&Type::File) {
                return Err(std::io::Error::from_raw_os_error(ENODEV));
                // return Err(std::io::ErrorKind::NotFound.into());
            }
            entries.remove(&name);
            self.files.write().expect("VirtDisk::remove_file rwLock").remove(&path.inner);
            Ok(())
        } else {
            Err(std::io::Error::from_raw_os_error(ENODEV))
            // Err(std::io::ErrorKind::NotFound.into())
        }
    }

    pub fn mv_file(&self, old_path: WhPath, new_path: WhPath) -> io::Result<()> {
        Ok(())
    }

    pub fn remove_dir(&self, mut path: WhPath) -> io::Result<()> {
        path = path.set_relative();
        let (f_path, name) = path.split_folder_file();
        if let Some(entries) = self.folders.write().expect("VirtDisk::remove_dir rwLock").get_mut(&f_path) {
            if entries.len() > 0 {
                return Err(io::ErrorKind::InvalidData.into());
            }
            if entries.get(&name) != Some(&Type::Folder) {
                return Err(std::io::Error::from_raw_os_error(ENODEV));
                // return Err(std::io::ErrorKind::NotFound.into());
            }
            entries.remove(&name);
            self.folders.write().expect("VirtDisk::remove_dir rwLock").remove(&path.inner);
            Ok(())
        } else {
            Err(io::Error::from_raw_os_error(ENODEV))
            // Err(io::ErrorKind::NotFound.into())
        }
    }

    pub fn write_file(&self, mut path: WhPath, binary: &[u8], offset: u64) -> io::Result<u64> {
        path = path.set_relative();
        log::error!("write file path: {}", &path.inner);
        if let Some(file) = self.files.write().expect("VirtDisk::write_file rwLock").get_mut(&path.inner) {
            let len = binary.len();
            file.splice((offset as usize)..(offset as usize), binary.iter().cloned());
            Ok(len as u64)
        } else {
            Err(io::Error::from_raw_os_error(ENODEV))
            // Err(io::ErrorKind::NotFound.into())
        }
    }

    pub fn set_file_size(&self, path: WhPath, size: u64) -> io::Result<()> {
        if let Some(file) = self.files.write().expect("VirtDisk::write_file rwLock").get_mut(&path.inner) {
            file.resize(size as usize, 0);
            Ok(())
        } else {
            Err(io::Error::from_raw_os_error(ENODEV))
        }
    }

    pub fn read_file(&self, mut path: WhPath, offset: u64, len: u64) -> io::Result<Vec<u8>> {
        path = path.set_relative();
        log::error!("read file path: {}", &path.inner);
        if let Some(file) = self.files.read().expect("VirtDisk::read_file rwLock").get(&path.inner) {
            let len = std::cmp::min(len as usize, file.len() - offset as usize);
            let data = Vec::from(&file[(offset as usize)..(offset as usize + len)]);
            Ok(data)
        } else {
            Err(io::Error::from_raw_os_error(ENODEV))
            // Err(io::ErrorKind::NotFound.into())
        }
    }

    pub fn new_dir(&self, mut path: WhPath) -> io::Result<()> {
        path = path.set_relative();
        log::error!("new folder path: {}", &path.inner);
        let (f_path, name) = path.split_folder_file();
        if let Some(entries) = self.folders.write().expect("VirtDisk::new_dir rwLock").get_mut(&f_path) {
            entries.insert(name, Type::Folder);
            Ok(())
        } else {
            Err(std::io::Error::from_raw_os_error(ENODEV))
            // Err(std::io::ErrorKind::NotFound.into())
        }
    }
}

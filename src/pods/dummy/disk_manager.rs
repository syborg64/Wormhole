use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use libc::ENODEV;
use tokio::io;

use super::whpath::WhPath;

#[derive(PartialEq)]
pub enum VirtualFile {
    File(Vec<u8>),
    Folder(Vec<WhPath>),
}

pub struct DiskManager {
    files: Arc<RwLock<HashMap<WhPath, VirtualFile>>>,
    size: Arc<RwLock<usize>>,
    mount_point: WhPath, // mountpoint on linux and mirror mountpoint on windows
}

/// always takes a WhPath and infers the real disk path
impl DiskManager {
    pub fn new(mount_point: WhPath) -> io::Result<Self> {
        let mut folders = HashMap::new();
        folders.insert(WhPath::from("."), VirtualFile::Folder(vec![]));
        folders.insert(WhPath::from(""), VirtualFile::Folder(vec![]));
        Ok(Self {
            files: Arc::new(RwLock::new(folders)),
            mount_point,
            size: Arc::new(RwLock::new(0)),
        })
    }

    pub fn new_file(&self, mut path: WhPath) -> io::Result<()> {
        path = path.set_relative();
        let (f_path, name) = path.split_folder_file();
        log::error!("new file path: {}: {} , {}", &path.inner, f_path, name);
        self.files
            .write()
            .expect("VirtDisk::new_file rwLock")
            .insert(path.clone(), VirtualFile::File(Vec::new()));
        Ok(())
    }

    pub fn remove_file(&self, mut path: WhPath) -> io::Result<()> {
        path = path.set_relative();
        *self.size.write().expect("VirtDisk::remove_file rwLock") -= match self.files
            .write()
            .expect("VirtDisk::remove_file rwLock")
            .remove(&path) {
                Some(VirtualFile::File(vec)) => vec.len(),
                _ => 0,
            };
        Ok(())
    }

    pub fn mv_file(&self, mut old_path: WhPath, mut new_path: WhPath) -> io::Result<()> {
        old_path = old_path.set_relative();
        new_path = new_path.set_relative();
        self.mv_recurse(&old_path, &new_path);
        Ok(())
    }

    fn mv_recurse(&self, old_path: &WhPath, new_path: &WhPath) {
        if let Some(file) = self
            .files
            .write()
            .expect("VirtDisk::tree rwLock")
            .remove(old_path)
        {
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
                .write()
                .expect("VirtDisk::tree rwLock")
                .insert(new_path.clone(), file);
        }
    }

    pub fn remove_dir(&self, path: WhPath) -> io::Result<()> {
        self.files
            .write()
            .expect("VirtDisk::remove_dir rwLock")
            .remove(&path);
        Ok(())
    }

    pub fn write_file(&self, mut path: WhPath, binary: &[u8], offset: usize) -> io::Result<usize> {
        path = path.set_relative();
        if let Some(VirtualFile::File(file)) = self
            .files
            .write()
            .expect("VirtDisk::write_file rwLock")
            .get_mut(&path)
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

    pub fn set_file_size(&self, path: WhPath, size: usize) -> io::Result<()> {
        if let Some(VirtualFile::File(file)) = self
            .files
            .write()
            .expect("VirtDisk::write_file rwLock")
            .get_mut(&path)
        {
            let grow = size - file.len();
            *self.size.write().expect("VirtDisk::write_file rwLock") += grow;

            file.resize(size, 0);
            Ok(())
        } else {
            Err(io::Error::from_raw_os_error(ENODEV))
        }
    }

    pub fn read_file(&self, mut path: WhPath, offset: usize, len: usize) -> io::Result<Vec<u8>> {
        path = path.set_relative();
        if let Some(VirtualFile::File(file)) = self
            .files
            .read()
            .expect("VirtDisk::read_file rwLock")
            .get(&path)
        {
            let len = std::cmp::min(len, file.len() - offset);
            let data = Vec::from(&file[(offset)..(offset + len)]);
            Ok(data)
        } else {
            Err(io::Error::from_raw_os_error(ENODEV))
        }
    }

    pub fn new_dir(&self, mut path: WhPath) -> io::Result<()> {
        path = path.set_relative();
        self.files
            .write()
            .expect("VirtDisk::new_dir rwLock")
            .insert(path, VirtualFile::Folder(vec![]));
        Ok(())
    }

    pub fn free_size(&self) -> io::Result<usize> {
        let s = sysinfo::System::new_all();
        Ok(s.available_memory() as usize)
    }

    pub fn size(&self) ->io::Result<usize> {
        self.size.read().map(|s|*s).map_err(|_| io::Error::new(io::ErrorKind::Other.into(), "poison error"))//.expect("VirtDisk::size rwLock")
    }
}

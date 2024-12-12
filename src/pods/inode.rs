use std::{collections::HashMap, io, path::PathBuf, sync::Arc};

use dashmap::DashMap;
use fuser::FileType;
use serde::{Deserialize, Serialize};

use crate::{network::message::Address, providers::whpath::WhPath};

pub const ROOT: InodeId = 0;

#[derive(Debug, Serialize, Deserialize)]
pub struct Inode {
    parent: InodeId,
    id: InodeId,
    name: String,
    entry: FsEntry,
}

pub type InodeId = u64;
pub type ArboIndex = HashMap<InodeId, Inode>;
pub struct Arbo {
    entries: ArboIndex,
}

/// InodeId is represented by an u64
pub type Hosts = Vec<Address>;

/// Hashmap containing file system data
/// (inode_number, (Type, Original path, Hosts))
pub type FsIndex = HashMap<InodeId, FsEntry>;

#[derive(Debug, Serialize, Deserialize, Clone)]
/// Should be extended until meeting [fuser::FileType]
pub enum FsEntry {
    File(Hosts),
    Directory(Vec<InodeId>),
}

impl FsEntry {
    // pub fn get_path(&self) -> &PathBuf {
    //     match self {
    //         FsEntry::File(path) => path,
    //         FsEntry::Directory(children) => path,
    //     }
    // }

    // pub fn get_name(&self) -> io::Result<&OsStr> {
    //     match Path::new(self.get_path()).file_name() {
    //         Some(name) => Ok(name),
    //         None => Err(io::Error::new(io::ErrorKind::Other, "Invalid path ending")),
    //     }
    // }

    pub fn get_filetype(&self) -> FileType {
        match self {
            FsEntry::File(_) => FileType::RegularFile,
            FsEntry::Directory(_) => FileType::Directory,
        }
    }

    pub fn get_children(&self) -> io::Result<&Vec<InodeId>> {
        match self {
            FsEntry::File(_) => Err(io::Error::new(
                io::ErrorKind::Other,
                "entry is not a directory",
            )),
            FsEntry::Directory(children) => Ok(children),
        }
    }
}

impl Arbo {
    pub fn add_inode(
        &mut self,
        name: String,
        ino: u64,
        parent_ino: u64,
        entry: FsEntry,
    ) -> io::Result<()> {
        if self.entries.contains_key(&ino) {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "file already existing",
            ))
        } else if !self.entries.contains_key(&parent_ino) {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "parent not existing",
            ))
        } else {
            match self.entries.get_mut(&parent_ino) {
                None => Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "parent not existing",
                )),
                Some(Inode {
                    parent: _,
                    id: _,
                    name: _,
                    entry: FsEntry::Directory(parent_children),
                }) => {
                    let new_entry = Inode {
                        parent: parent_ino,
                        id: ino.clone(),
                        name: name,
                        entry: entry,
                    };
                    parent_children.push(ino);
                    self.entries.insert(ino, new_entry);
                    Ok(())
                }
                Some(_) => Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "parent not a folder",
                )),
            }
        }
    }

    pub fn path_from_inode_id(&self, inode_index: InodeId) -> io::Result<WhPath> {
        if inode_index == ROOT {
            return Ok(WhPath::new("/"));
        }
        let inode = match self.entries.get(&inode_index) {
            Some(inode) => inode,
            None => {
                return Err(io::Error::new(io::ErrorKind::NotFound, "entry not found"));
            }
        };

        let mut parent_path = self.path_from_inode_id(inode.parent)?;
        parent_path.join(inode.name.clone());
        Ok(parent_path)
    }

    pub fn get_inode_child_by_name(&self, parent: &Inode, name: &String) -> io::Result<&Inode> {
        if let Ok(children) = parent.entry.get_children() {
            for child in children.iter() {
                if let Some(child) = self.entries.get(child) {
                    if child.name == *name {
                        return Ok(child);
                    }
                }
            }
            Err(io::Error::new(io::ErrorKind::NotFound, "entry not found"))
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "entry is not a directory",
            ))
        }
    }

    pub fn inode_from_path(&self, mut path: WhPath) -> io::Result<&Inode> {
        let mut actual_inode= self.entries.get(&ROOT).expect("inode_from_path: NO ROOT");

        for name in path.to_vector().iter() {
            actual_inode = self.get_inode_child_by_name(&actual_inode, name)?;
        }

        Ok(actual_inode)
    }
}

use std::{collections::HashMap, io, path::PathBuf, sync::Arc};

use fuser::FileType;
use serde::{Deserialize, Serialize};

use crate::{network::message::Address, providers::whpath::WhPath};

pub const ROOT: InodeIndex = 0;

#[derive(Debug, Serialize, Deserialize)]
pub struct Inode {
    parent_index: Arc<InodeIndex>,
    index: Arc<InodeIndex>,
    name: String,
    entry: FsEntry,
}
pub struct Arbo {
    tree: Arc<Inode>, /* ROOT */
    index: ArboIndex,
}

pub type ArboIndex = HashMap<InodeIndex, Inode>;

/// InodeIndex is represented by an u64
pub type InodeIndex = u64;

pub type Hosts = Vec<Address>;

/// Hashmap containing file system data
/// (inode_number, (Type, Original path, Hosts))
pub type FsIndex = HashMap<InodeIndex, FsEntry>;

#[derive(Debug, Serialize, Deserialize, Clone)]
/// Should be extended until meeting [fuser::FileType]
pub enum FsEntry {
    File(Hosts),
    Directory(Vec<Arc<Inode>>),
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

    pub fn get_children(&self) -> io::Result<&Vec<Arc<Inode>>> {
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
    pub fn path_from_inode_index(&self, inode_index: InodeIndex) -> io::Result<WhPath> {
        if inode_index == ROOT {
            return Ok(WhPath::new("/"));
        }
        let inode = match self.index.get(&inode_index) {
            Some(inode) => inode,
            None => {
                return Err(io::Error::new(io::ErrorKind::NotFound, "entry not found"));
            }
        };

        let mut parent_path = self.path_from_inode_index(*inode.parent_index)?;
        parent_path.join(inode.name.clone());
        Ok(parent_path)
    }

    pub fn get_inode_child_by_name(&self, inode: &Inode, name: &String) -> io::Result<Arc<Inode>> {
        if let Ok(children) = inode.entry.get_children() {
            for child in children.iter() {
                if *child.name == *name {
                    return Ok(child.clone());
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

    pub fn inode_from_path(&self, mut path: WhPath) -> io::Result<Arc<Inode>> {
        let mut actual_inode: Arc<Inode> = self.tree.clone();

        for name in path.to_vector().iter() {
            actual_inode = self.get_inode_child_by_name(&actual_inode, name)?;
        }

        Ok(actual_inode)
    }
}

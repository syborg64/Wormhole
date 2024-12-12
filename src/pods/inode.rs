use std::{collections::HashMap, io, path::PathBuf, sync::Arc};

use fuser::FileType;
use serde::{Deserialize, Serialize};

use crate::{network::message::Address, providers::whpath::WhPath};

pub const ROOT: InodeIndex = 0;

#[derive(Debug, Serialize, Deserialize)]
pub struct Inode {
    parent_index: InodeIndex,
    index: InodeIndex,
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
    fn hashmap_insert(&mut self, key: InodeIndex, value: Arc<Inode>) -> io::Result<()> {
        self.index.insert(key, value).is_some() {
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::AddrNotAvailable, "inode already in use"))
        }
    }

    fn tree_insert(&mut self, path: WhPath, inode: Arc<Inode>) -> io::Result<()> {
        let insert_into = self.inode_from_path(path)?.index;
        let insert_into = match self.index.get_mut(&insert_into) {
            Some(insert_into) => insert_into,
            None => return Err(io::Error::new(io::ErrorKind::NotFound, "tree_insert: path not found"))
        };

        match insert_into.entry {
            FsEntry::File(_) => Err(io::Error::new(io::ErrorKind::NotFound, "tree_insert: path not found")),
            FsEntry::Directory(children) => Ok(children.push(inode)),
        }
    }

    pub fn add_inode(&mut self, mut path: WhPath, ino: u64, parent_ino: u64, entry: FsEntry) -> io::Result<()> {
        if self.index.contains_key(&ino) {
            Err(io::Error::new(io::ErrorKind::InvalidInput, "file already existing"))
        } else {
            let insertion = Arc::new(Inode {
                parent_index: parent_ino,
                index: ino.clone(),
                name: path.get_end(),
                entry: entry,
            });

            self.hashmap_insert(ino, insertion.clone());

            // insertion in tree

        }
    }
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

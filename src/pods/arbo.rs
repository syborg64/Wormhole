use crate::{error::WhResult, network::message::Address};
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs, io,
    sync::Arc,
    time::{Duration, SystemTime},
};

use crate::error::WhError;
use crate::pods::filesystem::fs_interface::SimpleFileType;
use crate::pods::whpath::WhPath;

// SECTION consts

/*  NOTE - fuse root folder inode is 1.
    other inodes can start wherever we want
*/
pub const ROOT: InodeId = 1;
pub const LOCK_TIMEOUT: Duration = Duration::new(5, 0);

// !SECTION

// SECTION types

/// InodeId is represented by an u64
pub type Hosts = Vec<Address>;
pub type InodeId = u64;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
/// Should be extended until meeting [fuser::FileType]
pub enum FsEntry {
    File(Hosts),
    Directory(Vec<InodeId>),
}

pub type XAttrs = HashMap<String, Vec<u8>>;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Inode {
    pub parent: InodeId,
    pub id: InodeId,
    pub name: String,
    pub entry: FsEntry,
    pub meta: Metadata,
    pub xattrs: XAttrs,
}

pub type ArboIndex = HashMap<InodeId, Inode>;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Arbo {
    entries: ArboIndex,
}

// !SECTION

// SECTION implementations

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

    pub fn get_filetype(&self) -> SimpleFileType {
        match self {
            FsEntry::File(_) => SimpleFileType::File,
            FsEntry::Directory(_) => SimpleFileType::Directory,
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

impl Inode {
    pub fn new(name: String, parent_ino: InodeId, id: InodeId, entry: FsEntry) -> Self {
        let meta = Metadata {
            ino: id,
            size: 0,
            blocks: 0,
            atime: SystemTime::now(),
            mtime: SystemTime::now(),
            ctime: SystemTime::now(),
            crtime: SystemTime::now(),
            kind: match entry {
                FsEntry::Directory(_) => SimpleFileType::Directory,
                FsEntry::File(_) => SimpleFileType::File,
            },
            perm: 0o777,
            nlink: 0,
            uid: 0,
            gid: 0,
            rdev: 0,
            blksize: 1,
            flags: 0,
        };

        let xattrs = HashMap::new();

        Self {
            parent: parent_ino,
            id: id,
            name: name,
            entry: entry,
            meta,
            xattrs,
        }
    }
}

impl Arbo {
    pub fn new() -> Self {
        let mut arbo: Self = Self {
            entries: HashMap::new(),
        };

        arbo.entries.insert(
            ROOT,
            Inode {
                parent: ROOT,
                id: ROOT,
                name: "/".to_owned(),
                entry: FsEntry::Directory(vec![]),
                meta: Metadata {
                    ino: 0,
                    size: 0,
                    blocks: 0,
                    atime: SystemTime::now(),
                    mtime: SystemTime::now(),
                    ctime: SystemTime::now(),
                    crtime: SystemTime::now(),
                    kind: SimpleFileType::Directory,
                    perm: 0o666,
                    nlink: 0,
                    uid: 0,
                    gid: 0,
                    rdev: 0,
                    blksize: 1,
                    flags: 0,
                },
                xattrs: HashMap::new(),
            },
        );
        arbo
    }

    pub fn overwrite_self(&mut self, entries: ArboIndex) {
        self.entries = entries;
    }

    pub fn get_raw_entries(&self) -> ArboIndex {
        self.entries.clone()
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, InodeId, Inode> {
        self.entries.iter()
    }

    #[must_use]
    pub fn read_lock<'a>(
        arbo: &'a Arc<RwLock<Arbo>>,
        called_from: &'a str,
    ) -> io::Result<RwLockReadGuard<'a, Arbo>> {
        if let Some(arbo) = arbo.try_read_for(LOCK_TIMEOUT) {
            Ok(arbo)
        } else {
            Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                format!("{}: unable to read_lock arbo", called_from),
            ))
        }
    }

    #[must_use]
    pub fn n_read_lock<'a>(
        arbo: &'a Arc<RwLock<Arbo>>,
        called_from: &'a str,
    ) -> WhResult<RwLockReadGuard<'a, Arbo>> {
        arbo.try_read_for(LOCK_TIMEOUT).ok_or(WhError::WouldBlock {
            called_from: called_from.to_owned(),
        })
    }

    #[must_use]
    pub fn write_lock<'a>(
        arbo: &'a Arc<RwLock<Arbo>>,
        called_from: &'a str,
    ) -> io::Result<RwLockWriteGuard<'a, Arbo>> {
        if let Some(arbo) = arbo.try_write_for(LOCK_TIMEOUT) {
            Ok(arbo)
        } else {
            Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                format!("{}: unable to write_lock arbo", called_from),
            ))
        }
    }

    #[must_use]
    pub fn n_write_lock<'a>(
        arbo: &'a Arc<RwLock<Arbo>>,
        called_from: &'a str,
    ) -> WhResult<RwLockWriteGuard<'a, Arbo>> {
        arbo.try_write_for(LOCK_TIMEOUT).ok_or(WhError::WouldBlock {
            called_from: called_from.to_owned(),
        })
    }

    pub fn files_hosted_only_by<'a>(
        &'a self,
        host: &'a Address,
    ) -> impl Iterator<Item = Inode> + use<'a> {
        self.iter()
            .filter_map(move |(_, inode)| match &inode.entry {
                FsEntry::Directory(_) => None,
                FsEntry::File(hosts) => {
                    if hosts.len() == 1 && hosts.contains(&host) {
                        Some(inode.clone())
                    } else {
                        None
                    }
                }
            })
    }

    #[must_use]
    pub fn add_inode_from_parameters(
        &mut self,
        name: String,
        ino: InodeId,
        parent_ino: InodeId,
        entry: FsEntry,
    ) -> io::Result<()> {
        if self.entries.contains_key(&ino) {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "add_inode_from_parameters: file already existing",
            ))
        } else {
            match self.entries.get_mut(&parent_ino) {
                None => Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "add_inode_from_parameters: parent not existing",
                )),
                Some(Inode {
                    parent: _,
                    id: _,
                    name: _,
                    entry: FsEntry::Directory(parent_children),
                    meta: _,
                    xattrs: _,
                }) => {
                    let new_entry = Inode::new(name, parent_ino, ino, entry);
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

    #[must_use]
    pub fn add_inode(&mut self, inode: Inode) -> io::Result<()> {
        self.add_inode_from_parameters(inode.name, inode.id, inode.parent, inode.entry)
    }

    #[must_use]
    pub fn remove_children(&mut self, parent: InodeId, child: InodeId) -> io::Result<()> {
        let parent = self.get_inode_mut(parent)?;

        let children = match &mut parent.entry {
            FsEntry::File(_) => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "remove_children: specified parent is not a folder",
            )),
            FsEntry::Directory(children) => Ok(children),
        }?;

        children.retain(|v| *v != child);
        Ok(())
    }

    #[must_use]
    pub fn add_children(&mut self, parent: InodeId, child: InodeId) -> io::Result<()> {
        let parent = self.get_inode_mut(parent)?;

        let children = match &mut parent.entry {
            FsEntry::File(_) => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "add_children: specified parent is not a folder",
            )),
            FsEntry::Directory(children) => Ok(children),
        }?;

        children.push(child);
        Ok(())
    }

    #[must_use]
    pub fn remove_inode(&mut self, id: InodeId) -> io::Result<Inode> {
        let removed = match self.entries.remove(&id) {
            Some(inode) => Ok(inode),
            None => Err(io::Error::new(
                io::ErrorKind::NotFound,
                "remove_inode: specified inode not found",
            )),
        }?;

        self.remove_children(removed.parent, id)?;

        Ok(removed)
    }

    #[must_use]
    pub fn get_inode(&self, ino: InodeId) -> io::Result<&Inode> {
        match self.entries.get(&ino) {
            Some(inode) => Ok(inode),
            None => Err(io::Error::new(io::ErrorKind::NotFound, "entry not found")),
        }
    }

    #[must_use]
    pub fn n_get_inode(&self, ino: InodeId) -> WhResult<&Inode> {
        self.entries.get(&ino).ok_or(WhError::InodeNotFound)
    }

    #[must_use]
    pub fn mv_inode(
        &mut self,
        parent: InodeId,
        new_parent: InodeId,
        name: &String,
        new_name: &String,
    ) -> io::Result<()> {
        let parent_inode = self.entries.get(&parent).ok_or(io::Error::new(
            io::ErrorKind::NotFound,
            "add_inode_from_parameters: parent not existing",
        ))?;
        let item_id = match self.get_inode_child_by_name(parent_inode, name) {
            Ok(item_inode) => item_inode.id,
            Err(_) => todo!("mv_inode: inode not found"), // TODO
        };

        self.remove_children(parent, item_id)?;

        let item = self.get_inode_mut(item_id)?;
        item.name = new_name.clone();
        item.parent = new_parent;

        self.add_children(new_parent, item_id)
    }

    // not public as the modifications are not automaticly propagated on other related inodes
    #[must_use]
    fn get_inode_mut(&mut self, ino: InodeId) -> io::Result<&mut Inode> {
        self.entries
            .get_mut(&ino)
            .ok_or(io::Error::new(io::ErrorKind::NotFound, "entry not found"))
    }

    // not public as the modifications are not automaticly propagated on other related inodes
    #[must_use]
    fn n_get_inode_mut(&mut self, ino: InodeId) -> WhResult<&mut Inode> {
        self.entries.get_mut(&ino).ok_or(WhError::InodeNotFound)
    }

    #[must_use]
    pub fn n_get_path_from_inode_id(&self, inode_index: InodeId) -> WhResult<WhPath> {
        if inode_index == ROOT {
            return Ok(WhPath::from("/"));
        }
        let inode = self
            .entries
            .get(&inode_index)
            .ok_or(WhError::InodeNotFound)?;

        let mut parent_path = self.n_get_path_from_inode_id(inode.parent)?;
        parent_path.push(&inode.name.clone());
        Ok(parent_path)
    }

    #[must_use]
    pub fn get_path_from_inode_id(&self, inode_index: InodeId) -> io::Result<WhPath> {
        if inode_index == ROOT {
            return Ok(WhPath::from("/"));
        }
        let inode = match self.entries.get(&inode_index) {
            Some(inode) => inode,
            None => {
                return Err(io::Error::new(io::ErrorKind::NotFound, "entry not found"));
            }
        };

        let mut parent_path = self.get_path_from_inode_id(inode.parent)?;
        parent_path.push(&inode.name.clone());
        Ok(parent_path)
    }

    #[must_use]
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

    #[must_use]
    pub fn get_inode_from_path(&self, path: &WhPath) -> io::Result<&Inode> {
        let mut actual_inode = self.entries.get(&ROOT).expect("inode_from_path: NO ROOT");

        for name in path.clone().to_vector().iter() {
            actual_inode = self.get_inode_child_by_name(&actual_inode, name)?;
        }

        Ok(actual_inode)
    }

    pub fn set_inode_hosts(&mut self, ino: InodeId, hosts: Vec<Address>) -> io::Result<()> {
        let inode = self.get_inode_mut(ino)?;

        inode.entry = match &inode.entry {
            FsEntry::File(_) => FsEntry::File(hosts),
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "can't edit hosts on folder",
                ))
            }
        };
        Ok(())
    }

    /// Add hosts to an inode
    ///
    /// Only works on inodes pointing files (no folders)
    /// Ignore already existing hosts to avoid duplicates
    pub fn add_inode_hosts(&mut self, ino: InodeId, new_hosts: Vec<Address>) -> io::Result<()> {
        let inode = self.get_inode_mut(ino)?;

        inode.entry = match &inode.entry {
            FsEntry::File(old_hosts) => FsEntry::File(
                [
                    old_hosts.as_slice(),
                    new_hosts
                        .into_iter()
                        .filter(|host| !old_hosts.contains(host))
                        .collect::<Vec<Address>>()
                        .as_slice(),
                ]
                .concat(),
            ),
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "can't edit hosts on folder",
                ))
            }
        };
        Ok(())
    }

    /// Remove hosts from an inode
    ///
    /// Only works on inodes pointing files (no folders)
    pub fn remove_inode_hosts(
        &mut self,
        ino: InodeId,
        remove_hosts: Vec<Address>,
    ) -> io::Result<()> {
        let inode = self.get_inode_mut(ino)?;

        match &mut inode.entry {
            FsEntry::File(old_hosts) => old_hosts.retain(|host| !remove_hosts.contains(host)),
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "can't edit hosts on folder",
                ))
            }
        };
        Ok(())
    }

    pub fn set_inode_meta(&mut self, ino: InodeId, meta: Metadata) -> io::Result<()> {
        let inode = self.get_inode_mut(ino)?;

        inode.meta = meta;
        Ok(())
    }

    pub fn set_inode_xattr(&mut self, ino: InodeId, key: String, data: Vec<u8>) -> WhResult<()> {
        let inode = self.n_get_inode_mut(ino)?;

        inode.xattrs.insert(key, data);
        Ok(())
    }

    pub fn remove_inode_xattr(&mut self, ino: InodeId, key: String) -> WhResult<()> {
        let inode = self.n_get_inode_mut(ino)?;

        inode.xattrs.remove(&key);
        Ok(())
    }
}

// !SECTION

/// Reserved files names
pub const GLOBAL_CONFIG_INO: u64 = 2;
pub const GLOBAL_CONFIG_FNAME: &str = ".global_config.toml";
pub const LOCAL_CONFIG_INO: u64 = 3;
pub const LOCAL_CONFIG_FNAME: &str = ".local_config.toml";
pub const ARBO_FILE_INO: u64 = 4;
pub const ARBO_FILE_FNAME: &str = ".arbo";

fn index_folder_recursive(
    arbo: &mut Arbo,
    parent: InodeId,
    ino: &mut InodeId,
    path: &WhPath,
    host: &String,
) -> io::Result<()> {
    let str_path = path.to_string();
    for entry in fs::read_dir(str_path)? {
        let entry = entry.expect("error in filesystem indexion (1)");
        let ftype = entry.file_type().expect("error in filesystem indexion (2)");
        let fname = entry.file_name().to_string_lossy().to_string();
        let meta = entry.metadata()?;

        let used_ino = match (fname.as_str(), parent) {
            (GLOBAL_CONFIG_FNAME, 1) => GLOBAL_CONFIG_INO,
            (LOCAL_CONFIG_FNAME, 1) => LOCAL_CONFIG_INO,
            (ARBO_FILE_FNAME, 1) => ARBO_FILE_INO,
            _ => {
                let used = *ino;
                *ino += 1;
                used
            }
        };

        arbo.add_inode(Inode::new(
            fname.clone(),
            parent,
            used_ino,
            if ftype.is_file() {
                FsEntry::File(vec![host.clone()])
            } else {
                FsEntry::Directory(Vec::new())
            },
        ))?;
        arbo.set_inode_meta(used_ino, meta.try_into()?)?;

        if ftype.is_dir() {
            index_folder_recursive(arbo, *ino - 1, ino, &path.join(&fname), host)
                .expect("error in filesystem indexion (3)");
        };
    }
    Ok(())
}

pub fn index_folder(path: &WhPath, host: &String) -> io::Result<(Arbo, InodeId)> {
    let mut arbo = Arbo::new();
    let mut ino: u64 = 11; // NOTE - will be the first registered inode after root

    #[cfg(target_os = "linux")]
    index_folder_recursive(&mut arbo, ROOT, &mut ino, path, host)?;
    Ok((arbo, ino))
}

/* NOTE
 * is currently made with fuse in sight. Will probably need to be edited to be windows compatible
 */
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Metadata {
    /// Inode number
    pub ino: u64,
    /// Size in bytes
    pub size: u64,
    /// Size in blocks
    pub blocks: u64,
    /// Time of last access
    pub atime: SystemTime,
    /// Time of last modification
    pub mtime: SystemTime,
    /// Time of last change
    pub ctime: SystemTime,
    /// Time of creation (macOS only)
    pub crtime: SystemTime,
    /// Kind of file (directory, file, pipe, etc)
    pub kind: SimpleFileType,
    /// Permissions
    pub perm: u16,
    /// Number of hard links
    pub nlink: u32,
    /// User id
    pub uid: u32,
    /// Group id
    pub gid: u32,
    /// Rdev
    pub rdev: u32,
    /// Block size
    pub blksize: u32,
    /// Flags (macOS only, see chflags(2))
    pub flags: u32,
}

impl TryInto<Metadata> for fs::Metadata {
    type Error = std::io::Error;
    fn try_into(self) -> Result<Metadata, std::io::Error> {
        Ok(Metadata {
            ino: 0,
            size: self.len(),
            blocks: 1,
            atime: self.accessed()?,
            mtime: self.modified()?,
            ctime: self.modified()?,
            crtime: self.created()?,
            kind: if self.is_file() {
                SimpleFileType::File
            } else {
                SimpleFileType::Directory
            },
            perm: 0o666 as u16,
            nlink: 0,
            uid: 0,
            gid: 0,
            rdev: 0,
            blksize: 1,
            flags: 0,
        })
    }
}

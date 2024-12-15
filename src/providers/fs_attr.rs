use fuser::{FileAttr, FileType, TimeOrNow};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub type Permission = u16;
pub type AllPerm = u32;

pub type ATime = SystemTime;
pub type MTime = SystemTime;
pub type CTime = SystemTime;
pub type CrTime = SystemTime;

pub type Kind = FileType;

pub type Bytes = u64;
pub type Blocks = u64;
pub type Blksize = u32;

pub type Uid = u32;
pub type Gid = u32;

pub type Nlink = u32;
pub type Rdev = u32; //NOTE - Device ID - Only used for FileType::BlockDevice or FileType::CharDevice otherwise use 0
pub type Flags = u32;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct FsAttr {
    fs_time: (ATime, MTime, CTime, CrTime),
    kind: Kind,
    permission: AllPerm,
    size: (Bytes, Blocks, Blksize),
    id: (Uid, Gid),
    nlink: Nlink,
    rdev: Rdev,
    flags: Flags,
}

impl FsAttr {
    pub fn new(file_type: FileType) -> FsAttr {
        let time = SystemTime::now();
        FsAttr {
            fs_time: (time, time, time, time),
            kind: file_type,
            permission: if file_type == FileType::Directory {
                0o755
            } else {
                0o644
            },
            size: if file_type == FileType::Directory {
                (0, 0, 512)
            } else {
                (13, 1, 512)
            },
            id: (501, 20),
            nlink: if file_type == FileType::Directory {
                2
            } else {
                1
            },
            rdev: 0,
            flags: 0,
        }
    }

    pub fn set_file_attr(
        &mut self,
        mode: AllPerm,
        uid: Uid,
        gid: Gid,
        size: Bytes,
        _atime: SystemTime,
        _mtime: SystemTime,
        _ctime: SystemTime,
        _crtime: SystemTime,
        flags: Flags,
    ) {
        let block_size = self.size.2 as u64;
        self.permission = mode;
        self.id = (uid, gid);
        self.size = (size, (size + block_size - 1) / block_size, self.size.2);
        self.fs_time = (_atime, _mtime, _ctime, _crtime);
        self.flags = flags;
    }

    pub fn get_file_attr(&self) -> FileAttr {
        FileAttr {
            ino: 1, //FIXME - sera de toute façon changer avec la nouvelle implémentation de axel
            size: self.get_size_in_bytes(),
            blocks: self.get_size_in_blocks(),
            atime: self.get_last_access(),
            mtime: self.get_last_modif(),
            ctime: self.get_last_change(),
            crtime: self.get_creation_time(),
            kind: self.get_kind(),
            perm: self.get_permission(),
            nlink: self.get_hard_link_num(),
            uid: self.get_uid(),
            gid: self.get_gid(),
            rdev: self.get_device_id(),
            blksize: self.get_blocks_size(),
            flags: self.get_flags(),
        }
    }

    pub fn get_size_in_bytes(&self) -> Bytes {
        self.size.0
    }

    pub fn get_size_in_blocks(&self) -> Blocks {
        self.size.1
    }

    pub fn get_blocks_size(&self) -> Blksize {
        self.size.2
    }

    pub fn get_last_access(&self) -> ATime {
        self.fs_time.0
    }

    pub fn get_last_modif(&self) -> MTime {
        self.fs_time.1
    }

    pub fn get_last_change(&self) -> CTime {
        self.fs_time.2
    }

    pub fn get_creation_time(&self) -> CrTime {
        self.fs_time.3
    }

    pub fn get_kind(&self) -> Kind {
        self.kind
    }

    pub fn get_allpermission(&self) -> AllPerm {
        self.permission
    }

    //NOTE - Conserve que les 16 bits inférieur correspondant aux permissions classiques
    pub fn get_permission(&self) -> Permission {
        (self.permission & 0o7777) as u16
    }

    pub fn get_uid(&self) -> Uid {
        self.id.0
    }

    pub fn get_gid(&self) -> Gid {
        self.id.1
    }

    pub fn get_hard_link_num(&self) -> Nlink {
        self.nlink
    }

    pub fn get_device_id(&self) -> Rdev {
        self.rdev
    }

    pub fn get_flags(&self) -> Flags {
        self.flags
    }
}

use crate::pods::arbo::{FsEntry, Inode, Metadata};
use crate::pods::fs_interface::{self, FsInterface, SimpleFileType};
use crate::pods::whpath::WhPath;
use fuser::{
    BackgroundSession, FileAttr, FileType, Filesystem, MountOption, ReplyAttr, ReplyData,
    ReplyDirectory, ReplyEntry, Request, TimeOrNow,
};
use libc::{EIO, ENOENT};
use log::debug;
use openat::{Dir, SimpleType};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// NOTE - placeholders
const TTL: Duration = Duration::from_secs(1);

const MOUNT_DIR_ATTR: FileAttr = FileAttr {
    ino: 1,
    size: 0,
    blocks: 0,
    atime: UNIX_EPOCH, // 1970-01-01 00:00:00
    mtime: UNIX_EPOCH,
    ctime: UNIX_EPOCH,
    crtime: UNIX_EPOCH,
    kind: FileType::Directory,
    perm: 0o755,
    nlink: 2,
    uid: 501,
    gid: 20,
    rdev: 0,
    flags: 0,
    blksize: 512,
};

pub const TEMPLATE_FILE_ATTR: FileAttr = FileAttr {
    ino: 2,
    size: 2048,
    blocks: 1,
    atime: UNIX_EPOCH, // 1970-01-01 00:00:00
    mtime: UNIX_EPOCH,
    ctime: UNIX_EPOCH,
    crtime: UNIX_EPOCH,
    kind: FileType::RegularFile,
    perm: 0o777,
    nlink: 1,
    uid: 501,
    gid: 20,
    rdev: 0,
    flags: 0,
    blksize: 512,
};
// ^ placeholders

// const MIRROR_PTH: &str = "./wh_mirror/";

fn metadata_to_fileattr(meta: Metadata) -> FileAttr {
    FileAttr {
        ino: meta.ino,
        size: meta.size,
        blocks: meta.blocks,
        atime: meta.atime,
        mtime: meta.mtime,
        ctime: meta.ctime,
        crtime: meta.crtime,
        kind: meta.kind,
        perm: meta.perm,
        nlink: meta.nlink,
        uid: meta.uid,
        gid: meta.gid,
        rdev: meta.rdev,
        flags: meta.flags,
        blksize: meta.blksize,
    }
}

fn fileattr_to_metadata(attr: FileAttr) -> Metadata {
    Metadata {
        ino: attr.ino,
        size: attr.size,
        blocks: attr.blocks,
        atime: attr.atime,
        mtime: attr.mtime,
        ctime: attr.ctime,
        crtime: attr.crtime,
        kind: attr.kind,
        perm: attr.perm,
        nlink: attr.nlink,
        uid: attr.uid,
        gid: attr.gid,
        rdev: attr.rdev,
        flags: attr.flags,
        blksize: attr.blksize,
    }
}

pub struct FuseController {
    pub fs_interface: Arc<FsInterface>,
}

// NOTE for dev purpose while all metadata is not supported
fn inode_to_fuse_fileattr(inode: Inode) -> FileAttr {
    let mut attr = TEMPLATE_FILE_ATTR;
    attr.ino = inode.id;
    attr.kind = match inode.entry {
        FsEntry::Directory(_) => fuser::FileType::Directory,
        FsEntry::File(_) => fuser::FileType::RegularFile,
    };
    attr
}

// REVIEW - should later invest in proper error handling
impl Filesystem for FuseController {
    // READING

    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        debug!(
            "called lookup: {} > {}",
            parent,
            name.to_string_lossy().to_string()
        );

        match self
            .fs_interface
            .get_entry_from_name(parent, name.to_string_lossy().to_string())
        {
            Ok(inode) => {
                // debug!("yes entry for name {} - {}", parent, name.to_string_lossy().to_string());
                reply.entry(&TTL, &inode_to_fuse_fileattr(inode), 0);
            }
            Err(_) => {
                // debug!("no entry for name {} - {}", parent, name.to_string_lossy().to_string());
                reply.error(ENOENT);
            }
        };
    }

    fn getattr(&mut self, _req: &Request, ino: u64, _: Option<u64>, reply: ReplyAttr) {
        debug!("called getattr ino:{}", ino);
        let attrs = self.fs_interface.get_inode_attributes(ino);

        match attrs {
            Ok(attrs) => reply.attr(&TTL, &metadata_to_fileattr(attrs)),
            Err(err) => {
                log::error!("fuse_impl error: {:?}", err);
                reply.error(err.raw_os_error().unwrap_or(EIO))
            }
        }
    }

    fn setattr(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        mode: Option<u32>,
        uid: Option<u32>,
        gid: Option<u32>,
        size: Option<u64>,
        atime: Option<fuser::TimeOrNow>,
        mtime: Option<fuser::TimeOrNow>,
        ctime: Option<std::time::SystemTime>,
        fh: Option<u64>,
        crtime: Option<std::time::SystemTime>,
        chgtime: Option<std::time::SystemTime>,
        bkuptime: Option<std::time::SystemTime>,
        flags: Option<u32>,
        reply: ReplyAttr,
    ) {
        debug!("called setattr ino:{}", ino);
        let attrs = match self.fs_interface.get_inode_attributes(ino) {
            Ok(attrs) => Metadata {
                ino: attrs.ino,
                size: if let Some(size) = size {
                    size
                } else {
                    attrs.size
                },
                blocks: attrs.blocks,
                atime: if let Some(atime) = atime {
                    time_or_now_to_system_time(atime)
                } else {
                    attrs.atime
                },
                mtime: if let Some(mtime) = mtime {
                    time_or_now_to_system_time(mtime)
                } else {
                    attrs.mtime
                },
                ctime: if let Some(ctime) = ctime {
                    ctime
                } else {
                    attrs.ctime
                },
                crtime: if let Some(crtime) = crtime {
                    crtime
                } else {
                    attrs.crtime
                },
                kind: attrs.kind,
                perm: attrs.perm,
                nlink: attrs.nlink,
                uid: if let Some(uid) = uid { uid } else { attrs.uid },
                gid: if let Some(gid) = gid { gid } else { attrs.gid },
                rdev: attrs.rdev,
                blksize: attrs.blksize,
                flags: if let Some(flags) = flags {
                    flags
                } else {
                    attrs.flags
                },
            },
            Err(err) => {
                log::error!("fuse_impl::setattr: {:?}", err);
                reply.error(err.raw_os_error().unwrap_or(EIO));
                return;
            }
        };

        match self.fs_interface.set_inode_meta(ino, attrs.clone()) {
            Ok(_) => reply.attr(&TTL, &metadata_to_fileattr(attrs)),
            Err(err) => {
                log::error!("fuse_impl::setattr: {:?}", err);
                reply.error(err.raw_os_error().unwrap_or(EIO))
            }
        }
    }

    fn read(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        size: u32,
        _flags: i32,
        _lock: Option<u64>,
        reply: ReplyData,
    ) {
        debug!("called read ino:{}", ino);
        let content = self.fs_interface.read_file(
            ino,
            offset.try_into().expect("fuse_impl::read offset negative"),
            size.try_into().expect("fuse_impl::read size too large"),
        );

        match content {
            Ok(content) => reply.data(&content),
            Err(err) => {
                log::error!("fuse_impl error: {:?}", err);
                reply.error(err.raw_os_error().unwrap_or(EIO))
            }
        }
    }

    fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        debug!("called readdir ino:{} offset:{}", ino, offset);
        let entries = if let Ok(entries) = self.fs_interface.read_dir(ino) {
            entries
        } else {
            reply.error(ENOENT);
            return;
        };

        for (i, entry) in entries.into_iter().enumerate().skip(offset as usize) {
            debug!("....readdir entries : {:?}", entry);
            if reply.add(
                ino,
                // i + 1 means offset of the next entry
                (i + 1) as i64, // NOTE - in case of error, try i + 1
                entry.entry.get_filetype(),
                entry.name,
            ) {
                break;
            }
        }
        reply.ok();
    }

    // ^ READING

    // WRITING

    fn mknod(
        &mut self,
        _req: &Request<'_>,
        parent: u64,
        name: &OsStr,
        _mode: u32,
        _umask: u32,
        _rdev: u32,
        reply: ReplyEntry,
    ) {
        match self.fs_interface.make_inode(
            parent,
            name.to_string_lossy().to_string(),
            SimpleFileType::File,
        ) {
            Ok((id, _)) => {
                // creating metadata to return
                let mut new_attr = TEMPLATE_FILE_ATTR;
                new_attr.ino = id;
                new_attr.kind = FileType::RegularFile;
                new_attr.size = 0;

                reply.entry(&TTL, &new_attr, 0)
            }
            Err(err) => {
                log::error!("fuse_impl error: {:?}", err);
                reply.error(err.raw_os_error().unwrap_or(EIO))
            }
        }
    }

    fn mkdir(
        &mut self,
        _req: &Request<'_>,
        parent: u64,
        name: &OsStr,
        _mode: u32,
        _umask: u32,
        reply: ReplyEntry,
    ) {
        match self.fs_interface.make_inode(
            parent,
            name.to_string_lossy().to_string(),
            SimpleFileType::File,
        ) {
            Ok((id, _)) => {
                // creating metadata to return
                let mut new_attr = TEMPLATE_FILE_ATTR;
                new_attr.ino = id;
                new_attr.kind = FileType::Directory;
                new_attr.size = 0;

                reply.entry(&TTL, &new_attr, 0)
            }
            Err(err) => {
                log::error!("fuse_impl error: {:?}", err);
                reply.error(err.raw_os_error().unwrap_or(EIO))
            }
        }
    }

    fn unlink(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, reply: fuser::ReplyEmpty) {
        match self.fs_interface.fuse_remove_inode(parent, name) {
            Ok(()) => reply.ok(),
            Err(err) => {
                log::error!("fuse_impl error: {:?}", err);
                reply.error(err.raw_os_error().unwrap_or(EIO))
            }
        }
    }

    fn rmdir(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, reply: fuser::ReplyEmpty) {
        // should be only called on empty dirs ?
        match self.fs_interface.fuse_remove_inode(parent, name) {
            Ok(()) => reply.ok(),
            Err(err) => {
                log::error!("fuse_impl error: {:?}", err);
                reply.error(err.raw_os_error().unwrap_or(EIO))
            }
        }
    }

    fn rename(
        &mut self,
        _req: &Request<'_>,
        parent: u64,
        name: &OsStr,
        newparent: u64,
        newname: &OsStr,
        _flags: u32,
        reply: fuser::ReplyEmpty,
    ) {
        reply.error(ENOENT) // TODO
                            // let mut provider = self.provider.lock().unwrap();
                            // if let Some(()) = provider.rename(parent, name, newparent, newname) {
                            //     reply.ok()
                            // } else {
                            //     reply.error(ENOENT)
                            // }
    }

    fn write(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        _fh: u64,
        offset: i64,
        data: &[u8],
        _write_flags: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: fuser::ReplyWrite,
    ) {
        let offset = offset
            .try_into()
            .expect("fuser write: can't convert i64 to u64");

        match self.fs_interface.write(ino, data.to_vec(), offset) {
            Ok(written) => reply.written(
                written
                    .try_into()
                    .expect("fuser write: can't convert u64 to u32"),
            ),
            Err(err) => {
                log::error!("fuse_impl error: {:?}", err);
                reply.error(err.raw_os_error().unwrap_or(EIO))
            }
        }
    }

    // ^ WRITING

    fn create(
        &mut self,
        _req: &Request<'_>,
        parent: u64,
        name: &OsStr,
        mode: u32,
        umask: u32,
        flags: i32,
        reply: fuser::ReplyCreate,
    ) {
        debug!("CREATE called on parent {} for {:?}", parent, name);

        match self.fs_interface.make_inode(
            parent,
            name.to_string_lossy().to_string(),
            SimpleFileType::File,
        ) {
            Ok((id, _)) => {
                // creating metadata to return
                let mut new_attr = TEMPLATE_FILE_ATTR;
                new_attr.ino = id;
                new_attr.kind = FileType::RegularFile;
                new_attr.size = 0;

                reply.created(&TTL, &new_attr, 0, new_attr.ino, flags as u32);
            }
            Err(err) => {
                log::error!("fuse_impl::create : unable to create file. {:?}", err);
                {
                    log::error!("fuse_impl error: {:?}", err);
                    reply.error(err.raw_os_error().unwrap_or(EIO))
                }
            }
        }
    }

    fn open(&mut self, _req: &Request<'_>, ino: u64, flags: i32, reply: fuser::ReplyOpen) {
        log::error!("OPEN ON {}", ino);
        reply.opened(ino, flags as u32); // TODO - check flags ?
    }

    fn release(
        &mut self,
        _req: &Request<'_>,
        _ino: u64,
        _fh: u64,
        _flags: i32,
        _lock_owner: Option<u64>,
        _flush: bool,
        reply: fuser::ReplyEmpty,
    ) {
        log::error!("RELEASE CALLED");
        reply.ok();
    }
}

fn time_or_now_to_system_time(time: TimeOrNow) -> SystemTime {
    match time {
        TimeOrNow::Now => SystemTime::now(),
        TimeOrNow::SpecificTime(time) => time,
    }
}

pub fn mount_fuse(
    mount_point: &WhPath,
    fs_interface: Arc<FsInterface>,
) -> io::Result<BackgroundSession> {
    let options = vec![
        MountOption::RW,
        // MountOption::DefaultPermissions,
        MountOption::FSName("wormhole".to_string()),
    ];
    let ctrl = FuseController { fs_interface };

    fuser::spawn_mount2(ctrl, mount_point.to_string(), &options)
}

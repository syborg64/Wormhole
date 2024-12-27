use fuser::{
    BackgroundSession, FileAttr, FileType, Filesystem, MountOption, ReplyAttr, ReplyData,
    ReplyDirectory, ReplyEntry, Request,
};
use libc::{ENOENT, ENOSYS};
use log::debug;
use openat::{Dir, SimpleType};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, UNIX_EPOCH};
use crate::pods::fs_interface::{FsInterface, SimpleFileType};
use crate::providers::whpath::WhPath;

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
    size: 13,
    blocks: 1,
    atime: UNIX_EPOCH, // 1970-01-01 00:00:00
    mtime: UNIX_EPOCH,
    ctime: UNIX_EPOCH,
    crtime: UNIX_EPOCH,
    kind: FileType::RegularFile,
    perm: 0o644,
    nlink: 1,
    uid: 501,
    gid: 20,
    rdev: 0,
    flags: 0,
    blksize: 512,
};
// ^ placeholders

// const MIRROR_PTH: &str = "./wh_mirror/";

pub struct FuseController {
    pub fs_interface: Arc<FsInterface>,
}

fn index_folder_recursive(
    arbo: &mut FsIndex,
    inode: &mut u64,
    root_fd: &Dir,
    path: PathBuf,
) -> io::Result<()> {
    let errors_nb = root_fd
        .list_dir(&path)?
        .map(|entry| -> io::Result<()> {
            let entry = entry?;

            let name = entry.file_name();
            let stype = entry.simple_type().unwrap();

            let generated_path = path.join(name);

            let new_entry = match stype {
                SimpleType::Dir => FsEntry::Directory(generated_path.clone()),
                SimpleType::File => FsEntry::File(generated_path.clone(), vec![]),
                _ => return Ok(()),
            };
            arbo.insert(*inode, new_entry);
            println!("added entry to arbo {}:{:?}", inode, arbo.get(inode));
            *inode += 1;

            if stype == SimpleType::Dir {
                index_folder_recursive(arbo, inode, root_fd, generated_path)?;
            }
            Ok(())
        })
        .filter(|e| e.is_err())
        .collect::<Vec<Result<(), io::Error>>>()
        .len();
    println!(
        "indexing: {} error(s) in folder {}",
        errors_nb,
        path.display()
    );
    Ok(())
}

impl FuseController {
    fn index_folder(path: &Path) -> io::Result<(openat::Dir, FsIndex)> {
        let metal_mount_handle = Dir::open(path)?;
        let mut arbo: FsIndex = HashMap::new();
        let mut inode: u64 = 2;

        arbo.insert(1, FsEntry::Directory("./".into()));

        index_folder_recursive(&mut arbo, &mut inode, &metal_mount_handle, ".".into())?;
        Ok((metal_mount_handle, arbo))
    }
}

// REVIEW - should later invest in proper error handling
impl Filesystem for FuseController {
    // READING

    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        debug!("lookup is called {} {:?}", parent, name);
        let provider = self.provider.lock().unwrap();
        if let Ok(file_attr) = provider.fs_lookup(parent, name) {
            reply.entry(&TTL, &file_attr, 0)
        } else {
            reply.error(ENOENT)
        }
    }

    // TODO
    fn getattr(&mut self, _req: &Request, ino: u64, _: Option<u64>, reply: ReplyAttr) {
        debug!("getattr is called {}", ino);
        match ino {
            1 => reply.attr(&TTL, &MOUNT_DIR_ATTR),
            2 => reply.attr(&TTL, &TEMPLATE_FILE_ATTR),
            _ => reply.error(ENOENT),
        }
    }

    fn read(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        _size: u32,
        _flags: i32,
        _lock: Option<u64>,
        reply: ReplyData,
    ) {
        debug!("read is called");
        let provider = self.provider.lock().unwrap();
        if let Ok(content) = provider.read(ino) {
            reply.data(&content[offset as usize..])
        } else {
            reply.error(ENOENT);
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
        println!("readdir is called for ino {}", ino);
        let provider = self.provider.lock().unwrap();
        if let Ok(entries) = provider.fs_readdir(ino) {
            println!("....listing entries {:?}", entries);
            for (i, (ino, entry)) in entries.into_iter().enumerate().skip(offset as usize) {
                println!("....readdir entries : {:?}", entry);
                // i + 1 means the index of the next entry
                if reply.add(
                    ino,
                    (i + 1) as i64,
                    entry.get_filetype(),
                    entry.get_name().unwrap(),
                ) {
                    break;
                }
            }
            reply.ok()
        } else {
            println!("/!\\ readdir EONENT ");
            reply.error(ENOENT)
        }
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
        if let Ok((id, _)) = self.fs_interface.make_inode(
            parent,
            name.to_string_lossy().to_string(),
            SimpleFileType::File,
        ) {
            // creating metadata to return
            let mut new_attr = TEMPLATE_FILE_ATTR;
            new_attr.ino = id;
            new_attr.kind = FileType::RegularFile;
            new_attr.size = 0;

            reply.entry(&TTL, &new_attr, 0)
        } else {
            reply.error(ENOSYS)
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
        if let Ok((id, _)) = self.fs_interface.make_inode(
            parent,
            name.to_string_lossy().to_string(),
            SimpleFileType::File,
        ) {
            // creating metadata to return
            let mut new_attr = TEMPLATE_FILE_ATTR;
            new_attr.ino = id;
            new_attr.kind = FileType::Directory;
            new_attr.size = 0;

            reply.entry(&TTL, &new_attr, 0)
        } else {
            reply.error(ENOSYS)
        }
    }

    fn unlink(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, reply: fuser::ReplyEmpty) {
        if let Ok(()) = self.fs_interface.fuse_remove_inode(parent, name) {
            reply.ok()
        } else {
            reply.error(ENOENT)
        }
    }

    fn rmdir(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, reply: fuser::ReplyEmpty) {
        // should be only called on empty dirs ?
        let mut provider = self.provider.lock().unwrap();
        if let Ok(()) = provider.rmdir(parent, name) {
            reply.ok()
        } else {
            reply.error(ENOENT)
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
        // comment sont gérés les dossiers et sous fichiers ?
        let mut provider = self.provider.lock().unwrap();
        if let Some(()) = provider.rename(parent, name, newparent, newname) {
            reply.ok()
        } else {
            reply.error(ENOENT)
        }
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
        let provider = self.provider.lock().unwrap();
        if let Ok(written) = provider.write(ino, offset, data) {
            reply.written(written)
        } else {
            reply.error(ENOENT)
        }
    }

    // ^ WRITING
}

pub fn mount_fuse(
    mount_point: &WhPath,
    fs_interface: Arc<FsInterface>,
) -> io::Result<BackgroundSession> {
    let options = vec![MountOption::RW, MountOption::FSName("wormhole".to_string())];
    let ctrl = FuseController { fs_interface };

    fuser::spawn_mount2(ctrl, mount_point.to_string(), &options)
}

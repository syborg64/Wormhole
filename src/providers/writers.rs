use fuser::{FileAttr, FileType};
use log::info;
use std::{
    ffi::OsStr,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::PathBuf,
};

use crate::network::message::{self, Folder, NetworkMessage};

use super::{Provider, TEMPLATE_FILE_ATTR};

impl Provider {
    pub fn mkfile(&mut self, parent_ino: u64, name: &OsStr) -> io::Result<FileAttr> {
        match self.check_file_type(parent_ino, FileType::Directory) {
            Ok(_) => {
                let new_path =
                    PathBuf::from(self.mirror_path_from_inode(parent_ino).unwrap()).join(name);

                // bare metal file creation (on the mirror)
                match fs::File::create(&new_path) {
                    Ok(_) => (),
                    Err(e) => return Err(e),
                };

                // generation of the wormhole path
                let virt_path = self.virt_path_from_mirror_path(&new_path);

                // add entry to the index
                self.index
                    .insert(self.next_inode, (FileType::RegularFile, virt_path.clone()));
                self.tx
                    .send(NetworkMessage::File(message::File {
                        path: virt_path.into(),
                        file: [].to_vec(), // REVIEW - why this field ? useful ?
                        ino: self.next_inode,
                    }))
                    .expect("mkfile: unable to update modification on the network");

                // creating metadata to return
                let mut new_attr = TEMPLATE_FILE_ATTR;
                new_attr.ino = self.next_inode;
                new_attr.kind = FileType::RegularFile;
                new_attr.size = 0;
                self.next_inode += 1; // NOTE - ne jamais oublier d'incrémenter si besoin next_inode

                Ok(new_attr)
            }
            Err(e) => Err(e),
        }
    }

    pub fn mkdir(&mut self, parent_ino: u64, name: &OsStr) -> io::Result<FileAttr> {
        match self.check_file_type(parent_ino, FileType::Directory) {
            Ok(_) => {
                // generation of the real path (of the mirror)
                let new_path =
                    PathBuf::from(self.mirror_path_from_inode(parent_ino).unwrap()).join(name);

                // bare metal dir creation (on the mirror)
                match fs::create_dir(&new_path) {
                    Ok(_) => (),
                    Err(e) => return Err(e),
                };

                // generation of the wormhole path
                let virt_path = self.virt_path_from_mirror_path(&new_path);

                // adding path to the wormhole index
                self.index
                    .insert(self.next_inode, (FileType::Directory, virt_path.clone()));

                // send update to network
                self.tx
                    .send(NetworkMessage::NewFolder(Folder {
                        ino: self.next_inode,
                        path: virt_path.into(),
                    }))
                    .expect("mkdir: unable to update modification on the network");

                // creating metadata to return
                let mut new_attr = TEMPLATE_FILE_ATTR;
                new_attr.ino = self.next_inode;
                new_attr.kind = FileType::Directory;
                new_attr.size = 0;
                self.next_inode += 1; // NOTE - ne jamais oublier d'incrémenter si besoin next_inode

                println!("directory created: {:?}", new_path); //DEBUG

                Ok(new_attr)
            }
            Err(e) => Err(e),
        }
    }

    pub fn rmfile(&mut self, parent_ino: u64, name: &OsStr) -> io::Result<()> {
        let file = self.file_from_parent_ino_and_name(parent_ino, name)?;

        self.mirror_path_from_inode(file.0)
            .and_then(|file_path| fs::remove_file(file_path))
            .map(|_| {
                self.tx.send(NetworkMessage::Remove(file.0)).unwrap();
                self.index.remove(&file.0);
            })
    }

    pub fn rmdir(&mut self, parent_ino: u64, name: &OsStr) -> Option<()> {
        // should only be called on empty folders
        // if 404, not empty or file -> None
        Some(())
    }

    pub fn rename(
        &mut self,
        parent_ino: u64,
        name: &OsStr,
        newparent_ino: u64,
        newname: &OsStr,
    ) -> Option<()> {
        // pas clair de quand c'est appelé, si ça l'est sur des dossiers
        // non vides, go ignorer pour l'instant
        Some(())
    }

    // returns the writed size
    pub fn write(&self, ino: u64, _offset: i64, data: &[u8]) -> io::Result<u32> {
        match self.index.get(&ino) {
            Some((FileType::RegularFile, _)) => {
                let path = self.mirror_path_from_inode(ino)?;
                fs::write(path, data)?;
                self.tx
                    .send(NetworkMessage::Write(ino, data.to_owned()))
                    .unwrap();
                Ok(data.len() as u32)
            }
            Some(_) => Err(io::Error::new(io::ErrorKind::Other, "File not writable")),
            None => Err(io::Error::new(io::ErrorKind::NotFound, "File not found")),
        }
    }

    // RECEPTION
    // REVIEW - not yet refactored nor properly error handled
    pub fn new_folder(&mut self, ino: u64, path: PathBuf) {
        let real_path = PathBuf::from(self.local_source.clone()).join(&path);
        println!("Provider make new folder at: {:?}", real_path);
        fs::create_dir(&real_path).unwrap();
        self.index.insert(
            ino,
            (FileType::Directory, path.to_string_lossy().to_string()),
        );
    }

    pub fn new_file(&mut self, ino: u64, path: PathBuf) {
        println!("Provider make new file at ORIGINAL PATH: {:?}", path);
        let real_path = PathBuf::from(self.local_source.clone()).join(&path);
        println!("Provider make new file at: {:?}", real_path);
        fs::File::create(&real_path).unwrap();
        self.index.insert(
            ino,
            (FileType::RegularFile, path.to_string_lossy().to_string()),
        );
    }

    pub fn recpt_remove(&mut self, ino: u64) {
        let (file_type, path) = self.index.get(&ino).unwrap();
        let real_path = PathBuf::from(self.local_source.clone()).join(&path);
        println!("Provider remove object at: {:?}", real_path);
        match file_type {
            FileType::Directory => fs::remove_dir_all(&real_path).unwrap(),
            FileType::RegularFile => fs::remove_file(&real_path).unwrap(),
            _ => todo!(),
        }
        self.index.remove(&ino);
    }

    pub fn recpt_write(&mut self, ino: u64, content: Vec<u8>) {
        let (_, path) = self.index.get(&ino).unwrap();
        let real_path = PathBuf::from(self.local_source.clone()).join(&path);
        println!("Provider write to file at: {:?}", real_path);
        let mut file = OpenOptions::new()
            .read(true)
            .write(true) // <--------- this
            .create(true)
            .open(real_path)
            .unwrap();
        file.write_all(&content).unwrap();
    }

    // pub fn recpt_rename(&mut self, ino: u64, newparent_ino: u64, newname: &OsStr) {
    //     let (_, path) = self.index.get(&ino).unwrap();
    //     let real_path = PathBuf::from(self.local_source.clone()).join(&path);
    //     let real_path = PathBuf::from(self.local_source.clone()).join(&path);
    //     println!("Provider rename object at: {:?}", real_path);

    // }
}

use std::{ffi::OsStr, path::PathBuf};

use fuser::{FileAttr, FileType};

use super::{Provider, TEMPLATE_FILE_ATTR};

impl Provider {
    // Basically no error handling for the poc
    // If good : Some(requested_data)
    // Else : None

    pub fn mkfile(&mut self, parent_ino: u64, name: &OsStr) -> Option<FileAttr> {
        // should check that the parent exists and is a folder
        // return None if error
        if let Some(meta) = self.get_metadata(parent_ino) {
            if meta.kind == FileType::Directory {
                let new_name =
                    PathBuf::from(self.mirror_path_from_inode(parent_ino).unwrap()).join(name);

                // TODO - write the real file in the mirror

                // add entry to the index
                self.index.insert(
                    self.next_inode,
                    (
                        FileType::RegularFile,
                        new_name.to_string_lossy().to_string(),
                    ),
                );
                let mut new_attr = TEMPLATE_FILE_ATTR;
                new_attr.ino = self.next_inode;
                new_attr.kind = FileType::RegularFile;
                new_attr.size = 0;
                self.next_inode += 1; // NOTE - ne jamais oublier d'incrémenter si besoin next_inode

                Some(new_attr)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn mkdir(&mut self, parent_ino: u64, name: &OsStr) -> Option<FileAttr> {
        // should check that the parent exists and is a folder
        // return None if error
        Some(TEMPLATE_FILE_ATTR)
    }

    pub fn rmfile(&mut self, parent_ino: u64, name: &OsStr) -> Option<()> {
        // should only be called on files and not folders
        // if 404 or Folder -> None
        Some(())
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
        // non vides, go ignorer et pas tester à la démo
        Some(())
    }

    pub fn write(&self, ino: u64, offset: i64, data: &[u8]) -> Option<u32> {
        // returns the writed size
        Some(0)
    }
}

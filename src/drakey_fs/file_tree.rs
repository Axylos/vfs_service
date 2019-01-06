use time::Timespec;
use fuse::{FileAttr};
use std::collections;
use super::inode;

pub struct FileTree {
    files: collections::HashMap<u64, inode::Inode>,
    ino_ctr: u64
}

impl FileTree {
    pub fn get_file_attrs(&self, _ino: &u64) -> Option<(&Timespec, &FileAttr)> {
        None
    } 

    pub fn new() -> FileTree {
        FileTree {
            files: collections::HashMap::<u64, inode::Inode>::new(),
            ino_ctr: 0
        }
    }
}

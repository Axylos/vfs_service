use std::collections;
use crate::drakey_fs::inode;
use std::ffi;

#[derive(Debug)]
pub struct FileNode {
    content: Vec<u8>
}

impl FileNode {
    pub fn new() -> FileNode {
        FileNode {
            content: vec![]
        }
    }
}

#[derive(Debug)]
pub struct DirNode {
    pub children: collections::BTreeSet<u64>,
    pub name_map: collections::HashMap<ffi::OsString, u64>
}

impl DirNode {
    pub fn new() -> DirNode {
        DirNode {
            children: collections::BTreeSet::<u64>::new(),
            name_map: collections::HashMap::<ffi::OsString, u64>::new()
        }
    }

    pub fn lookup_child(&self, path: &ffi::OsStr) -> Option<&u64> {
        println!("lookup child called");
        self.name_map.get(path)
    }

    pub fn add_child(&mut self, ino: &u64, path: &ffi::OsStr) -> Option<()> {
        self.children.insert(*ino);
        self.name_map.insert(path.to_os_string(), *ino);
        Some(())
    }

}

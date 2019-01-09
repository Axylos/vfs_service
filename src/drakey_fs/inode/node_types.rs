use std::collections;
use std::ffi;

pub trait DrakeyFile {
    fn write(&mut self, buf: &[u8]) -> u32;
    fn get_content(&self) -> &[u8];
}

pub trait DrakeyDir {
    fn get_children(&self) -> &collections::BTreeSet<u64>;
    fn lookup_path(&self, path: &ffi::OsStr) -> Option<&u64>;
    fn add_child(&mut self, child: &u64, path: &ffi::OsStr) -> Option<()>;
}

impl DrakeyFile for FileNode {
    fn write(&mut self, buf: &[u8]) -> u32 {
        self.content = buf.to_vec();
        std::mem::size_of_val(buf) as u32
    }

    fn get_content(&self) -> &[u8] {
        &self.content[..]
    }
}

impl std::fmt::Debug for DrakeyFile + Send {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "here is a thing")
    }
}

impl std::fmt::Debug for DrakeyDir + Send {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "here is a thing")
    }
}

impl DrakeyDir for DirNode {
    fn get_children(&self) -> &collections::BTreeSet<u64> {
        &self.children
    }

    fn lookup_path(&self, path: &ffi::OsStr) -> Option<&u64> {
        self.name_map.get(path)
    }

    fn add_child(&mut self, child: &u64, path: &ffi::OsStr) -> Option<()> {
        self.children.insert(*child);
        self.name_map.insert(path.to_os_string(), *child);
        Some(())
    }
}

#[derive(Debug)]
pub struct FileNode {
    content: Vec<u8>,
}

impl FileNode {
    pub fn new() -> FileNode {
        FileNode { content: vec![] }
    }
}

#[derive(Debug)]
pub struct DirNode {
    pub children: collections::BTreeSet<u64>,
    pub name_map: collections::HashMap<ffi::OsString, u64>,
}

impl DirNode {
    pub fn new() -> DirNode {
        DirNode {
            children: collections::BTreeSet::<u64>::new(),
            name_map: collections::HashMap::<ffi::OsString, u64>::new(),
        }
    }

    pub fn _lookup_child(&self, path: &ffi::OsStr) -> Option<&u64> {
        println!("lookup child called");
        self.name_map.get(path)
    }
}

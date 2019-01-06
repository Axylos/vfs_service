/*
use std::collections;
use std::path;
use time::Timespec;
use fuse::{FileAttr};
use std::ffi::OsStr;
use std::ffi::OsString;

pub struct NodeFileData {
    pub content: Vec<u8>
}

#[derive(Debug)]
pub struct NodeDirData {
    pub children: collections::BTreeSet<u64>,
    pub name_map: collections::HashMap<OsString, u64>,
}

trait NodeDir {
    fn get_children(&self) -> collections::BTreeSet<u64>;
    fn get_names(&self) -> collections::HashMap<OsString, u64>;
}

impl NodeDir for NodeDirData {
    fn get_children(&self) -> collections::BTreeSet<u64> {
        self.children

    }

    fn get_names(&self) -> collections::HashMap<OsString, u64> {
        self.name_map
    }
}

trait NodeFile {
    fn get_content(&self) -> Vec<u8>;
    fn clear(&mut self);
}

impl std::fmt::Debug for NodeFile + Send {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result{
        write!(f, "here is a thing")
    }
}
    
impl std::fmt::Debug for NodeDir + Send {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result{
        write!(f, "here is a thing")
    }
}

impl NodeFile for NodeFileData {
    fn get_content(&self) -> Vec<u8> {
       self.content 
    }

    fn clear(&mut self) {
        self.content = vec![];
    }
}

#[derive(Debug)]
pub enum NodeData {
    File(Box<NodeFile + Send>),
    Dir(Box<NodeDir + Send>)
}

unsafe impl Send for NodeFileData {}
unsafe impl Send for NodeDirData {}

#[derive(Debug)]
pub struct Inode {
    pub id: u64,
    pub file_data: FileAttr,
    pub ttl: Timespec,
    pub data: NodeData,
    pub xattr: collections::HashMap<OsString, String>,
    pub path: path::PathBuf,
}

impl Inode {
    pub fn access(&mut self) {
        let now = time::now().to_timespec();
        self.file_data.atime = now;
    }

    pub fn new(id: u64, data: NodeData, name: &OsStr) -> Inode {

    let ts = time::now().to_timespec();
    let ino = 1;
    let attrs = FileAttr {
        ino,
        kind: fuse::FileType::Directory,
        nlink: 0,
        perm: 0o777,
        rdev: 0,
        size: 0,
        atime: ts,
        ctime: ts,
        crtime: ts,
        mtime: ts,
        blocks: 100,
        flags: 0,
        gid: 0,
        uid: 501,
    };


        let ttl = time::now().to_timespec() + time::Duration::hours(10);
        let path = path::PathBuf::from(name);
        Inode {
            id,
            path,
            file_data: attrs,
            data,
            ttl,
            xattr: collections::HashMap::new(),
        }
    }

    pub fn add(&mut self, id: u64, name: std::ffi::OsString) {
        self.get_children();
        self.data.children.insert(id);
        self.data.name_map.insert(name, id);
    }
}
    */


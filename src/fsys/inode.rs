use std::collections;
use std::path;
use std::ffi::{OsStr, OsString};
use fuse::{FileAttr, FileType};
use time::Timespec;

#[derive(Debug)]
pub struct FileNode {
    pub content: Vec<u8>,
}

impl FileNode {
    pub fn new() -> FileNode {
        FileNode {
            content: Vec::new()
        }
    }
}

#[derive(Debug)]
pub struct DirNode {
    pub children: collections::BTreeSet<u64>,
    pub name_map: collections::HashMap<OsString, u64>,
}

#[derive(Debug)]
pub enum NodeData {
    File(FileNode),
    Dir(DirNode),
}


#[derive(Debug)]
pub struct Inode {
    pub id: u64,
    pub ttl: Timespec,
    pub data: NodeData,
    pub attr: FileAttr,
    pub xattr: collections::HashMap<OsString, String>,
    pub path: path::PathBuf,
}

impl Inode {
    pub fn new(id: u64, data: NodeData, name: &OsStr, uid: u32, gid: u32) -> Inode {
        let ttl = time::now().to_timespec() + time::Duration::hours(24);
        let path = path::PathBuf::from(name);
        let mut attr = build_dummy_file();
        attr.uid = uid;
        attr.gid = gid;
        Inode {
            id,
            attr,
            path,
            data,
            ttl,
            xattr: collections::HashMap::new(),
        }

    }
}

fn build_dummy_file() -> FileAttr {
    let ts = time::now().to_timespec();
    let ino = 1;
    FileAttr {
        ino,
        kind: FileType::Directory,
        nlink: 0,
        perm: 0755,
        rdev: 0,
        size: 0,
        atime: ts,
        ctime: ts,
        crtime: ts,
        mtime: ts,
        blocks: 100,
        flags: 0,
        gid: 0,
        uid: 0,
    }
}


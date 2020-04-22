use std::collections;
use std::ffi::{OsStr, OsString};
use std::path;
use time;
use time::Timespec;

extern crate file_node;
use file_node::NodeData;
use fuse::{FileAttr, FileType};

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
    pub fn new(id: u64, data: NodeData, name: &OsStr, _uid: u32, _gid: u32) -> Inode {
        let ttl = Timespec::new(1, 0);
        let path = path::PathBuf::from(name);
        let kind = match data {
            NodeData::File(_) => FileType::RegularFile,
            NodeData::RegularDir(_) => FileType::Directory,
            NodeData::ServiceDir(_) => FileType::Directory,
        };
        let mut attr = build_dummy_file(kind);
        attr.uid = 501;
        attr.gid = 20;
        Inode {
            id,
            attr,
            path,
            data,
            ttl,
            xattr: collections::HashMap::new(),
        }
    }

    pub fn access(&mut self) {
        let now = time::get_time();
        self.attr.atime = now;
    }
}

fn build_dummy_file(kind: FileType) -> FileAttr {
    let ts = time::get_time();
    let ino = 1;
    FileAttr {
        ino,
        kind,
        nlink: 2,
        perm: 0o755,
        rdev: 0,
        size: 0,
        atime: ts,
        ctime: ts,
        crtime: ts,
        mtime: ts,
        blocks: 0,
        flags: 0,
        gid: 0,
        uid: 0,
    }
}

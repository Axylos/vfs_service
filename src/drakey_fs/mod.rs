mod inode;

mod file_tree;

use log;
use libc::{ENOENT};
use std::ffi;
use fuse::{ Filesystem, Request, ReplyEntry, ReplyAttr };

pub struct Fs {
    file_tree: file_tree::FileTree,
}

impl Fs {
    pub fn new() -> Fs {
        Fs { 
            file_tree: file_tree::FileTree::new()
        }
    }
}

impl Filesystem for Fs {
    fn mkdir(
        &mut self, 
        _req: &Request, 
        parent: u64, 
        name: &ffi::OsStr, 
        mode: u32, 
        _reply: ReplyEntry
    ) {
        log::error!("make dir parent={} name={:?} mode={}", parent, name, mode);
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        log::error!("getattr: {}", ino);
        match self.file_tree.get_file_attrs(&ino) {
            Some((ttl, attr)) => reply.attr(ttl, attr),
            None => reply.error(ENOENT)
        }
    }
}

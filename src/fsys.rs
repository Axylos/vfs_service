use crate::file_tree;
use fuse::{FileAttr, FileType, Filesystem, ReplyAttr, Request, ReplyDirectory};
use libc::{ENOENT, ENOSYS};
use time::Timespec;
use std::path::Path;

pub struct Fs {
    file_tree: file_tree::FileMap,
}

impl Fs {
    pub fn new() -> Fs {
        Fs {
            file_tree: file_tree::FileMap::new(),
        }
    }
}

impl Filesystem for Fs {
    fn init(&mut self, _req: &Request) -> Result<(), i32> {
        println!("up and running");

        Ok(())
    }
    fn readdir(&mut self, _req: &Request, ino: u64, fh: u64, offset: i64, mut reply: ReplyDirectory) {
        match self.file_tree.get(&ino) {
            Some(node) => {
                if offset == 0 {
                    reply.add(1, 0, FileType::Directory, &Path::new("."));
                    reply.add(1, 1, FileType::Directory, &Path::new(".."));
                    reply.ok();
                }
            },
            None => reply.error(ENOENT)
        };

    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        match self.file_tree.get(&ino) {
            Some(file) => {
                let ttl = Timespec::new(1, 0);
                let ts = Timespec::new(0, 0);
                reply.attr(&ttl, &file.data.file_data);
            }
            None => reply.error(ENOENT),
        }
    }
}

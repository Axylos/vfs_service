use crate::file_tree;
use fuse::{FileAttr, FileType, Filesystem, ReplyAttr, ReplyCreate, ReplyDirectory, Request};
use libc::{ENOENT, ENOSYS};
use std::ffi::OsStr;
use std::path::Path;
use time::Timespec;

pub struct Fs {
    file_tree: file_tree::FileMap<'static>,
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

    fn create(
        &mut self,
        _req: &Request,
        parent: u64,
        name: &OsStr,
        mode: u32,
        flags: u32,
        reply: ReplyCreate,
    ) {
        let now = time::now().to_timespec();
    }
    fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        match self.file_tree.get(&ino) {
            Some(node) => {
                if offset == 0 {
                    reply.add(1, 0, FileType::Directory, &Path::new("."));
                    reply.add(1, 1, FileType::Directory, &Path::new(".."));
                    reply.ok();
                }
            }
            None => reply.error(ENOENT),
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

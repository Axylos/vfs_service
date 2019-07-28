use fuse::{
    FileType,
    Filesystem,
    ReplyAttr,
    ReplyCreate,
    ReplyData, ReplyDirectory, ReplyEmpty,
    ReplyEntry, ReplyOpen, ReplyWrite, ReplyXattr,
    Request,
};

use crate::fsys::fstore::{FileStore};

use libc::{ENOENT};

pub struct Fs {
    store: FileStore
}

impl Fs {
    pub fn new() -> Fs {
        Fs {
            store: FileStore::new()
        }
    }
}

impl Filesystem for Fs {
    fn init(&mut self, _req: &Request) -> Result<(), i32> {
        log::debug!("up and running");

        Ok(())
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        log::debug!("get attr {:?} {:?}", _req, ino);
        reply.error(ENOENT);

    }
}

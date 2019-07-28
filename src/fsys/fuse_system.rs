use std::ffi::{OsStr};
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
use time::Timespec;

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

    fn access(&mut self, _req: &Request, ino: u64, mask: u32, reply: ReplyEmpty) {
        log::error!("access: {} {}", ino, mask);
                reply.ok();
    }

    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        log::error!("called lookup");
    }

    fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        log::error!("readdir: {}, {}, {}", ino, fh, offset);
    }

    fn setattr(
        &mut self,
        _req: &Request,
        ino: u64,
        mode: Option<u32>,
        gid: Option<u32>,
        uid: Option<u32>,
        size: Option<u64>,
        atime: Option<Timespec>,
        mtime: Option<Timespec>,
        fh: Option<u64>,
        crtime: Option<Timespec>,
        chgtime: Option<Timespec>,
        bkuptime: Option<Timespec>,
        flags: Option<u32>,
        reply: ReplyAttr,
    ) {
        log::error!(
            "set attr: ino={} mode={:?} gid={:?} \
             uid={:?} size={:?} atime{:?} \
             mtime={:?} fh={:?} crtime={:?} \
             chgtime={:?} bkuptime={:?} flags={:?}",
            ino,
            mode,
            gid,
            uid,
            size,
            atime,
            mtime,
            fh,
            crtime,
            chgtime,
            bkuptime,
            flags
            );
    }

    fn flush(&mut self, _req: &Request, ino: u64, fh: u64, lock_owner: u64, reply: ReplyEmpty) {
        log::error!("flush: {}, {}, {}", ino, fh, lock_owner);
        reply.ok();

    }

    fn release(
        &mut self,
        _req: &Request,
        ino: u64,
        fh: u64,
        flags: u32,
        lock_owner: u64,
        flush: bool,
        reply: ReplyEmpty,

        ) {
        log::error!("release {} {} {} {} {}", ino, fh, flags, lock_owner, flush);
        reply.ok();
    }

    fn opendir(&mut self, _req: &Request, ino: u64, flags: u32, reply: ReplyOpen) {
        log::error!("opendir: {}, {}", ino, flags);
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        log::error!("get attr {:?} {:?}", _req, ino);
        match self.store.get(&ino) {
            Some(file) => {
                log::debug!("found filez: {:?} {:?}", file.attr, file.ttl);
                reply.attr(&file.ttl, &file.attr);
            }
            None => {
                log::error!("none found!");
                reply.error(ENOENT);
            }
        }

    }
}

use super::file_tree;
use fuse::{
    Filesystem, ReplyAttr, ReplyCreate, ReplyData, ReplyDirectory, ReplyEmpty, ReplyEntry,
    ReplyOpen, ReplyWrite, Request,
};
use libc::ENOENT;
use log;
use std::{ffi, path};

pub struct Fs {
    file_tree: file_tree::FileTree,
}

impl Fs {
    pub fn new() -> Fs {
        Fs {
            file_tree: file_tree::FileTree::new(),
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
        reply: ReplyEntry,
    ) {
        log::error!("make dir parent={} name={:?} mode={}", parent, name, mode);
        match self.file_tree.add_dir(&parent, name) {
            Ok(f) => {
                log::error!("everything ok, {:?} {:?} {}", &f.ttl, &f.file_data, f.id);
                reply.entry(&f.ttl, &f.file_data, f.id)
            }
            Err(i) => {
                log::error!("wat");
                reply.error(i)
            }
        }
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        log::error!("getattr: {}", ino);
        match self.file_tree.get_file_attrs(&ino) {
            Some((ttl, attr)) => reply.attr(ttl, attr),
            None => reply.error(ENOENT),
        }
    }

    fn lookup(&mut self, _req: &Request, parent: u64, name: &ffi::OsStr, reply: ReplyEntry) {
        log::error!("lookup: {}, {:?}", parent, name);
        // TODO: Check to see if the file exists and reply with info
        match self.file_tree.lookup_path(&parent, name) {
            Some(file) => {
                log::error!("found file: {:?}", file);
                let _data = &file.data;

                // the generation final arg needs to be the id.
                // seems similar to fh wtf
                reply.entry(&file.ttl, &file.file_data, file.id);
            }
            None => {
                log::error!("no file found in lookup");
                reply.error(ENOENT);
            }
        }
    }

    fn open(&mut self, _req: &Request, ino: u64, flags: u32, reply: ReplyOpen) {
        log::error!("open: ino={} flags={}", ino, flags);
        reply.opened(ino, flags);
    }

    fn write(
        &mut self,
        _req: &Request,
        ino: u64,
        fh: u64,
        offset: i64,
        data: &[u8],
        flags: u32,
        reply: ReplyWrite,
    ) {
        log::error!("write: {} {} {} {:?} {}", ino, fh, offset, data, flags);
        let w_size = std::mem::size_of_val(data) as u32;
        log::error!("write size: {}", w_size as u32);
        let size = self.file_tree.write_file(ino, data, flags, offset);
        log::error!("size={:?}", size);
        // must return exact same size as data that was requested to be written
        // or else stupid io invalid arg error or something happens
        // really stupid
        reply.written(w_size)
    }

    fn opendir(&mut self, _req: &Request, ino: u64, flags: u32, reply: ReplyOpen) {
        log::error!("opendir: {}, {}", ino, flags);
        reply.opened(ino, flags);
    }

    fn setattr(
        &mut self,
        _req: &Request,
        ino: u64,
        mode: Option<u32>,
        gid: Option<u32>,
        uid: Option<u32>,
        size: Option<u64>,
        atime: Option<time::Timespec>,
        mtime: Option<time::Timespec>,
        fh: Option<u64>,
        crtime: Option<time::Timespec>,
        chgtime: Option<time::Timespec>,
        bkuptime: Option<time::Timespec>,
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
        let now = time::now().to_timespec();
        if let Some(_) = size {
            //self.file_tree.clear_file(&ino);
        }

        match self.file_tree.lookup(&ino) {
            Some(file) => reply.attr(&now, &file.file_data),
            None => reply.error(ENOENT),
        }
    }

    fn read(
        &mut self,
        _req: &Request,
        ino: u64,
        fh: u64,
        offset: i64,
        size: u32,
        reply: ReplyData,
    ) {
        log::error!("read: {}, {}, {}, size={}", ino, fh, offset, size);
        match self.file_tree.get_file(&ino) {
            Some(f) => {
                let data = &f.get_content();
                // this is just a stupid way
                // to push a value into a byte slice
                // has to be a better way
                let v = data.to_vec();
                //disable for now seems to work on os x
                //v.push(EOF);

                let o = offset as usize;
                let d: &[u8] = &v[o..];
                reply.data(d)
            }
            None => reply.error(ENOENT),
        }
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
        match self.file_tree.read_dir(&ino) {
            Some(children) => {
                log::error!("children: {:?}", children);
                let mut idx: u64 = 0;
                let offset = offset as u64;
                if offset > 2 {
                    idx = offset - 2;
                }

                let len = children.len() as u64;
                if offset < len + 1 as u64 {
                    reply.add(1, 0, fuse::FileType::Directory, &path::Path::new("."));
                    reply.add(1, 1, fuse::FileType::Directory, &path::Path::new(".."));
                    let mut ctr = 2 + offset as i64;
                    for id in children.range(idx..) {
                        match self.file_tree.lookup(&id) {
                            Some(f) => {
                                reply.add(f.id, ctr, f.file_data.kind, &f.path);
                                ctr += 1;
                                log::error!("{:?}", f);
                            }
                            None => log::error!(
                                "dangling child reference: parent={} child={}",
                                &ino,
                                &id
                            ),
                        }
                    }
                }
                reply.ok();
            }
            None => {
                log::error!("file not found ino={}", ino);
                reply.error(ENOENT);
            }
        };
    }

    fn access(&mut self, req: &Request, ino: u64, mask: u32, reply: ReplyEmpty) {
        log::error!("access: {} {} {:?}", ino, mask, req);
        match self.file_tree.access_file(&ino) {
            Ok(()) => {
                reply.ok();
            }
            Err(_) => reply.error(ENOENT),
        }
    }

    fn create(
        &mut self,
        _req: &Request,
        parent: u64,
        name: &ffi::OsStr,
        _mode: u32,
        flags: u32,
        reply: ReplyCreate,
    ) {
        log::error!("touching file: {} {:?}", parent, name);
        match self.file_tree.add_file(&parent, name) {
            Ok(f) => reply.created(&f.ttl, &f.file_data, f.id, f.id, flags),
            Err(i) => reply.error(i),
        }
    }
}

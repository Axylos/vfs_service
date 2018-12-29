use crate::file_tree;
use time::Timespec;
use libc::{ENOENT, ENOSYS};
use fuse::{
    Request,
    FileAttr,
    Filesystem,
    ReplyAttr,
    FileType
};

pub struct Fs {
    file_tree: file_tree::FileMap
}

impl Fs {
    pub fn new() -> Fs {
        Fs { file_tree: file_tree::FileMap::new() }
    }
}

impl Filesystem for Fs {
    fn init(&mut self, _req: &Request) -> Result<(), i32> {
        println!("up and running");

        Ok(())
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {

        match self.file_tree.get(&ino) {
            Some(file) => {
                let ttl = Timespec::new(1, 0);
                let ts = Timespec::new(0, 0);
                reply.attr(&ttl, &file.data.file_data);
            }
            None => reply.error(ENOENT)
        }
    }
}

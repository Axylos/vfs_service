use super::inode;
use std::collections;
use std::ffi;
mod dir_ops;
mod file_ops;

#[derive(Debug)]
pub struct FileTree {
    files: collections::HashMap<u64, inode::Inode>,
    ino_ctr: u64,
}

impl FileTree {
    pub fn new() -> FileTree {
        let mut tree = FileTree {
            files: collections::HashMap::<u64, inode::Inode>::new(),
            ino_ctr: 2, // init at 2 to ignore root
        };

        let name = ffi::OsStr::new("root");
        let root = inode::Inode::new_dir(&1, name);
        tree.files.insert(1, root);

        tree
    }

    #[cfg(test)]
    pub fn get_file_count(&self) -> u64 {
        self.files.len() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_file_tree() {
        let f = FileTree::new();
        assert_eq!(f.get_file_count(), 1);
    }
}

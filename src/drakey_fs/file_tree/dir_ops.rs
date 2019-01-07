use super::FileTree;
use crate::drakey_fs::inode;
use crate::drakey_fs::inode::{node_data::NodeData, node_types};
use libc::ENOENT;
use std::boxed;
use std::collections;
use std::ffi;

impl FileTree {
    fn add_node(
        &mut self,
        ino: &u64,
        node: inode::Inode,
        path: &ffi::OsStr,
    ) -> Result<&inode::Inode, i32> {
        match self.get_dir(ino) {
            Some(parent) => match parent.lookup_path(path) {
                Some(_) => {
                    println!("wat");
                    Err(1)
                }
                None => {
                    let id = self.ino_ctr;
                    self.ino_ctr += 1;

                    self.files.insert(id, node);
                    self.add_child_to_parent(ino, &id, path);
                    match self.lookup(&id) {
                        Some(node) => Ok(node),
                        _ => {
                            log::error!("oopsie");
                            Err(ENOENT)
                        }
                    }
                }
            },
            None => Err(ENOENT),
        }
    }

    fn add_child_to_parent(&mut self, ino: &u64, child: &u64, path: &ffi::OsStr) -> Option<()> {
        // TODO: should prob add better error handling
        self.files
            .entry(*ino)
            .and_modify(|node| match &mut node.data {
                NodeData::Dir(parent) => {
                    parent.add_child(child, path);
                }
                _ => log::error!("not a dir: {}", ino),
            });

        Some(())
    }

    pub fn read_dir(&self, ino: &u64) -> Option<&collections::BTreeSet<u64>> {
        log::error!("read dir {}", ino);
        let f = self.lookup(ino)?;
        match &f.data {
            NodeData::Dir(dir) => Some(&dir.get_children()),
            _ => None,
        }
    }

    pub fn add_dir(&mut self, parent: &u64, path: &ffi::OsStr) -> Result<&inode::Inode, i32> {
        let node = inode::Inode::new_dir(&self.ino_ctr, path);
        log::error!("new dir node: {:?}", node);
        self.add_node(parent, node, path)
    }

    pub fn add_file(&mut self, parent: &u64, path: &ffi::OsStr) -> Result<&inode::Inode, i32> {
        log::error!("adding file: {} {:?}", parent, path);
        let node = inode::Inode::new_file(&self.ino_ctr, path);
        self.add_node(parent, node, path)
    }

    pub fn lookup(&self, ino: &u64) -> Option<&inode::Inode> {
        self.files.get(ino)
    }

    fn resolve_path(&mut self, parent: &u64, path: &ffi::OsStr) -> Option<&u64> {
        let node = self.files.get_mut(parent)?;
        match &mut node.data {
            NodeData::Dir(dir) => dir.lookup_path(path),
            _ => None,
        }
    }

    fn get_dir(&self, ino: &u64) -> Option<&boxed::Box<node_types::DrakeyDir + Send>> {
        let node = self.lookup(ino)?;
        match &node.data {
            NodeData::Dir(dir) => Some(dir),
            _ => None,
        }
    }

    pub fn lookup_path(&self, ino: &u64, path: &ffi::OsStr) -> Option<&inode::Inode> {
        let parent = self.get_dir(ino)?;
        let child_id = parent.lookup_path(path)?;
        self.lookup(child_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_file() {
        let f = FileTree::new();
        match f.lookup(&1) {
            Some(root) => assert_eq!(root.id, 1),
            None => panic!("root does not exist"),
        }
    }

    #[test]
    fn lookup_invalid() {
        let f = FileTree::new();
        match f.lookup(&2) {
            Some(_invalid) => panic!("invalid ino"),
            None => assert!(true),
        }
    }

    #[test]
    fn add_dir() {
        let mut f = FileTree::new();
        match f.add_dir(&1, ffi::OsStr::new("thingy")) {
            Ok(dir) => assert_eq!(dir.id, 2),
            _ => panic!("mkdir failed"),
        }
    }

    #[test]
    fn add_invalid_dir() {
        let mut f = FileTree::new();
        f.add_dir(&0, ffi::OsStr::new("thingy"));
        match f.add_dir(&0, ffi::OsStr::new("thingy")) {
            Ok(_id) => panic!("duplicate dir created"),
            _ => assert!(true),
        }
    }
}

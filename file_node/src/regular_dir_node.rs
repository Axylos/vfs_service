use crate::node_data::DirNode;
use std::collections;
use std::ffi::{OsStr, OsString};

#[derive(Debug, Clone)]
pub struct RegularDirNode {
    pub children: collections::BTreeSet<u64>,
    pub name_map: collections::HashMap<OsString, u64>,
}

impl RegularDirNode {
    pub fn new() -> RegularDirNode {
        RegularDirNode {
            children: collections::BTreeSet::new(),
            name_map: collections::HashMap::new(),
        }
    }
}

impl DirNode for RegularDirNode {
    fn remove(&mut self, id: &u64, name: &OsStr) {
        self.children.remove(id);
        self.name_map.remove(name);
    }

    fn add(&mut self, id: u64, name: std::ffi::OsString) {
        self.children.insert(id);
        self.name_map.insert(name, id);
    }
}

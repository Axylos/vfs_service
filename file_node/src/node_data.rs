use std::ffi::OsStr;
use crate::file_node::FileNode;
use crate::regular_dir_node::RegularDirNode;
use crate::service_node::ServiceDirNode;

pub trait DirNode {
    fn remove(&mut self, id: &u64, name: &OsStr);
    fn add(&mut self, id: u64, name: std::ffi::OsString);
}

#[derive(Debug)]
pub enum NodeData {
    File(FileNode),
    RegularDir(RegularDirNode),
    ServiceDir(ServiceDirNode)
}

pub fn gen_dir_node() -> NodeData {
    let dir = RegularDirNode::new();
    NodeData::RegularDir(dir)
}

pub fn gen_file_node() -> NodeData {
    let file = FileNode::new();
    NodeData::File(file)
}

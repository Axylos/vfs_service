use super::node_types;

#[derive(Debug)]
pub enum NodeData {
    File(Box<node_types::FileNode>),
    Dir(Box<node_types::DirNode>)
}

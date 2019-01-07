use super::node_types;

#[derive(Debug)]
pub enum NodeData {
    File(Box<node_types::DrakeyFile + Send>),
    Dir(Box<node_types::DrakeyDir + Send>)
}

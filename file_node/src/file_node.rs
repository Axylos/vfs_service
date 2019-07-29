#[derive(Debug, Clone)]
pub struct FileNode {
    pub content: Vec<u8>,
}

impl FileNode {
    pub fn new() -> FileNode {
        FileNode {
            content: Vec::new()
        }
    }
}


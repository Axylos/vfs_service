use std::collections;
use std::ffi::{OsString};
use std::fmt;

#[derive(Debug, Clone)]
pub struct BundleServiceDirNode {
    pub children: collections::BTreeSet<u64>,
    pub name_map: collections::HashMap<OsString, u64>,
}

pub trait SingleService {
    fn fetch_data(&self, query: Option<&str>) -> Vec<String>;
    fn get_name(&self) -> String;
}

impl std::fmt::Debug for SingleService + 'static + Send {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "it worked")
    }
}

#[derive(Debug)]
pub struct ServiceDirNode {
    pub children: collections::BTreeSet<u64>,
    pub name_map: collections::HashMap<OsString, u64>,
    pub service: Box<dyn SingleService + Send>
}

impl ServiceDirNode {
    pub fn new(service: Box<dyn SingleService + Send>) -> ServiceDirNode {
        ServiceDirNode {
            children: collections::BTreeSet::new(),
            name_map: collections::HashMap::new(),
            service,
        }
    }

    pub fn add(&mut self, id: u64, name: std::ffi::OsString) {
        self.children.insert(id);
        self.name_map.insert(name, id);
    }
}

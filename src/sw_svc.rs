use std::collections::HashMap;
use reqwest;
use serde_json;
use crate::fsys::inode::{ServiceDirNode, SingleService};

pub struct StarWarsService {}

impl SingleService for StarWarsService  {
    fn fetch_data(query: Option<&str>) -> Result<(u64), Box<dyn std::error::Error>> {
        let resp: HashMap<String, serde_json::Value> = reqwest::get("https://swapi.co/api/people/")?
            .json()?;


        Ok(3)
    }
}

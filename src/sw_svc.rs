use std::fmt;
use reqwest;
use serde::Deserialize;
use crate::fsys::inode::{ServiceDirNode, SingleService};

#[derive(Deserialize, Debug)]
pub struct Res {
    pub results: Vec<Person>
}

#[derive(Debug, Deserialize, Clone)]
pub struct Person {
    gender: String,
    name: String,
    height: String,
}

impl fmt::Display for Person {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "The person's name is: {}, and they are {} and about {}cm tall", self.name, self.gender, self.height)
    }

}

pub struct StarWarsService {}

impl SingleService for StarWarsService  {

    fn fetch_data(&self, _query: Option<&str>) -> Vec<String> {
        let data: Res = reqwest::get("https://swapi.co/api/people/")
            .unwrap()
            .json()
            .unwrap();

        data.results.iter().map(|person| {
            person.to_string() + "\n"
        }).collect()
    }
}

pub fn build_sw_service() -> ServiceDirNode {
    let svc = StarWarsService {};
    ServiceDirNode::new(Box::new(svc))
}

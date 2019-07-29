use std::fmt;
use reqwest;
use serde::Deserialize;
use crate::fsys::inode::{ServiceDirNode, SingleService};

const URL: &str = "https://wdi-nyc-planets-api.herokuapp.com/planets";

#[derive(Debug, Deserialize, Clone)]
pub struct Planet {
    name: String,
    color: String,
    num_moons: i64,
}

#[derive(Deserialize)]
pub struct PlanetList {
    planets: Vec<Planet>,
}

impl fmt::Display for Planet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} is {} and has {} moons", self.name, self.color, self.num_moons)
    }

}

pub struct PlanetService {}

impl SingleService for PlanetService  {

    fn fetch_data(&self, _query: Option<&str>) -> Vec<String> {
        let data: PlanetList = reqwest::get(URL)
            .unwrap()
            .json()
            .unwrap();

        data.planets.into_iter().map(|planet| {
            planet.to_string()
        }).collect()
    }
}

pub fn build_planet_service() -> ServiceDirNode {
    let svc = PlanetService {};
    ServiceDirNode::new(Box::new(svc))
}

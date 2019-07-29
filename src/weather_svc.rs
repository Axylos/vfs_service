use std::fmt;
use reqwest;
use serde::Deserialize;
use crate::fsys::inode::{ServiceDirNode, SingleService};

extern crate dotenv;

use dotenv::dotenv;
use std::env;


#[derive(Debug, Deserialize, Clone)]
pub struct Weather {
    temp: f64,
    pressure: f64,
}

impl fmt::Display for Weather {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "The temp is {} and the pressure is {}", self.temp, self.pressure)
    }
}

#[derive(Deserialize, Debug)]
pub struct Meta {
    pub main: Weather
}

pub struct WeatherService {}

impl SingleService for WeatherService  {

    fn fetch_data(&self, _query: Option<&str>) -> Vec<String> {
        dotenv().ok();

        let mut str = String::new();
        let appid = env::var("WEATHER_KEY").unwrap().to_string();
        let url= "https://samples.openweathermap.org/data/2.5/weather?zip=94040,us&appid=";
        str.push_str(&url);
        str.push_str(&appid);

        let data: Meta = reqwest::get(&str)
            .unwrap()
            .json()
            .unwrap();



        vec!(data.main.to_string())
    }
}

pub fn build_weather_service() -> ServiceDirNode {
    let svc = WeatherService {};
    ServiceDirNode::new(Box::new(svc))
}

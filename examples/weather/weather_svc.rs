use std::fmt;
use reqwest;
use serde::Deserialize;
use vfs_service::{SingleService};
extern crate dotenv;

use dotenv::dotenv;
use std::env;


#[derive(Debug, Deserialize)]
pub struct Weather {
    temp: f64,
    pressure: f64,
}

impl fmt::Display for Weather {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "The temp is {}C and the pressure is {}atm", self.temp, self.pressure)
    }
}

#[derive(Deserialize, Debug)]
pub struct Meta {
    pub main: Weather,
    pub name: String,
}

impl fmt::Display for Meta {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "The temp is {}C and the pressure is {}atm for {}", self.main.temp, self.main.pressure, self.name)
    }
}
pub struct WeatherService {}

impl SingleService for WeatherService  {

    fn get_name(&self) -> String {
        "weather_svc".to_string()
    }

    fn fetch_data(&self, query: Option<&str>) -> Vec<String> {
        dotenv().ok();
        let zip = match query {
            Some(q) => q,
            None => "10002"
        };

        let appid = env::var("WEATHER_KEY").unwrap().to_string();
        let url= format!("https://api.openweathermap.org/data/2.5/weather?zip={},us&appid={}&units=metric",
                         zip, appid);

        let data: Meta = reqwest::get(&url)
            .unwrap()
            .json()
            .unwrap();



        vec!(data.to_string())
    }
}

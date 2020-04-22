use std::{env, io};
use vfs_service::{run, SingleService};

mod weather_svc;
use weather_svc::WeatherService;

fn main() {
    let weather = Box::new(WeatherService {});
    let svcs: Vec<Box<dyn SingleService + Send>> = vec!(weather);

    run(svcs);
}

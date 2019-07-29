use std::{env, io};
use vfs_service::{fuse_system, SingleService};

mod weather_svc;
use weather_svc::WeatherService;

fn main() {
    let weather = Box::new(WeatherService {});
    let svcs: Vec<Box<dyn SingleService + Send>> = vec!(weather);

    unsafe {
        let fs = fuse_system::Fs::new(svcs);

        let mnt = match env::args().nth(1) {
            Some(path) => path,
            None => "./test_dir".to_string(),
        };

        println!("{}", mnt);
        let _sys = fuse::spawn_mount(fs, &mnt, &[]).unwrap();
        let mut str = String::new();

        io::stdin().read_line(&mut str).expect("invalid input");
        println!("all done!");
    }
}

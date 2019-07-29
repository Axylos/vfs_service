use std::{env, io};
use vfs_service::{fuse_system};
use log::*;
use syslog::{Facility};

fn init() {

    match syslog::init(
        Facility::LOG_USER,
        LevelFilter::Debug,
        "file system".into(),
    ) {
        Ok(()) => log::info!("logger up"),
        _ => log::error!("logger not up"),
    }

    log::info!("logging started");

}

fn main() {

    init();
    unsafe {
        let fs = fuse_system::Fs::new(vec!());

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

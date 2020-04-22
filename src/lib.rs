use std::{env, io};

pub mod fuse_system;
//pub use fuse_system::{Fs};
extern crate file_node;

pub use file_node::{ServiceDirNode, SingleService};

pub fn run(svcs: Vec<Box<dyn SingleService + Send>>) {
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

pub fn init(svc: Vec<Box<dyn SingleService + Send>>) {
    //logger::foo();

    let mnt = match env::args().nth(1) {
        Some(path) => path,
        None => "./test_dir".to_string(),
    };

    let fs = fuse_system::Fs::new(svc);

    println!("{}", mnt);
    let _sys = fuse::mount(fs, &mnt, &[]).unwrap();
    let mut str = String::new();

    io::stdin().read_line(&mut str).expect("invalid input");
    println!("all done!");
}

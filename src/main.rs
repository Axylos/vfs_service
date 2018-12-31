use std::{env, io};
mod logger;
use log;

mod wiki;
mod file_tree;
mod fsys;
fn main() {
    logger::init();
    let mnt = match env::args().nth(1) {
        Some(path) => path,
        None => "./test_dir".to_string(),
    };

    let fs = fsys::Fs::new();

    log::error!("watwatwat");

    println!("{}", mnt);
    unsafe {
        let _sys = fuse::spawn_mount(fs, &mnt, &[]).unwrap();
        let mut str = String::new();

        io::stdin().read_line(&mut str).expect("invalid input");
    }
    println!("all done!");
}

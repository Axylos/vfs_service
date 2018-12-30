use std::{env, io};
mod logger;
use log;

mod file_tree;
mod fsys;
fn main() {
    logger::init();
    let mnt = match env::args().nth(1) {
        Some(path) => path,
        None => "./test_dir".to_string(),
    };

    let fs = fsys::Fs::new();


    println!("{}", mnt);
    unsafe {
        let sys = fuse::spawn_mount(fs, &mnt, &[]).unwrap();
        let mut str = String::new();

        io::stdin().read_line(&mut str).expect("invalid input");
    }
    println!("all done!");
}

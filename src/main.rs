use std::{env, io};
mod logger;
use log;

mod drakey_fs;
mod fsys;

mod wiki;
fn main() {
    
    logger::init();
    let mnt = match env::args().nth(1) {
        Some(path) => path,
        None => "./test_dir".to_string(),
    };

    let fs = drakey_fs::fs::Fs::new();
    println!("{}", mnt);

    unsafe {
        let _sys = fuse::spawn_mount(fs, &mnt, &[]).unwrap();
        let mut str = String::new();

        io::stdin().read_line(&mut str).expect("invalid input");
    }

    println!("all done!");
}

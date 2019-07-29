use std::{env, io};
mod logger;

mod fsys;

mod wiki;
fn main() {
    logger::init();
    let str = wiki::query();
    println!("{:?}", str);

    let mnt = match env::args().nth(1) {
        Some(path) => path,
        None => "./test_dir".to_string(),
    };

    let fs = fsys::fuse_system::Fs::new();

    println!("{}", mnt);
        let _sys = fuse::mount(fs, &mnt, &[]).unwrap();
        let mut str = String::new();

        io::stdin().read_line(&mut str).expect("invalid input");
    println!("all done!");
}

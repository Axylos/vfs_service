use syslog::{Facility};

pub fn init() {
    match syslog::init(
        Facility::LOG_USER,
        log::LevelFilter::Debug,
        "file system".into(),
    ) {
        Ok(()) => log::info!("logger up"),
        _ => log::error!("logger not up"),
    }

    log::info!("logging started");
}

pub fn foo() {
    println!("ok");
}

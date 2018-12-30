use log;
use syslog::{Facility};

pub fn init() {
    syslog::init(syslog::Facility::LOG_USER,
                 log::LevelFilter::Trace,
                 Some("file system"));

    log::trace!("logging started");
}

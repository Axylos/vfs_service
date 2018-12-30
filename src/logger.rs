use log;
use syslog::Facility;

pub fn init() {
    /*
    let formatter = Formatter3164 {
        facility: Facility::LOG_USER,
        hostname: None,
        process: "file system".into(),
        pid: 42,

    };
    */
    //syslog::init_tcp("192.168.1.176:514", "thingy".into(), Facility::LOG_USER, log::LevelFilter::Debug);

    match syslog::init(
        Facility::LOG_USER,
        log::LevelFilter::Debug,
        "file system".into(),
    ) {
        Ok(()) => log::info!("logger up"),
        _ => log::error!("logger not up"),
    }

    log::error!("logging started");
    log::info!("logging from info");
}

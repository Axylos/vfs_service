use serde_json::Error;
use std::process::Command;

pub fn query() -> Result<String, Error> {
    let resp = Command::new("curl")
        .arg("https://randomuser.me/api")
        .output()
        .expect("wat");

    let data = String::from_utf8_lossy(&resp.stdout);
    serde_json::from_str(&data)
}

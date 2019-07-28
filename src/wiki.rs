use serde_json::{Value, Error};
use std::process::Command;

pub fn query() -> Result<String, Error> {
    let resp = Command::new("curl")
        .arg("https://randomuser.me/api")
        .output()
        .expect("wat");

    let data = String::from_utf8_lossy(&resp.stdout);
    let val: Value = serde_json::from_str(&data)?;
    println!("{:?}", val);
    Ok("wat".to_string())
}

use std::process::Command;

pub fn capture(timeout: &str, path: &str) {
    Command::new("raspistill")
        .args(["-t", timeout, "-o", path])
        .output()
        .expect("raspistill sucks !!");
}

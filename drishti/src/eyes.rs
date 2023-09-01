use std::process::Command;

use anyhow::{Context, Result};
use opencv::highgui;
use opencv::imgcodecs;
use opencv::prelude::*;

pub fn cv_example(img_path: &str) -> Result<()> {
    let image =
        imgcodecs::imread(img_path, imgcodecs::IMREAD_COLOR).context("Image reading failed")?;

    highgui::imshow("Display window", &image).context("Image display failed")?;
    highgui::wait_key(0).context("Waiting for a keystroke in the window failed")?; // Wait for a keystroke in the window

    Ok(())
}

pub fn capture(timeout: &str, path: &str) {
    Command::new("libcamera-still")
        .args(["-t", timeout, "-o", path])
        .output()
        .expect("libcamera-still sucks !!");
}

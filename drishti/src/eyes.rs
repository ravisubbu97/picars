use std::process::Command;

use anyhow::{Context, Result};
use opencv::highgui;
use opencv::imgcodecs;
use opencv::imgproc;
use opencv::prelude::*;

pub fn cv_example(img_path: &str) -> Result<()> {
    let image =
        imgcodecs::imread(img_path, imgcodecs::IMREAD_GRAYSCALE).context("Image reading failed")?;
    let mut edges = Mat::default();

    imgproc::canny(&image, &mut edges, 50.0, 150.0, 3, false).context("Canny algo failed")?;
    highgui::imshow("Original Image", &image).context("Image show failed")?;
    highgui::imshow("Canny Edges", &edges).context("Image show failed")?;
    highgui::wait_key(0).context("Waiting for a keystroke in the window failed")?;

    Ok(())
}

pub fn capture(timeout: &str, path: &str) {
    Command::new("libcamera-still")
        .args(["-t", timeout, "-o", path])
        .output()
        .expect("libcamera-still sucks !!");
}

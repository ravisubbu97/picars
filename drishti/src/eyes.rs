use std::{
    io::Error,
    process::{Command, Output},
    thread,
    time::Duration,
};

use anyhow::{Context, Result};
use opencv::{core, imgcodecs, imgproc, prelude::*};

pub fn cv_example(img_path: &str, cap_img_path: &str, edge_img_path: &str) -> Result<()> {
    capture("1000", img_path).context("Image capture failed")?;
    thread::sleep(Duration::from_secs(1));

    let image =
        imgcodecs::imread(img_path, imgcodecs::IMREAD_GRAYSCALE).context("Image reading failed")?;
    let mut edges = Mat::default();
    let params = core::Vector::new();

    imgproc::canny(&image, &mut edges, 50.0, 150.0, 3, false).context("Canny algo failed")?;
    imgcodecs::imwrite(cap_img_path, &image, &params).context("Image saving failed")?;
    imgcodecs::imwrite(edge_img_path, &edges, &params).context("Image saving failed")?;

    Ok(())
}

pub fn capture(timeout: &str, path: &str) -> Result<Output, Error> {
    Command::new("libcamera-still")
        .args(["-t", timeout, "-o", path])
        .output()
}

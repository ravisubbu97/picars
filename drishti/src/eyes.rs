use std::{
    io::Error,
    process::{Command, Output},
    thread,
    time::Duration,
};

use anyhow::{Context, Result};
use nokhwa::{pixel_format::RgbFormat, utils::*, Camera};
use opencv::{core, imgcodecs, imgproc, prelude::*};

pub fn nokhwa_example(img_path: &str) -> Result<()> {
    let index = CameraIndex::Index(0);
    let requested =
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);
    let mut camera = Camera::new(index, requested).context("Cannot create camera")?;
    let frame = camera.frame().context("Cannot capture frame")?;
    let decoded = frame
        .decode_image::<RgbFormat>()
        .context("Cannot decode frame")?;

    decoded.save(img_path).context("Cannot save image")?;

    Ok(())
}
pub fn cv_example(img_path: &str, cap_img_path: &str, edge_img_path: &str) -> Result<f64> {
    capture("1000", img_path).context("Image capture failed")?;
    thread::sleep(Duration::from_secs(1));

    let strt = core::get_tick_count()? as f64;
    let image =
        imgcodecs::imread(img_path, imgcodecs::IMREAD_GRAYSCALE).context("Image reading failed")?;
    let mut edges = Mat::default();
    let params = core::Vector::new();

    imgproc::canny(&image, &mut edges, 50.0, 150.0, 3, false).context("Canny algo failed")?;
    let time = (core::get_tick_count()? as f64 - strt) / core::get_tick_frequency()?;

    imgcodecs::imwrite(cap_img_path, &image, &params).context("Image saving failed")?;
    imgcodecs::imwrite(edge_img_path, &edges, &params).context("Image saving failed")?;

    Ok(time)
}

pub fn capture(timeout: &str, path: &str) -> Result<Output, Error> {
    Command::new("libcamera-still")
        .args(["-t", timeout, "-o", path])
        .output()
}

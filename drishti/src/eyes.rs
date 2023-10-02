use std::{
    io::Error,
    process::{Command, Output},
    thread,
    time::Duration,
};

use anyhow::{Context, Result};
use opencv::{
    core, highgui, imgcodecs, imgproc,
    prelude::*,
    videoio::{self, VideoCapture, VideoCaptureAPIs},
};

pub fn cuda_check() -> opencv::Result<bool> {
    let dev_count = core::get_cuda_enabled_device_count()?;
    let cuda_available = dev_count > 0;
    if cuda_available {
        for dev_num in 0..dev_count {
            core::print_short_cuda_device_info(dev_num)?;
        }
    }

    Ok(cuda_available)
}

pub fn camera_backends() -> opencv::Result<core::Vector<VideoCaptureAPIs>> {
    videoio::get_camera_backends()
}

pub fn video_capture() -> Result<()> {
    let window = "video capture";
    highgui::named_window(window, highgui::WINDOW_AUTOSIZE)?;
    let mut cam = VideoCapture::new(0, videoio::CAP_V4L2)?;
    let opened = VideoCapture::is_opened(&cam)?;
    if !opened {
        panic!("Unable to open default camera!");
    }
    loop {
        let mut frame = Mat::default();
        cam.read(&mut frame)?;
        if frame.size()?.width > 0 {
            highgui::imshow(window, &frame)?;
        }
        let key = highgui::wait_key(10)?;
        if key > 0 && key != 255 {
            break;
        }
    }

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

use std::{
    f64::consts::PI,
    io::Error,
    process::{Command, Output},
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use opencv::{
    core::{self, Point, Point2f, Scalar, VecN, Vector},
    highgui, imgcodecs, imgproc,
    prelude::*,
    types::{VectorOfVec2f, VectorOfVec4i},
    videoio::{self, VideoCapture, VideoCaptureAPIs},
};

const WAIT_MILLIS: i32 = 1000;
const MIN_THRESHOLD: i32 = 50;
const MAX_TRACKBAR: i32 = 150;
const STANDARD_NAME: &str = "Standard Hough Lines Demo";
const PROBABILISTIC_NAME: &str = "Probabilistic Hough Lines Demo";

pub fn standard_hough(edges: &Mat, s_trackbar: i32) -> Result<Vector<VecN<f32, 2>>> {
    let mut s_lines = VectorOfVec2f::new();
    let mut standard_hough = Mat::default();

    imgproc::cvt_color(edges, &mut standard_hough, imgproc::COLOR_GRAY2BGR, 0)?;
    imgproc::hough_lines(
        edges,
        &mut s_lines,
        1.,
        PI / 180.,
        MIN_THRESHOLD + s_trackbar,
        0.,
        0.,
        0.,
        PI,
    )?;

    for s_line in s_lines.iter() {
        let [r, t] = *s_line;
        let cos_t = t.cos();
        let sin_t = t.sin();
        let x0 = r * cos_t;
        let y0 = r * sin_t;
        let alpha = 1000.;

        let pt1 = Point2f::new(x0 + alpha * -sin_t, y0 + alpha * cos_t)
            .to::<i32>()
            .unwrap();
        let pt2 = Point2f::new(x0 - alpha * -sin_t, y0 - alpha * cos_t)
            .to::<i32>()
            .unwrap();
        imgproc::line(
            &mut standard_hough,
            pt1,
            pt2,
            Scalar::new(255., 0., 0., 0.),
            3,
            imgproc::LINE_AA,
            0,
        )?;
    }
    highgui::imshow(STANDARD_NAME, &standard_hough)?;
    highgui::wait_key(WAIT_MILLIS)?;
    imgcodecs::imwrite("standard_hough.jpg", &standard_hough, &Vector::new())?;

    Ok(s_lines)
}

pub fn probabilistic_hough(edges: &Mat, p_trackbar: i32) -> Result<Vector<VecN<i32, 4>>> {
    let mut p_lines = VectorOfVec4i::new();
    let mut probabalistic_hough = Mat::default();

    imgproc::cvt_color(edges, &mut probabalistic_hough, imgproc::COLOR_GRAY2BGR, 0)?;
    imgproc::hough_lines_p(
        edges,
        &mut p_lines,
        1.,
        PI / 180.,
        MIN_THRESHOLD + p_trackbar,
        30.,
        10.,
    )?;

    for l in p_lines.iter() {
        imgproc::line(
            &mut probabalistic_hough,
            Point::new(l[0], l[1]),
            Point::new(l[2], l[3]),
            Scalar::new(255., 0., 0., 0.),
            3,
            imgproc::LINE_AA,
            0,
        )?;
    }
    highgui::imshow(PROBABILISTIC_NAME, &probabalistic_hough)?;
    highgui::wait_key(WAIT_MILLIS)?;
    imgcodecs::imwrite(
        "probabalistic_hough.jpg",
        &probabalistic_hough,
        &Vector::new(),
    )?;

    Ok(p_lines)
}

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

pub fn video_capture(time: u64) -> Result<()> {
    let window = "video capture";
    highgui::named_window(window, highgui::WINDOW_AUTOSIZE)?;
    let mut cam = VideoCapture::new(0, videoio::CAP_V4L2)?;
    let opened = VideoCapture::is_opened(&cam)?;
    if !opened {
        panic!("Unable to open default camera!");
    }

    let s_trackbar = MAX_TRACKBAR;
    let p_trackbar = MAX_TRACKBAR;

    loop {
        let time_spent = Instant::now();
        let mut frame = Mat::default();
        cam.read(&mut frame)?;
        if frame.size()?.width > 0 {
            highgui::imshow(window, &frame)?;
            highgui::wait_key(WAIT_MILLIS)?;

            let mut src_gray = Mat::default();
            imgproc::cvt_color(&frame, &mut src_gray, imgproc::COLOR_BGR2GRAY, 0)
                .context("BGR2RGB conversion failed")?;

            let mut edges = Mat::default();
            imgproc::canny(&src_gray, &mut edges, 50., 200., 3, false)
                .context("Canny Algorithm failed")?;

            standard_hough(&edges, s_trackbar).context("Standard Hough Transfrom failed")?;
            probabilistic_hough(&edges, p_trackbar)
                .context("Probabilistic Hough Transfrom failed")?;
        }
        if time_spent.elapsed().as_secs() > time {
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

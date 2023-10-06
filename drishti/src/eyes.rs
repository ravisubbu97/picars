use std::{
    f64::consts::PI,
    io::Error,
    process::{Command, Output},
    thread,
    time::Duration,
};

use anyhow::{Context, Result};
use opencv::{
    core::{self, Mat, Point, Point2f, Scalar, VecN, Vector, BORDER_DEFAULT, CV_8UC1},
    imgcodecs, imgproc,
    prelude::*,
    types::{VectorOfVec2f, VectorOfVec3f, VectorOfVec4i},
    videoio::{self, VideoCapture, VideoCaptureAPIs},
};

#[cfg(feature = "gui")]
use opencv::highgui;

#[cfg(feature = "gui")]
const WAIT_MILLIS: i32 = 1000;
#[cfg(feature = "gui")]
const STANDARD_NAME: &str = "Standard Hough Lines Demo";
#[cfg(feature = "gui")]
const PROBABILISTIC_NAME: &str = "Probabilistic Hough Lines Demo";

const MIN_THRESHOLD: i32 = 50;
const MAX_TRACKBAR: i32 = 150;

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

    #[cfg(feature = "gui")]
    {
        highgui::imshow(STANDARD_NAME, &standard_hough)?;
        highgui::wait_key(WAIT_MILLIS)?;
    }

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

    #[cfg(feature = "gui")]
    {
        highgui::imshow(PROBABILISTIC_NAME, &probabalistic_hough)?;
        highgui::wait_key(WAIT_MILLIS)?;
    }

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

pub fn cv_example_vid() -> Result<()> {
    #[cfg(feature = "gui")]
    let window = "video capture";
    #[cfg(feature = "gui")]
    {
        highgui::named_window(window, highgui::WINDOW_AUTOSIZE)?;
    }
    let mut cam = VideoCapture::new(0, videoio::CAP_V4L2)?;
    let opened = VideoCapture::is_opened(&cam)?;
    if !opened {
        panic!("Unable to open default camera!");
    }

    let s_trackbar = MAX_TRACKBAR;
    let p_trackbar = MAX_TRACKBAR;

    let mut frame = Mat::default();
    let mut src_gray = Mat::default();

    cam.read(&mut frame)?;

    if frame.size()?.width > 0 {
        #[cfg(feature = "gui")]
        {
            highgui::imshow(window, &frame)?;
            highgui::wait_key(WAIT_MILLIS)?;
        }
        imgproc::cvt_color(&frame, &mut src_gray, imgproc::COLOR_BGR2GRAY, 0)
            .context("BGR2GRAY conversion failed")?;

        let circles = hough_circles(&src_gray)?; // giving gray scale image to hough circles function

        println!("number of circles detected{}", circles.len());

        // Create an empty black mask with the same size as the input image
        let mut mask =
            Mat::new_rows_cols_with_default(frame.rows(), frame.cols(), CV_8UC1, Scalar::all(0.0))?;

        // Draw the detected circles on the mask
        for circle in circles.iter() {
            let center = core::Point {
                x: circle[0] as i32,
                y: circle[1] as i32,
            };
            let radius = circle[2] as i32;
            imgproc::circle(
                &mut mask,
                center,
                radius,
                core::Scalar::all(255.0),
                -1,
                imgproc::LINE_AA,
                0,
            )?;
        }

        // Create a result image by bitwise AND-ing the input image with the mask
        let mut only_circles = Mat::default();
        core::bitwise_and(&frame, &frame, &mut only_circles, &mask)?;

        #[cfg(feature = "gui")]
        {
            highgui::imshow("circles", &only_circles)?;
            highgui::wait_key(WAIT_MILLIS)?;
        }

        let green_light =
            detect_green_light(&only_circles).context("Green light detection failed")?;

        if green_light {
            println!("🟢 🟩 💚");
        } else {
            println!("🔴 🟥 😡");
        }

        let mut edges = Mat::default();
        imgproc::canny(&src_gray, &mut edges, 50., 200., 3, false)
            .context("Canny Algorithm failed")?;

        standard_hough(&edges, s_trackbar).context("Standard Hough Transfrom failed")?;
        probabilistic_hough(&edges, p_trackbar).context("Probabilistic Hough Transfrom failed")?;
    }

    Ok(())
}

pub fn cv_example_photo(img_path: &str, cap_img_path: &str, edge_img_path: &str) -> Result<f64> {
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

// Function to perform circle detection using Hough Circle Transform
pub fn hough_circles(input_image: &Mat) -> Result<VectorOfVec3f> {
    // Apply Gaussian blur to reduce noise and improve circle detection
    let mut blurred = Mat::default(); // ToDo: Can be used accross all functions..
    imgproc::gaussian_blur(
        input_image,
        &mut blurred,
        core::Size {
            width: 9,
            height: 9,
        },
        2.0,
        2.0,
        BORDER_DEFAULT,
    )
    .expect("something wrong during blurring");

    // Detect circles using the Hough Circle Transform
    let mut circles = VectorOfVec3f::new();
    imgproc::hough_circles(
        &blurred,                // Input grayscale image
        &mut circles,            // Output vector of circles (x, y, radius)
        imgproc::HOUGH_GRADIENT, // Detection method
        1.0,   // Inverse ratio of the accumulator resolution to the image resolution
        50.0,  // Minimum distance between detected centers
        100.0, // Canny edge detection threshold
        30.0,  // Accumulator threshold for circle detection
        10,    // Minimum circle radius
        200,   // Maximum circle radius
    )?;

    Ok(circles)
}

pub fn detect_green_light(image: &Mat) -> Result<bool> {
    // Convert the image to HSV color space
    let mut hsv_image = Mat::default();
    imgproc::cvt_color(image, &mut hsv_image, imgproc::COLOR_BGR2HSV, 0)
        .context("BGR to HSV conversion failed")?;

    // Define the lower and upper bounds for green in HSV
    let lower_green = Scalar::new(35.0, 100.0, 100.0, 0.0);
    let upper_green = Scalar::new(85.0, 255.0, 255.0, 0.0);
    let lower_red = Scalar::new(0.0, 100.0, 100.0, 0.0);
    let upper_red = Scalar::new(10.0, 255.0, 255.0, 0.0);

    // Create masks for red and green regions in the image
    let mut red_mask = Mat::default();
    let mut green_mask = Mat::default();
    core::in_range(&hsv_image, &lower_red, &upper_red, &mut red_mask)
        .context("Filtering of red Pixels failed")?;
    core::in_range(&hsv_image, &lower_green, &upper_green, &mut green_mask)
        .context("Filtering of green Pixels failed")?;

    // Calculate the total number of non-zero (white) pixels in each mask
    let red_pixel_count =
        core::count_non_zero(&red_mask).context("Count of non-zero red pixels failed")?;
    let green_pixel_count =
        core::count_non_zero(&green_mask).context("Count of non-zero green pixels failed")?;

    #[cfg(feature = "gui")]
    {
        highgui::imshow("green_msk_out", &green_mask)?;
        highgui::wait_key(WAIT_MILLIS)?;
        highgui::imshow("red_msk_out", &red_mask)?;
        highgui::wait_key(WAIT_MILLIS)?;
    }

    // Determine if the green/red light is detected based on the pixel count
    Ok(green_pixel_count > red_pixel_count)
}

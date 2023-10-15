use std::{
    f64::consts::PI,
    io::Error,
    process::{Command, Output},
    thread,
    time::Duration,
};

use anyhow::{Context, Result};
use opencv::{
    core::{self, Mat, Point, Point2f, Scalar, VecN, Vector, BORDER_DEFAULT, CV_8UC1, LINES},
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

const HOUGH_THRESHOLD: i32 = 50;

pub fn standard_hough(edges: &Mat) -> Result<Vector<VecN<f32, 2>>> {
    let mut s_lines = VectorOfVec2f::new();
    let mut hough_lines = Mat::default();

    imgproc::cvt_color(edges, &mut hough_lines, imgproc::COLOR_GRAY2BGR, 0)?;
    imgproc::hough_lines(
        edges,
        &mut s_lines,
        1.,
        PI / 180.,
        HOUGH_THRESHOLD + 100,
        0.,
        0.,
        0.,
        PI,
    )?;

    #[cfg(feature = "gui")]
    {
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
                &mut hough_lines,
                pt1,
                pt2,
                Scalar::new(255., 0., 0., 0.),
                3,
                imgproc::LINE_AA,
                0,
            )?;
        }
        highgui::imshow(STANDARD_NAME, &hough_lines)?;
        highgui::wait_key(WAIT_MILLIS)?;
    }

    Ok(s_lines)
}

pub fn probabilistic_hough(edges: &Mat) -> Result<Vector<VecN<i32, 4>>> {
    let mut p_lines = VectorOfVec4i::new();
    let mut hough_lines = Mat::default();

    imgproc::cvt_color(edges, &mut hough_lines, imgproc::COLOR_GRAY2BGR, 0)?;
    imgproc::hough_lines_p(edges, &mut p_lines, 1., PI / 180., HOUGH_THRESHOLD, 30., 3.)?;

    #[cfg(feature = "gui")]
    {
        for l in p_lines.iter() {
            imgproc::line(
                &mut hough_lines,
                Point::new(l[0], l[1]),
                Point::new(l[2], l[3]),
                Scalar::new(255., 0., 0., 0.),
                3,
                imgproc::LINE_AA,
                0,
            )?;
        }
        highgui::imshow(PROBABILISTIC_NAME, &hough_lines)?;
        highgui::wait_key(WAIT_MILLIS)?;
    }

    Ok(p_lines)
}

fn calculate_lane_center(lines: &VectorOfVec4i, image_width: f32) -> opencv::Result<(f32, f32)> {
    // Calculate the lane center as the average of x-coordinates of the detected lines
    // TODO: Line count should be 2 (maybe nearest two ?), bcz we need to detect only one lane
    let mut lane_center_x: f32 = 0.0;
    let mut line_count: f32 = 0.0;

    for line in lines.iter() {
        let x1 = line[0] as f32;
        let x2 = line[2] as f32;

        lane_center_x += (x1 + x2) / 2.0;
        line_count += 1.0;
    }

    if line_count > 0.0 {
        lane_center_x /= line_count;
    }

    // Convert lane_center_x to be relative to the image width (0.0 = left, 1.0 = right)
    lane_center_x /= image_width;

    Ok((lane_center_x, line_count))
}

#[cfg(feature = "gui")]
fn lane_detector(lines: &VectorOfVec4i, image_width: f32, image: &Mat) -> Result<()> {
    // Calculate the lane center
    let (lane_center_x, line_count) = calculate_lane_center(lines, image_width)?;
    // image centre is always 0.5 ?
    let image_center_x = 0.5;
    // Calculate the deviation from the lane center
    let deviation = image_center_x - lane_center_x;
    println!(
        "[lane_center_x: {}] [line_count: {}] [image_center_x: {}] [deviation: {}]",
        lane_center_x, line_count, image_center_x, deviation
    );

    // Example steering decision based on the deviation
    if deviation < 0.0 {
        println!("Steer left");
    } else if deviation > 0.0 {
        println!("Steer right");
    } else {
        println!("Keep straight");
    }

    // Draw detected lines and lane center on the original image
    let mut result = image.clone();
    for line in lines.iter() {
        let pt1 = core::Point {
            x: line[0],
            y: line[1],
        };
        let pt2 = core::Point {
            x: line[2],
            y: line[3],
        };
        imgproc::line(
            &mut result,
            pt1,
            pt2,
            Scalar::new(0.0, 0.0, 255.0, 0.0),
            2,
            imgproc::LINE_AA,
            0,
        )?;
    }
    imgproc::line(
        &mut result,
        core::Point {
            x: lane_center_x as i32,
            y: 0,
        },
        core::Point {
            x: lane_center_x as i32,
            y: image.rows(),
        },
        Scalar::new(0.0, 255.0, 0.0, 0.0),
        2,
        imgproc::LINE_AA,
        0,
    )?;

    // Display or save the result image
    highgui::imshow("Lane Detection", &result)?;
    highgui::wait_key(0)?;

    Ok(())
}

#[cfg(not(feature = "gui"))]
fn lane_detector(lines: &VectorOfVec4i, image_width: f32) -> Result<()> {
    // Calculate the lane center
    let (lane_center_x, line_count) = calculate_lane_center(lines, image_width)?;
    // image centre is always 0.5 ?
    let image_center_x = 0.5;
    // Calculate the deviation from the lane center
    let deviation = image_center_x - lane_center_x;
    println!(
        "[lane_center_x: {}] [line_count: {}] [image_center_x: {}] [deviation: {}]",
        lane_center_x, line_count, image_center_x, deviation
    );

    // Example steering decision based on the deviation
    if deviation < 0.0 {
        println!("Steer left");
    } else if deviation > 0.0 {
        println!("Steer right");
    } else {
        println!("Keep straight");
    }

    Ok(())
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
    let mut cam = VideoCapture::new(0, videoio::CAP_V4L2).context("unable to create camera")?;
    let opened = VideoCapture::is_opened(&cam).context("unable to open camera")?;
    if !opened {
        panic!("Unable to open default camera!");
    }

    let mut frame = Mat::default();
    cam.read(&mut frame).context("unable to capture frame")?;

    if frame.size()?.width > 0 {
        #[cfg(feature = "gui")]
        {
            highgui::imshow(window, &frame)?;
            highgui::wait_key(WAIT_MILLIS)?;
        }

        // Create an trapeziodal black mask for lane detection with the same size as the input image
        let mut mask2 =
            Mat::new_rows_cols_with_default(frame.rows(), frame.cols(), CV_8UC1, Scalar::all(0.0))?;

        // points for trapeziod
        let points = [
            Point::new(200, 180),
            Point::new(400, 180),
            Point::new(850, 350),
            Point::new(-50, 350),
        ];

        let roi_poly = Mat::from_slice(&points)?;
        // filling the trapeziod with white color
        imgproc::fill_poly(
            &mut mask2,
            &roi_poly,
            Scalar::new(255.0, 255.0, 255.0, 255.0),
            LINES,
            0,
            Point::new(0, 0),
        )?;

        let trapezion_frame = frame.clone();
        //bit-wise and to the original image
        let mut trapeziod = Mat::default();
        core::bitwise_and(&trapezion_frame, &trapezion_frame, &mut trapeziod, &mask2)?;

        #[cfg(feature = "gui")]
        {
            highgui::imshow("trapeziodal_frame", &trapeziod)?;
            highgui::wait_key(WAIT_MILLIS)?;
        }

        // trying trapezoidal image for hough lines
        let mut src_gray = Mat::default();
        imgproc::cvt_color(&trapeziod, &mut src_gray, imgproc::COLOR_BGR2GRAY, 0)
            .context("BGR2GRAY conversion failed")?;
        // Apply Gaussian blur to reduce noise and improve circle detection
        let mut src_blurred = Mat::default();
        imgproc::gaussian_blur(
            &src_gray,
            &mut src_blurred,
            core::Size {
                width: 3,
                height: 3,
            },
            2.0,
            2.0,
            BORDER_DEFAULT,
        )
        .context("Gaussian filter failed")?;

        let mut edges = Mat::default();
        imgproc::canny(&src_blurred, &mut edges, 50., 200., 3, false)
            .context("Canny Algorithm failed")?;

        let hough_lines = probabilistic_hough(&edges).context("Standard Hough Transfrom failed")?;
        println!("LINES: {:?}", hough_lines);

        let circles = hough_circles(&src_blurred).context("circles are not created")?; // giving gray scale image to hough circles function

        println!("number of circles detected{}", circles.len());

        // Create an empty black mask for circle detection with the same size as the input image
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
            println!("🟢 🟩 💚 [GREEN]");
        } else {
            println!("🔴 🟥 😡 [RED]");
        }

        #[cfg(feature = "gui")]
        {
            lane_detector(&hough_lines, frame.cols() as f32, &frame)
                .context("Lane detection failed")?;
        }
        #[cfg(not(feature = "gui"))]
        {
            lane_detector(&hough_lines, frame.cols() as f32).context("Lane detection failed")?;
        }
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
    // Detect circles using the Hough Circle Transform
    let mut circles = VectorOfVec3f::new();
    imgproc::hough_circles(
        &input_image,            // Input grayscale image
        &mut circles,            // Output vector of circles (x, y, radius)
        imgproc::HOUGH_GRADIENT, // Detection method
        1.0,   // Inverse ratio of the accumulator resolution to the image resolution
        5.0,   // Minimum distance between detected centers
        150.0, // Canny edge detection threshold
        0.9,   // Accumulator threshold for circle detection
        0,     // Minimum circle radius
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

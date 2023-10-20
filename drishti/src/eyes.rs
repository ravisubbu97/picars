use std::{
    f64::consts::PI,
    io::Error,
    process::{Command, Output},
    thread,
    time::Duration,
};

use anyhow::{Context, Result};
use opencv::{
    core::{self, Mat, Point, Scalar, Vec4i, VecN, Vector, BORDER_DEFAULT, CV_8UC1, LINES},
    imgcodecs, imgproc,
    prelude::*,
    types::{VectorOfVec2f, VectorOfVec3f, VectorOfVec4i},
    videoio::{self, VideoCapture, VideoCaptureAPIs},
};

#[cfg(feature = "gui")]
use opencv::core::Point2f;
#[cfg(feature = "gui")]
use opencv::highgui;

pub const WAIT_MILLIS: i32 = 1000;
#[cfg(feature = "gui")]
const STANDARD_NAME: &str = "Standard Hough Lines Demo";
#[cfg(feature = "gui")]
const PROBABILISTIC_NAME: &str = "Probabilistic Hough Lines Demo";

const HOUGH_THRESHOLD: i32 = 50;

pub fn standard_hough(canny_img: &Mat) -> Result<Vector<VecN<f32, 2>>> {
    let mut s_lines = VectorOfVec2f::new();
    let mut hough_lines = Mat::default();

    imgproc::cvt_color(canny_img, &mut hough_lines, imgproc::COLOR_GRAY2BGR, 0)?;
    imgproc::hough_lines(
        canny_img,
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

pub fn probabilistic_hough(canny_img: &Mat) -> Result<Vector<VecN<i32, 4>>> {
    let mut p_lines = VectorOfVec4i::new();
    let mut hough_lines = Mat::default();

    imgproc::cvt_color(canny_img, &mut hough_lines, imgproc::COLOR_GRAY2BGR, 0)?;
    imgproc::hough_lines_p(
        canny_img,
        &mut p_lines,
        1.,
        PI / 180.,
        HOUGH_THRESHOLD,
        30.,
        3.,
    )?;

    #[cfg(feature = "gui")]
    {
        for l in p_lines.iter() {
            // imgproc::polylines(
            //     &mut hough_lines,
            //     &l,
            //     true,
            //     Scalar::new(0., 255., 0., 0.),
            //     2,
            //     imgproc::LINE_AA,
            //     0,
            // )?;
            imgproc::line(
                &mut hough_lines,
                Point::new(l[0], l[1]),
                Point::new(l[2], l[3]),
                Scalar::new(255., 0., 0., 0.),
                2,
                imgproc::LINE_AA,
                0,
            )?;
        }
        highgui::imshow(PROBABILISTIC_NAME, &hough_lines)?;
        highgui::wait_key(WAIT_MILLIS)?;
    }

    Ok(p_lines)
}

fn calculate_lane_center(
    v_lines: &VectorOfVec4i,
    h_lines: &VectorOfVec4i,
    image_width: f32,
) -> opencv::Result<(f32, Vec4i, Vec4i)> {
    // Calculate the lane center as the average of x-coordinates of the detected lines
    // TODO: Line count should be 2 (maybe nearest two ?), bcz we need to detect only one lane
    let mut nearest_left = VecN([0, 0, 0, 0]);
    let mut left_dis: i32 = 320;
    let mut nearest_right = VecN([0, 0, 0, 0]);
    let mut right_dis: i32 = 320;

    // figuring out nearest right and nearest left lines
    for line in v_lines.iter() {
        let x1 = line[0];
        let x2 = line[1];
        let y1 = line[2];
        let y2 = line[3];
        if ((x1 - 320) < 0) && ((x1 - 320).abs() < left_dis) && (x1 < x2) && (y1 > y2) {
            nearest_left = line;
            left_dis = (x1 - 320).abs();
        } else if ((x1 - 320) > 0) && ((x1 - 320).abs() < right_dis) && (x1 > x2) && (y1 > y2) {
            nearest_right = line;
            right_dis = (x1 - 320).abs();
        }
    }

    let lane_center_x = (nearest_left[0] + nearest_right[0]) as f32 / 2.0;

    Ok((lane_center_x, nearest_left, nearest_right))
}

fn line_categorization(
    lines: &VectorOfVec4i,
    horizontal_threshold: f32,
    vertical_threshold: f32,
    /*curved_slope_range: (f32, f32),*/
) -> (
    VectorOfVec4i,
    VectorOfVec4i,
    /*VectorOfVec4i,*/ VectorOfVec4i,
) {
    let mut horizontal_lines = VectorOfVec4i::new();
    let mut vertical_lines = VectorOfVec4i::new();
    /*let mut curved_lines = Vec::new();*/
    let mut other_lines = VectorOfVec4i::new();

    for line in lines.iter() {
        let x1 = line[0] as f32;
        let y1 = line[1] as f32;
        let x2 = line[2] as f32;
        let y2 = line[3] as f32;

        // Calculate the slope
        let slope = if (x2 - x1).abs() > f32::EPSILON {
            (y2 - y1) / (x2 - x1)
        } else {
            std::f32::INFINITY // Undefined slope (vertical line)
        };

        // Categorize lines based on slope using match
        match slope.abs() {
            s if s < horizontal_threshold => horizontal_lines.push(line),
            s if s.is_infinite() || s < vertical_threshold => vertical_lines.push(line),
            /*s if s >= curved_slope_range.0 && s <= curved_slope_range.1 => {
                curved_lines.push(line)
            }*/
            _ => other_lines.push(line),
        }
        println!("[slope: {}] [of line: {:?}]", slope, line)
    }

    (
        horizontal_lines,
        vertical_lines,
        /*curved_lines,*/ other_lines,
    )
}

#[cfg(feature = "gui")]
fn lane_detector(
    v_lines: &VectorOfVec4i,
    h_lines: &VectorOfVec4i,
    image_width: f32,
    image: &Mat,
) -> Result<()> {
    // Calculate the lane center
    let (lane_center_x, nearest_left, nearest_right) =
        calculate_lane_center(v_lines, h_lines, image_width)?;
    // image centre is always 0.5 ?
    let image_center_x = 320.0;
    // Calculate the deviation from the lane center
    let deviation = image_center_x - lane_center_x;
    println!(
        "[lane_center_x: {}] [nearest_left: {:?}] [nearest_right: {:?}] [image_center_x: {}] [deviation: {}]",
        lane_center_x, nearest_left, nearest_right, image_center_x, deviation
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
    for line in [nearest_left, nearest_right].iter() {
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
    {
        highgui::named_window("video capture", highgui::WINDOW_AUTOSIZE)?;
    }
    let mut cam = VideoCapture::new(0, videoio::CAP_V4L2).context("unable to create camera")?;
    let opened = VideoCapture::is_opened(&cam).context("unable to open camera")?;
    if !opened {
        panic!("Unable to open default camera!");
    }

    let mut frame_img = Mat::default();
    cam.read(&mut frame_img)
        .context("unable to capture frame")?;

    if frame_img.size()?.width > 0 {
        #[cfg(feature = "gui")]
        {
            highgui::imshow("frame image", &frame_img)?;
            highgui::wait_key(WAIT_MILLIS)?;
        }

        // Create an trapeziodal black mask for lane detection with the same size as the input image
        let mut trapezoid_mask = Mat::new_rows_cols_with_default(
            frame_img.rows(),
            frame_img.cols(),
            CV_8UC1,
            Scalar::all(0.0),
        )?;

        // points for trapezoid
        let points = [
            Point::new(200, 180),
            Point::new(400, 180),
            Point::new(850, 350),
            Point::new(-50, 350),
        ];

        let roi_poly = Mat::from_slice(&points)?;
        // filling the trapezoid with white color
        imgproc::fill_poly(
            &mut trapezoid_mask,
            &roi_poly,
            Scalar::new(255.0, 255.0, 255.0, 255.0),
            LINES,
            0,
            Point::new(0, 0),
        )?;

        //bit-wise and to the original image
        let mut trapezoid_img = Mat::default();
        core::bitwise_and(&frame_img, &frame_img, &mut trapezoid_img, &trapezoid_mask)?;

        #[cfg(feature = "gui")]
        {
            highgui::imshow("trapeziodal image", &trapezoid_img)?;
            highgui::wait_key(WAIT_MILLIS)?;
        }

        // trying trapezoidal image for hough lines
        let mut gray_img = Mat::default();
        imgproc::cvt_color(&trapezoid_img, &mut gray_img, imgproc::COLOR_BGR2GRAY, 0)
            .context("BGR2GRAY conversion failed")?;

        #[cfg(feature = "gui")]
        {
            highgui::imshow("gray image", &gray_img)?;
            highgui::wait_key(WAIT_MILLIS)?;
        }

        // Apply Gaussian blur to reduce noise and improve circle detection
        let mut blurr_img = Mat::default();
        imgproc::gaussian_blur(
            &gray_img,
            &mut blurr_img,
            core::Size {
                width: 3,
                height: 3,
            },
            2.0,
            2.0,
            BORDER_DEFAULT,
        )
        .context("Gaussian filter failed")?;

        #[cfg(feature = "gui")]
        {
            highgui::imshow("blurr image", &blurr_img)?;
            highgui::wait_key(WAIT_MILLIS)?;
        }

        let mut canny_img = Mat::default();
        imgproc::canny(&blurr_img, &mut canny_img, 50., 200., 3, false)
            .context("Canny Algorithm failed")?;

        #[cfg(feature = "gui")]
        {
            highgui::imshow("canny image", &canny_img)?;
            highgui::wait_key(WAIT_MILLIS)?;
        }

        let hough_lines =
            probabilistic_hough(&canny_img).context("Standard Hough Transfrom failed")?;
        println!("LINES: {:?}", hough_lines);

        let circles = hough_circles(&blurr_img).context("circles are not created")?; // giving gray scale image to hough circles function

        println!("number of circles detected{}", circles.len());

        // Create an empty black mask for circle detection with the same size size as the input image
        let mut circle_mask = Mat::new_rows_cols_with_default(
            frame_img.rows(),
            frame_img.cols(),
            CV_8UC1,
            Scalar::all(0.0),
        )?;
        // Draw the detected circles on the mask
        for circle in circles.iter() {
            let center = core::Point {
                x: circle[0] as i32,
                y: circle[1] as i32,
            };
            let radius = circle[2] as i32;
            // draw the outer circle
            imgproc::circle(
                &mut circle_mask,
                center,
                radius,
                core::Scalar::all(255.0),
                -1,
                imgproc::LINE_AA,
                0,
            )?;
            // // draw the center of the circle
            // imgproc::circle(
            //     &mut circle_mask,
            //     center,
            //     2,
            //     Scalar::new(0.0, 0.0, 255.0, 0.0),
            //     -1,
            //     imgproc::LINE_AA,
            //     0,
            // )?;
        }

        // Create a result image by bitwise AND-ing the input image with the mask
        let mut circle_image = Mat::default();
        core::bitwise_and(&frame_img, &frame_img, &mut circle_image, &circle_mask)?;
        #[cfg(feature = "gui")]
        {
            highgui::imshow("circles", &circle_image)?;
            highgui::wait_key(WAIT_MILLIS)?;
        }

        let green_light =
            detect_green_light(&circle_image).context("Green light detection failed")?;

        if green_light {
            println!("🟢 🟩 💚 [GREEN]");
        } else {
            println!("🔴 🟥 😡 [RED]");
        }
        let (horizontal, vertical, others) = line_categorization(
            &hough_lines,
            0.01,
            1000.,
            /*curved_slope_range,*/
        );
        #[cfg(feature = "gui")]
        {
            lane_detector(&vertical, &horizontal, frame_img.cols() as f32, &frame_img)
                .context("Lane detection failed")?;
        }
        #[cfg(not(feature = "gui"))]
        {
            lane_detector(&vertical, frame_img.cols() as f32).context("Lane detection failed")?;
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
    let mut canny_img = Mat::default();
    let params = core::Vector::new();

    imgproc::canny(&image, &mut canny_img, 50.0, 150.0, 3, false).context("Canny algo failed")?;
    let time = (core::get_tick_count()? as f64 - strt) / core::get_tick_frequency()?;

    imgcodecs::imwrite(cap_img_path, &image, &params).context("Image saving failed")?;
    imgcodecs::imwrite(edge_img_path, &canny_img, &params).context("Image saving failed")?;

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

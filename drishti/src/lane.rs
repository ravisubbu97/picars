use std::ops::Add;

use anyhow::{Context, Result};

use opencv::{
    core::{self, Point2f, Scalar, BORDER_DEFAULT, DECOMP_LU},
    imgcodecs, imgproc,
    prelude::*,
    types::{VectorOfMat, VectorOfPoint2f},
    videoio::{self, VideoCapture},
};

#[cfg(feature = "gui")]
use crate::eyes::WAIT_MILLIS;
#[cfg(feature = "gui")]
use opencv::highgui;

extern crate opencv;

use opencv::core::TermCriteria;
use opencv::core::KMEANS_RANDOM_CENTERS;

fn create_bar(
    height: i32,
    width: i32,
    color: core::Scalar,
) -> Result<(core::Mat, core::Vec3b), opencv::Error> {
    let bar = core::Mat::new_rows_cols_with_default(height, width, core::CV_8UC3, color)?;
    let color_arr = core::Vec3b::all(color[0] as u8);

    Ok((bar, color_arr))
}

pub fn dominant_color(img: &Mat) -> Result<()> {
    let (height, width) = {
        let size = img.size()?;
        (size.height, size.width)
    };

    let data = img.reshape(1, height * width)?;
    let mut data_converted = Mat::default();
    data.convert_to(&mut data_converted, core::CV_32FC1, 1.0, 0.0)?;

    let criteria = TermCriteria::new(core::TermCriteria_Type::COUNT as i32, 10, 1.0)?;
    let flags = KMEANS_RANDOM_CENTERS;
    let mut labels = Mat::default();
    let mut centers = Mat::default();
    core::kmeans(
        &data_converted,
        3,
        &mut labels,
        criteria,
        10,
        flags,
        &mut centers,
    )?;

    let font = imgproc::FONT_HERSHEY_SIMPLEX;
    let mut bars = VectorOfMat::new();
    let mut rgb_values = Vec::new();

    for i in 0..centers.rows() {
        let row = centers.at_row::<core::Vec3f>(i)?;
        let (bar, rgb) = create_bar(
            200,
            200,
            core::Scalar::new(row[0][0] as f64, row[0][1] as f64, row[0][2] as f64, 0.0),
        )?;
        bars.push(bar);
        rgb_values.push(rgb);
    }

    let mut img_bar = Mat::default();
    core::hconcat(&bars, &mut img_bar)?;

    #[cfg(feature = "gui")]
    {
        for (index, row) in rgb_values.iter().enumerate() {
            imgproc::put_text(
                &mut img_bar,
                &format!("{}. RGB: {:?}", index + 1, row),
                core::Point {
                    x: 5 + 200 * index as i32,
                    y: 200 - 10,
                },
                font,
                0.5,
                Scalar::new(255.0, 0.0, 0.0, 0.0),
                1,
                imgproc::LINE_AA,
                false,
            )?;
            println!("{}. RGB: {:?}", index + 1, row);
        }
        highgui::imshow("Image", &img)?;
        highgui::imshow("Dominant colors", &img_bar)?;

        highgui::wait_key(0)?;
    }

    Ok(())
}

pub fn white_thresholding(img: &Mat) -> Result<Mat> {
    // Change from BGR to HSV image
    let mut hsv_img = Mat::default();
    imgproc::cvt_color(&img, &mut hsv_img, imgproc::COLOR_BGR2HSV, 0)
        .context("COLOR_BGR2HSV conversion failed")?;

    let lower_white = Scalar::new(0.0, 0.0, 0.0, 0.0);
    let upper_white = Scalar::new(0.0, 0.0, 255.0, 0.0);

    let mut masked_white = Mat::default();
    core::in_range(&hsv_img, &lower_white, &upper_white, &mut masked_white)?;

    #[cfg(feature = "gui")]
    {
        // SAVE images
        let params = core::Vector::new();
        imgcodecs::imwrite("images/masked_white.jpg", &masked_white, &params)
            .context("Image saving failed")?;

        highgui::imshow("masked_white image", &masked_white)?;
        highgui::wait_key(WAIT_MILLIS)?;
    }

    Ok(masked_white)
}

pub fn canny_edge_transform(img: &Mat) -> Result<Mat> {
    // Apply Gaussian blur to reduce noise
    let mut blurr_img = Mat::default();
    imgproc::gaussian_blur(
        &img,
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

    // Change from BGR to Gray image
    let mut gray_img = Mat::default();
    imgproc::cvt_color(&blurr_img, &mut gray_img, imgproc::COLOR_BGR2GRAY, 0)
        .context("BGR2GRAY conversion failed")?;

    // Apply canny edge detection
    let mut canny_img = Mat::default();
    imgproc::canny(&gray_img, &mut canny_img, 50., 200., 3, false)
        .context("Canny Algorithm failed")?;

    #[cfg(feature = "gui")]
    {
        // SAVE images
        let params = core::Vector::new();
        imgcodecs::imwrite("images/orig_image.jpg", &img, &params)
            .context("Image saving failed")?;
        imgcodecs::imwrite("images/blurr_img.jpg", &blurr_img, &params)
            .context("Image saving failed")?;
        imgcodecs::imwrite("images/gray_img.jpg", &gray_img, &params)
            .context("Image saving failed")?;
        imgcodecs::imwrite("images/canny_img.jpg", &canny_img, &params)
            .context("Image saving failed")?;

        highgui::imshow("orig image", &img)?;
        highgui::wait_key(WAIT_MILLIS)?;
        highgui::imshow("blurr image", &blurr_img)?;
        highgui::wait_key(WAIT_MILLIS)?;
        highgui::imshow("gray image", &gray_img)?;
        highgui::wait_key(WAIT_MILLIS)?;
        highgui::imshow("canny image", &canny_img)?;
        highgui::wait_key(WAIT_MILLIS)?;
    }

    Ok(canny_img)
}

pub fn warp_perspective_transform(img: &Mat) -> Result<Mat> {
    let src_points = VectorOfPoint2f::from_iter([
        Point2f::new(550.4, 467.999_97),
        Point2f::new(742.399_96, 467.999_97),
        Point2f::new(128., 720.),
        Point2f::new(1280., 720.),
    ]);
    let dst_points = VectorOfPoint2f::from_iter([
        Point2f::new(0.0, 0.0),
        Point2f::new(1280., 0.),
        Point2f::new(0., 720.),
        Point2f::new(1280., 720.),
    ]);

    let h = imgproc::get_perspective_transform(&src_points, &dst_points, DECOMP_LU)?;
    let warped_image_size = img.size()?;

    let mut warped_image = Mat::default();
    imgproc::warp_perspective(
        &img,
        &mut warped_image,
        &h,
        warped_image_size,
        imgproc::INTER_LINEAR,
        core::BORDER_CONSTANT,
        Scalar::default(),
    )
    .context("warp perspective failed")?;

    #[cfg(feature = "gui")]
    {
        // SAVE images
        let params = core::Vector::new();
        imgcodecs::imwrite("images/warped_image_transform.jpg", &warped_image, &params)
            .context("Image saving failed")?;

        highgui::imshow("wrapped image", &warped_image)?;
        highgui::wait_key(WAIT_MILLIS)?;
    }

    Ok(warped_image)
}

pub fn warp_example() -> Result<()> {
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
        let masked_white = white_thresholding(&frame_img)?;
        let canny_img = canny_edge_transform(&frame_img)?;
        let _warped_img = warp_perspective_transform(&canny_img)?;
    }

    Ok(())
}

/// How to run this test:
/// 1. Remove "gui" feature from Cargo.toml file of drishti
/// 2. `cargo test -p drishti -- --nocapture` (to capture print statements)
#[cfg(test)]
mod tests {
    use super::*;
    use opencv::imgcodecs;

    #[test]
    fn test_lane_transform() -> Result<()> {
        let filename = "images/test3.jpg".to_string();
        let filename = core::find_file(&filename, true, false)?;
        let img = imgcodecs::imread(&filename, imgcodecs::IMREAD_COLOR)?;

        let masked_white = white_thresholding(&img)?;
        let canny_img = canny_edge_transform(&img)?;
        let warped_img = warp_perspective_transform(&canny_img)?;
        let params = core::Vector::new();
        imgcodecs::imwrite("images/warped_image_transform.jpg", &warped_img, &params)
            .context("Image saving failed")?;
        imgcodecs::imwrite("images/masked_white.jpg", &masked_white, &params)
            .context("Image saving failed")?;

        println!("Original img size: [{}:{}]", img.cols(), img.rows());
        println!(
            "Wrapped  img size: [{}:{}]",
            warped_img.cols(),
            warped_img.rows()
        );

        Ok(())
    }
}

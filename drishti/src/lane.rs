use anyhow::{Context, Result};

use opencv::{
    core::{self, Point2f, Scalar, BORDER_DEFAULT, DECOMP_LU},
    imgcodecs, imgproc,
    prelude::*,
    types::VectorOfPoint2f,
    videoio::{self, VideoCapture},
};

#[cfg(feature = "gui")]
use crate::eyes::WAIT_MILLIS;
#[cfg(feature = "gui")]
use opencv::highgui;

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

        let canny_img = canny_edge_transform(&img)?;
        let warped_img = warp_perspective_transform(&canny_img)?;
        let params = core::Vector::new();
        imgcodecs::imwrite("images/warped_image_transform.jpg", &warped_img, &params)
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

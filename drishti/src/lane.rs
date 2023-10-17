use anyhow::{Context, Result};

use opencv::{
    calib3d,
    core::{self, Point, Point2f, Scalar, Size, DECOMP_LU},
    imgproc,
    prelude::*,
    types::{VectorOfPoint, VectorOfPoint2f},
};

#[cfg(feature = "gui")]
use crate::eyes::WAIT_MILLIS;
#[cfg(feature = "gui")]
use opencv::highgui;

pub fn warp_perspective_homography(img: &Mat) -> Result<Mat> {
    let cols = img.cols() as f32;
    let rows = img.rows() as f32;

    // Define the warp perspective parameters, such as source and destination corners.
    let mut roi_corners = VectorOfPoint::with_capacity(4);
    roi_corners.push(Point::new((cols / 1.7) as i32, (rows / 4.2) as i32));
    roi_corners.push(Point::new((cols / 1.15) as i32, (rows / 3.32) as i32));
    roi_corners.push(Point::new((cols / 1.33) as i32, (rows / 1.1) as i32));
    roi_corners.push(Point::new((cols / 1.93) as i32, (rows / 1.36) as i32));

    // Define the destination corners for the warped image.
    let mut dst_corners = VectorOfPoint::with_capacity(4);

    let norm1 = (roi_corners.get(0)? - roi_corners.get(1)?).norm();
    let norm2 = (roi_corners.get(2)? - roi_corners.get(3)?).norm();
    let x_max = norm1.max(norm2) as i32;
    let norm3 = (roi_corners.get(1)? - roi_corners.get(2)?).norm();
    let norm4 = (roi_corners.get(3)? - roi_corners.get(0)?).norm();
    let y_max = norm3.max(norm4) as i32;

    dst_corners.push(Point::new(0, 0));
    dst_corners.push(Point::new(x_max, 0));
    dst_corners.push(Point::new(x_max, y_max));
    dst_corners.push(Point::new(0, y_max));

    let roi_corners_mat = Mat::from_exact_iter(roi_corners.iter())?;
    let dst_corners_mat = Mat::from_exact_iter(dst_corners.iter())?;

    let h = calib3d::find_homography(
        &roi_corners_mat,
        &dst_corners_mat,
        &mut Mat::default(),
        0,
        3.,
    )?;

    let warped_image_size = Size::new(dst_corners.get(2)?.x, dst_corners.get(2)?.y);

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
        // Display the warped image
        highgui::named_window("Warped Image", highgui::WINDOW_AUTOSIZE)?;
        highgui::imshow("wrapped image", &warped_image)?;
        highgui::wait_key(WAIT_MILLIS)?;
    }

    Ok(warped_image)
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
        // Display the warped image
        highgui::named_window("Warped Image", highgui::WINDOW_AUTOSIZE)?;
        highgui::imshow("wrapped image", &warped_image)?;
        highgui::wait_key(WAIT_MILLIS)?;
    }

    Ok(warped_image)
}

/// How to run this test:
/// 1. Remove "gui" feature from Cargo.toml file of drishti
/// 2. `cargo test -p drishti -- --nocapture` (to capture print statements)
#[cfg(test)]
mod tests {
    use super::*;
    use opencv::imgcodecs;

    #[test]
    fn test_warp_perspective_homography() -> Result<()> {
        let filename = "images/test3.jpg".to_string();
        let filename = core::find_file(&filename, true, false)?;
        let img = imgcodecs::imread(&filename, imgcodecs::IMREAD_COLOR)?;

        let warped_image = warp_perspective_homography(&img).unwrap();
        let params = core::Vector::new();
        imgcodecs::imwrite("images/warped_image_homography.jpg", &warped_image, &params)
            .context("Image saving failed")?;

        println!("Original img size: [{}:{}]", img.cols(), img.rows());
        println!(
            "Wrapped  img size: [{}:{}]",
            warped_image.cols(),
            warped_image.rows()
        );

        Ok(())
    }

    #[test]
    fn test_warp_perspective_transform() -> Result<()> {
        let filename = "images/test3.jpg".to_string();
        let filename = core::find_file(&filename, true, false)?;
        let img = imgcodecs::imread(&filename, imgcodecs::IMREAD_COLOR)?;

        let warped_image = warp_perspective_transform(&img).unwrap();
        let params = core::Vector::new();
        imgcodecs::imwrite("images/warped_image_transform.jpg", &warped_image, &params)
            .context("Image saving failed")?;

        println!("Original img size: [{}:{}]", img.cols(), img.rows());
        println!(
            "Wrapped  img size: [{}:{}]",
            warped_image.cols(),
            warped_image.rows()
        );

        Ok(())
    }
}

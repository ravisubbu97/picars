use anyhow::{Context, Result};

use opencv::{
    core::{self, Point2f, Scalar, DECOMP_LU},
    imgproc,
    prelude::*,
    types::VectorOfPoint2f,
};

#[cfg(feature = "gui")]
use crate::eyes::WAIT_MILLIS;
#[cfg(feature = "gui")]
use opencv::highgui;

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

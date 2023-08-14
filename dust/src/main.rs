use anyhow::Result;

use drishti::eyes::capture;
use vahana::drive::periodic_run;

fn main() -> Result<()> {
    let image_path = "images/image.jpg";
    capture("1000", image_path);
    let _ = periodic_run();

    Ok(())
}

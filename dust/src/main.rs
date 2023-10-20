use anyhow::{Context, Result};

use drishti::eyes::cv_example_vid;
use drishti::lane::warp_example;

fn main() -> Result<()> {
    warp_example().context("[ERROR] warp example failed")?;
    cv_example_vid().context("[ERROR] cv example failed")?;

    Ok(())
}

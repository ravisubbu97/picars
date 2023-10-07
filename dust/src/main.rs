use anyhow::{Context, Result};

use drishti::eyes::cv_example_vid;

fn main() -> Result<()> {
    cv_example_vid().context("[ERROR] Video capture failed")?;

    Ok(())
}

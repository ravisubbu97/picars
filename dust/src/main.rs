use anyhow::{Context, Result};

use drishti::lane::warp_example;

fn main() -> Result<()> {
    warp_example().context("[ERROR] warp example failed")?;

    Ok(())
}

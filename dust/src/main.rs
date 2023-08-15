use std::{thread::sleep, time::Duration};

use anyhow::Result;

// use drishti::eyes::capture;
use vahana::drive::Motor;

fn main() -> Result<()> {
    // let image_path = "images/image.jpg";
    // capture("1000", image_path);
    let mut motor = Motor::new().expect("Failed to initialize motor.");

    // Example usage
    motor.wheel(50, -1);

    // Sleep for a while to observe the effect
    sleep(Duration::from_secs(5));

    // Turn off the motor
    motor.wheel(0, -1);
    Ok(())
}

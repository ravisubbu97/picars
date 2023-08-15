use std::time::Duration;

use anyhow::Result;

// use drishti::eyes::capture;
use drishti::depth::Ultrasonic;
// use vahana::drive::Motor;

fn main() -> Result<()> {
    // let image_path = "images/image.jpg";
    // capture("1000", image_path);
    // let mut motor = Motor::new().expect("Failed to initialize motor.");

    // // Example usage
    // motor.wheel(50, -1);

    // // Sleep for a while to observe the effect
    // sleep(Duration::from_secs(5));

    // // Turn off the motor
    // motor.wheel(0, -1);
    // Ok(())
    let trig_pin = 27; // D2 (robot-hat)
    let echo_pin = 22; // D3 (robot-hat)
    let timeout = Duration::from_millis(20);

    let mut ultrasonic = Ultrasonic::new(trig_pin, echo_pin, timeout)?;

    let distance = ultrasonic.read(10);
    println!("Distance: {} cm", distance);

    Ok(())
}

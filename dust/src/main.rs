// use std::thread;
// use std::time::Duration;

use anyhow::Result;
// use rppal::gpio::Gpio;
use rppal::system::DeviceInfo;

use drishti::eyes::capture;

// Gpio uses BCM pin numbering. BCM GPIO 23 is tied to physical pin 16.
// const GPIO_LED: u8 = 23;

fn main() -> Result<()> {
    println!("DEVICE: {}", DeviceInfo::new()?.model());

    capture("1000", "../../images/image.jpg");

    Ok(())
}

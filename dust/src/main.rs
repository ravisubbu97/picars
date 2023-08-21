use std::{thread, time::Duration};

use anyhow::Result;

// use drishti::eyes::capture;
use drishti::depth::Ultrasonic;
use vahana::drive::init_i2c;
use vahana::drive::Motor;

fn main() -> Result<()> {
    // let image_path = "images/image.jpg";
    // capture("1000", image_path);
    let rst_pin = dust::recet_mcu().expect("MCU RESET UNSUCCESSFULL [BEGIN]");
    println!("MCU RESET SUCCESSFULLY WITH PIN [{rst_pin}] [BEGIN]");
    let _i2c = init_i2c().expect("I2C INITIALIZED SUCCESSFULLY");

    let mut motor = Motor::new().expect("Failed to initialize motor.");
    println!("motors initialized successfully");

    // Example usage : expectation is both motors will run in forward direction at half speed for 5 secs
    for _ in 0..30 {
        motor.wheel(50.0, -1);
        thread::sleep(Duration::from_secs(1));
    }

    let iterations = 5;
    let trig_pin = 27; // D2 (robot-hat)
    let echo_pin = 22; // D3 (robot-hat)

    let mut ultrasonic = Ultrasonic::new(trig_pin, echo_pin)?;

    for _ in 0..iterations {
        let distance = ultrasonic.read();
        println!("Distance: {} cm", distance);
        // Sleep for 60 milliseconds (as per DATASHEET) --> FIX ME: consider ultrasonic.read() timing into account
        thread::sleep(Duration::from_millis(60));
    }

    let rst_pin = dust::recet_mcu().expect("MCU RESET UNSUCCESSFULL [END]");
    println!("MCU RESET SUCCESSFULLY WITH PIN [{rst_pin}] [END]");

    Ok(())
}

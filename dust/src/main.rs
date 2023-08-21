use std::{thread, time::Duration};

use anyhow::Result;

// use drishti::eyes::capture;
use drishti::depth::Ultrasonic;
use vahana::drive::init_i2c;
use vahana::drive::Motor;

fn main() -> Result<()> {
    // let image_path = "images/image.jpg";
    // capture("1000", image_path);
    let rst_pin = dust::recet_mcu().expect("MCU RESET UNSUCCESSFULL");
    println!("MCU RESET SUCCESSFULLY WITH PIN [{rst_pin}]");
    let _i2c = init_i2c().expect("I2C INITIALIZED SUCCESSFULLY");

    let mut motor = Motor::new().expect("Failed to initialize motor.");
    println!("motors initialized successfully");

    // Example usage : expectation is both motors will run in forward direction at half speed for 5 secs
    motor.wheel(50.0, -1);
    println!(" both motors will run in forward direction");
    thread::sleep(Duration::from_secs(5));

    // motors should stop when speed is 0
    motor.wheel(0.0, -1);
    println!("motors should stop when speed is 0");
    thread::sleep(Duration::from_secs(5));

    // running left motor forward
    motor.wheel(50.0, 0);
    println!(" running left motor forward");
    thread::sleep(Duration::from_secs(5));

    //running right motor forward
    motor.wheel(50.0, 1);
    println!(" running right motor forward");
    thread::sleep(Duration::from_secs(5));

    //running both motors forward
    motor.wheel(50.0, -1);
    println!(" running both motors forward");
    thread::sleep(Duration::from_secs(5));

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

    Ok(())
}

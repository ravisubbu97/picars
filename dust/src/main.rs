use std::{thread, time::Duration};

use anyhow::Result;
use rppal::gpio::Level;

use drishti::depth::Ultrasonic;
use vahana::drive::init_i2c;
use vahana::drive::Motor;

fn main() -> Result<()> {
    // let image_path = "images/image.jpg";
    // capture("1000", image_path);
    let rst_pin = dust::recet_mcu().expect("MCU RESET UNSUCCESSFULL [BEGIN]");
    println!("MCU RESET SUCCESSFULLY WITH PIN [{rst_pin}] [BEGIN]");
    let _i2c = init_i2c().expect("I2C INITIALIZATION FAILED");

    let mut motor = Motor::new().expect("Failed to initialize motor.");
    println!("motors initialized successfully");

    motor.left_rear_pwm_pin.period(1000)?;
    motor.right_rear_pwm_pin.period(1000)?;
    motor.left_rear_pwm_pin.prescaler(10)?;
    motor.right_rear_pwm_pin.prescaler(10)?;

    println!("MOTORS STARTED.......................................");

    motor.left_rear_dir_pin.write(Level::High);
    let _ = motor.left_rear_pwm_pin.pulse_width(1000);
    motor.right_rear_dir_pin.write(Level::Low);
    let _ = motor.right_rear_pwm_pin.pulse_width(1000);
    thread::sleep(Duration::from_secs(2));

    println!("***************************************************");
    motor.left_rear_dir_pin.write(Level::Low);
    let _ = motor.left_rear_pwm_pin.pulse_width(0);
    motor.right_rear_dir_pin.write(Level::High);
    let _ = motor.right_rear_pwm_pin.pulse_width(0);
    thread::sleep(Duration::from_secs(2));

    println!("***************************************************");
    motor.left_rear_dir_pin.write(Level::High);
    let _ = motor.left_rear_pwm_pin.pulse_width(500);
    motor.right_rear_dir_pin.write(Level::Low);
    let _ = motor.right_rear_pwm_pin.pulse_width(500);
    thread::sleep(Duration::from_secs(2));

    println!("MOTORS STOPPED.......................................");

    let trig_pin = 27; // D2 (robot-hat)
    let echo_pin = 22; // D3 (robot-hat)

    let mut ultrasonic = Ultrasonic::new(trig_pin, echo_pin)?;

    for _ in 0..5 {
        let distance = ultrasonic.read();
        println!("Distance: {} cm", distance);
        // Sleep for 60 milliseconds (as per DATASHEET) --> FIX ME: consider ultrasonic.read() timing into account
        thread::sleep(Duration::from_millis(60));
    }

    let rst_pin = dust::recet_mcu().expect("MCU RESET UNSUCCESSFULL [END]");
    println!("MCU RESET SUCCESSFULLY WITH PIN [{rst_pin}] [END]");

    Ok(())
}

use std::{thread, time::Duration};

use anyhow::{Context, Result};

use drishti::{
    depth::Ultrasonic,
    eyes::{camera_backends, cuda_check, cv_example, gapi_check, video_capture},
};
use vahana::{
    drive::{Motors, Servo},
    init_i2c,
};

fn main() -> Result<()> {
    // RESET MCU and INIT I2C
    let rst_pin = dust::recet_mcu().expect("MCU RESET UNSUCCESSFULL [BEGIN]");
    println!("MCU RESET SUCCESSFULLY WITH PIN [{rst_pin}] [BEGIN]");
    let _i2c = init_i2c().expect("I2C INITIALIZATION FAILED");

    // servo
    let mut camera_servo_pin1 = Servo::new(0).context("camera_servo_pin1 init failed")?; // P0
    let mut camera_servo_pin2 = Servo::new(1).context("camera_servo_pin2 init failed")?; // P1
    let mut dir_servo_pin = Servo::new(2).context("dir_servo_pin init failed")?; // P2

    camera_servo_pin1.angle(90)?;
    camera_servo_pin2.angle(90)?;
    dir_servo_pin.angle(45)?;

    // opencv camera example
    gapi_check().context("gapi check failed")?;

    let cuda_available = cuda_check().context("CUDA check failed")?;
    println!(
        "CUDA is {}",
        if cuda_available {
            "available"
        } else {
            "not available"
        }
    );

    let backends = camera_backends().context("unable to get camera backends")?;
    println!("Avalable camera backends: {:?}", backends);

    video_capture().context("Video capture failed")?;

    let time_spent = cv_example("image.jpg", "captured_image.jpg", "edge_image.jpg")?;
    println!(
        "Time spent for image loading and canny edge detection: {} secs",
        time_spent
    );

    // motors
    let mut motors = Motors::new().context("motors init failed")?;
    // set period and prescaler for motors
    motors.left_motor.pwm.period(4095)?;
    motors.left_motor.pwm.prescaler(10)?;
    motors.right_motor.pwm.period(4095)?;
    motors.right_motor.pwm.prescaler(10)?;

    motors.speed(0, 0);
    println!("MOTORS STARTED.......................................");
    motors.forward(50);
    thread::sleep(Duration::from_secs(1));
    motors.stop();
    println!("MOTORS STOPPED.......................................");

    let mut ultrasonic = Ultrasonic::new()?;

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

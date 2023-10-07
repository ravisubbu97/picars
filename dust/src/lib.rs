use std::thread::sleep;
use std::time::Duration;

use anyhow::{Context, Result};
use rppal::gpio::Gpio;

use drishti::depth::Ultrasonic;
use vahana::{
    drive::{Motors, Servo},
    init_i2c,
};

// const BOARD_TYPE: u8 = 12;

// fn check_board_type() -> Result<bool> {
//     let type_pin = Gpio::new()?.get(BOARD_TYPE)?.into_input();

//     Ok(type_pin.is_low())
// }

fn reset_mcu() -> Result<()> {
    // We already know reset pin is 5 -> so skipping functions
    // let rst_pin: u8 = if check_board_type().expect("Error checking board type") {
    //     21
    // } else {
    //     5
    // };
    let mut rst_pin = Gpio::new()?.get(5)?.into_output();

    rst_pin.set_low();
    sleep(Duration::from_millis(1));
    rst_pin.set_high();
    sleep(Duration::from_millis(1));

    Ok(())
}

pub fn main_init() -> Result<()> {
    // RESET MCU
    reset_mcu().context("MCU RESET UNSUCCESSFULL [BEGIN]")?;
    // INIT I2C
    init_i2c().context("I2C INITIALIZATION FAILED")?;

    Ok(())
}

pub fn servos_init(init_angles: [i32; 3]) -> Result<[Servo; 3]> {
    let mut camera_servo_pin1 = Servo::new(0).context("camera_servo_pin1 init failed")?; // P0
    let mut camera_servo_pin2 = Servo::new(1).context("camera_servo_pin2 init failed")?; // P1
    let mut dir_servo_pin = Servo::new(2).context("dir_servo_pin init failed")?; // P2
    camera_servo_pin1.angle(init_angles[0]);
    camera_servo_pin2.angle(init_angles[1]);
    dir_servo_pin.angle(init_angles[2]);

    Ok([camera_servo_pin1, camera_servo_pin2, dir_servo_pin])
}

pub fn motors_init(period: u16, prescaler: u16) -> Result<Motors> {
    let mut motors = Motors::new().context("motors init failed")?;
    // set period and prescaler for motors
    motors.left_motor.pwm.period(period)?;
    motors.left_motor.pwm.prescaler(prescaler)?;
    motors.right_motor.pwm.period(period)?;
    motors.right_motor.pwm.prescaler(prescaler)?;

    Ok(motors)
}

pub fn ultrasonic_init() -> Result<Ultrasonic> {
    let ultrasonic = Ultrasonic::new().context("context")?;

    Ok(ultrasonic)
}

pub fn scratchpad() -> Result<()> {
    // cv_example_vid().context("something in video failed")?;

    // let time_spent = cv_example_photo("image.jpg", "captured_image.jpg", "edge_image.jpg")?;
    // println!(
    //     "Time spent for image loading and canny edge detection: {} secs",
    //     time_spent
    // // );

    // motors.speed(0, 0);
    // println!("MOTORS STARTED.......................................");
    // motors.forward(50);
    // thread::sleep(Duration::from_secs(1));
    // motors.stop();
    // println!("MOTORS STOPPED.......................................");

    // for _ in 0..5 {
    //     let distance = ultrasonic.read();
    //     println!("Distance: {} cm", distance);
    //     // Sleep for 60 milliseconds (as per DATASHEET) --> FIX ME: consider ultrasonic.read() timing into account
    //     thread::sleep(Duration::from_millis(60));
    // }

    // let rst_pin = reset_mcu().expect("MCU RESET UNSUCCESSFULL [END]");

    Ok(())
}
/*
===============================================
PIN REFERENCES FROM ROBOT-HAT (BCM NUMBERS)
===============================================
    "D0":  17
    "D1":   4
    "D2":  27
    "D3":  22
    "D4":  23
    "D5":  24
    "D6":  25
    "D7":   4
    "D8":   5
    "D9":   6
    "D10": 12
    "D11": 13
    "D12": 19
    "D13": 16
    "D14": 26
    "D15": 20
    "D16": 21
    "SW":  25
    "USER": 25
    "LED": 26
    "BOARD_TYPE": 12
    "RST": 16
    "BLEINT": 13
    "BLERST": 20
    "MCURST":  5
===============================================
*/

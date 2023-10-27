// rustimport:pyo3

pub mod depth;
pub mod drive;
pub mod i2c_pwm;

use pyo3::prelude::*;

use std::thread::sleep;
use std::time::Duration;

use anyhow::{Context, Result};
use rppal::gpio::Gpio;

use depth::Ultrasonic;
use drive::{Motors, Servo};
use i2c_pwm::init_i2c;

#[pyfunction]
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

#[pyfunction]
pub fn main_init() -> Result<()> {
    // RESET MCU
    reset_mcu().context("MCU RESET UNSUCCESSFULL [BEGIN]")?;
    // INIT I2C
    init_i2c().context("I2C INITIALIZATION FAILED")?;

    Ok(())
}

#[pyfunction]
pub fn servos_init(init_angles: [i32; 3]) -> Result<[Servo; 3]> {
    let mut camera_servo_pin1 = Servo::new(0).context("camera_servo_pin1 init failed")?; // P0
    let mut camera_servo_pin2 = Servo::new(1).context("camera_servo_pin2 init failed")?; // P1
    let mut dir_servo_pin = Servo::new(2).context("dir_servo_pin init failed")?; // P2
    camera_servo_pin1.angle(init_angles[0]);
    camera_servo_pin2.angle(init_angles[1]);
    dir_servo_pin.angle(init_angles[2]);

    Ok([camera_servo_pin1, camera_servo_pin2, dir_servo_pin])
}

#[pyfunction]
pub fn motors_init(period: u16, prescaler: u16) -> Result<Motors> {
    let mut motors = Motors::new().context("motors init failed")?;
    // set period and prescaler for motors
    motors.left_motor.pwm.period(period)?;
    motors.left_motor.pwm.prescaler(prescaler)?;
    motors.right_motor.pwm.period(period)?;
    motors.right_motor.pwm.prescaler(prescaler)?;

    Ok(motors)
}

#[pyfunction]
pub fn ultrasonic_init() -> Result<Ultrasonic> {
    let ultrasonic = Ultrasonic::new().context("context")?;

    Ok(ultrasonic)
}

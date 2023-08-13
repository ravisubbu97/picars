use std::thread;
use std::time::Duration;

use anyhow::Result;
use rppal::gpio::Gpio;

// Gpio uses BCM pin numbering. BCM GPIO 23 is tied to physical pin 16.
const LEFT_RED: u8 = 23;
const LEFT_BLACK: u8 = 24;

const RIGHT_RED: u8 = 25;
const RIGHT_BLACK: u8 = 8;

pub fn periodic_run() -> Result<()> {
    let mut left_red = Gpio::new()?.get(LEFT_RED)?.into_output();
    let mut left_black = Gpio::new()?.get(LEFT_BLACK)?.into_output();
    let mut right_red = Gpio::new()?.get(RIGHT_RED)?.into_output();
    let mut right_black = Gpio::new()?.get(RIGHT_BLACK)?.into_output();

    left_red.set_high();
    right_red.set_high();
    thread::sleep(Duration::from_millis(6000));
    left_black.set_low();
    right_black.set_low();
    thread::sleep(Duration::from_millis(6000));

    left_black.set_high();
    right_black.set_high();
    thread::sleep(Duration::from_millis(6000));
    left_red.set_low();
    right_red.set_low();
    thread::sleep(Duration::from_millis(6000));
    Ok(())
}
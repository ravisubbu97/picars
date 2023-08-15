use std::thread;
use std::time::Duration;

use anyhow::Result;
use rppal::gpio::Gpio;

const LEFT_RED: u8 = 4;  // connecting to GPIO pin 4 as per sun founder module
// const LEFT_BLACK: u8 = 24;

const RIGHT_RED: u8 = 5; // connecting to GPIO pin 5 as per sun founder module
// const RIGHT_BLACK: u8 = 8;

pub fn periodic_run() -> Result<()> {
    let mut left_red = Gpio::new()?.get(LEFT_RED)?.into_output();
    let mut right_red = Gpio::new()?.get(RIGHT_RED)?.into_output();

    left_red.set_high();
    right_red.set_high();
    thread::sleep(Duration::from_millis(6000));

    left_red.set_low();
    right_red.set_low();
    thread::sleep(Duration::from_millis(6000));

    Ok(())
}

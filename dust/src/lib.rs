use std::thread::sleep;
use std::time::Duration;

use anyhow::Result;
use rppal::gpio::Gpio;

const BOARD_TYPE: u8 = 12;

fn check_board_type() -> Result<bool> {
    let type_pin = Gpio::new()?.get(BOARD_TYPE)?.into_input();

    Ok(type_pin.is_low())
}

pub fn recet_mcu() -> Result<u8> {
    let rst_pin: u8 = if check_board_type().expect("Error checking board type") {
        21
    } else {
        5
    };
    let mut rst_pin = Gpio::new()?.get(rst_pin)?.into_output();

    rst_pin.set_low();
    sleep(Duration::from_millis(1));
    rst_pin.set_high();
    sleep(Duration::from_millis(1));

    Ok(rst_pin.pin())
}

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

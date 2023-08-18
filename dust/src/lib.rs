use std::thread::sleep;
use std::time::Duration;

use anyhow::Result;
use rppal::gpio::Gpio;
use rppal::i2c::I2c;

const BUS: u8 = 1;
const SLAVE_ADDR: u16 = 0x14;
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

pub fn init_i2c() -> Result<I2c> {
    let mut i2c = I2c::with_bus(BUS).expect("I2C Initialization Failure");
    i2c.set_slave_address(SLAVE_ADDR)?;
    i2c.smbus_send_byte(0x2C)?;
    i2c.smbus_send_byte(0)?;
    i2c.smbus_send_byte(0)?;

    Ok(i2c)
}

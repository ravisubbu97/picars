use anyhow::{Context, Result};
use std::{thread::sleep, time::Duration};

use rppal::{
    gpio::{Gpio, Level, OutputPin},
    i2c::I2c,
};

const BUS: u8 = 1;
const REG_CHN: u8 = 0x20;
// const REG_FRE: u8 = 0x30;
const REG_PSC: u8 = 0x40;
const REG_ARR: u8 = 0x44;
const SLAVE_ADDR: u16 = 0x14;
const CLOCK: u32 = 72_000_000;

pub fn init_i2c() -> Result<I2c> {
    let mut i2c = I2c::with_bus(BUS)?;
    // wait after I2C init to avopid 121 IO error
    sleep(Duration::from_secs(1));

    i2c.set_slave_address(SLAVE_ADDR)?;
    i2c.smbus_send_byte(0x2C)?;
    i2c.smbus_send_byte(0)?;
    i2c.smbus_send_byte(0)?;

    Ok(i2c)
}

pub struct PWM {
    channel: u8,
    bus: I2c,
}

impl PWM {
    pub fn new(channel: u8) -> Result<Self> {
        let i2c = init_i2c().context("PWM I2C INIT FAILED")?;
        let mut pwm = Self { channel, bus: i2c };

        pwm.freq(50).context("PWM FREQ INIT FAILED")?;

        Ok(pwm)
    }

    pub fn freq(&mut self, freq: u16) -> Result<()> {
        let mut result_psc = Vec::with_capacity(12); // Create a vector for prescaler
        let mut result_per = Vec::with_capacity(12); // Create a vector for period
        let mut result_acy = Vec::with_capacity(12); // Create a vector for accuracy

        let st = ((CLOCK as f32 / freq as f32).sqrt() as u16) - 5;

        for psc in st..st + 10 {
            let per = (CLOCK / (freq * psc) as u32) as u16;
            result_psc.push(psc);
            result_per.push(per);
            result_acy.push(f32::abs(freq as f32 - CLOCK as f32 / (psc * per) as f32));
        }

        let i = result_acy
            .iter()
            .position(|&x| x == result_acy.iter().cloned().fold(f32::INFINITY, f32::min))
            .unwrap();
        let psc = result_psc[i];
        let per = result_per[i];

        self.prescaler(psc).context("PWM PRESCALER INIT FAILED")?;
        self.period(per).context("PWM PERIOD INIT FAILED")?;

        Ok(())
    }

    pub fn prescaler(&mut self, prescaler: u16) -> Result<()> {
        let prescaler = prescaler - 1;
        let timer = self.channel / 4_u8;
        let reg = REG_PSC + timer;
        self.bus
            .smbus_write_word(reg, prescaler)
            .context("PWM PRESCALER SEND FAILED")?;

        Ok(())
    }

    pub fn period(&mut self, arr: u16) -> Result<()> {
        let timer = arr - 1;
        let reg = REG_ARR + timer as u8;
        self.bus
            .smbus_write_word(reg, timer)
            .context("PWM PERIOD SEND FAILED")?;

        Ok(())
    }

    pub fn pulse_width(&mut self, pulse_width: u16) -> Result<()> {
        let reg = REG_CHN + self.channel;
        self.bus
            .smbus_write_word(reg, pulse_width)
            .context("PWM PULSE WIDTH SEND FAILED")?;

        Ok(())
    }

    // This code is buggy, FIX ME !!
    pub fn pulse_width_percent(&mut self, pulse_width_percent: f32) -> Result<()> {
        let temp = pulse_width_percent / 100.0;
        let timer = self.channel / 4_u8;
        let pulse_width = (temp * timer as f32) as u16;
        self.pulse_width(pulse_width)?;

        Ok(())
    }
}

pub struct Motor {
    pub left_rear_pwm_pin: PWM,
    pub right_rear_pwm_pin: PWM,
    pub left_rear_dir_pin: OutputPin,
    pub right_rear_dir_pin: OutputPin,
}

impl Motor {
    pub fn new() -> Result<Self> {
        let gpio = Gpio::new().context("Gpio init failed (drive)")?;

        let left_rear_pwm_pin = PWM::new(13).context("PWM 13 init failed")?; // P13 (robot-hat)
        let right_rear_pwm_pin = PWM::new(12).context("PWM 12 init failed")?; // P12 (robot-hat)
        let left_rear_dir_pin = gpio.get(23).context("Gpio 23 init failed")?.into_output(); // D4 (robot-hat)
        let right_rear_dir_pin = gpio.get(24).context("Gpio 24 init failed")?.into_output(); // D5 (robot-hat)

        Ok(Self {
            left_rear_pwm_pin,
            right_rear_pwm_pin,
            left_rear_dir_pin,
            right_rear_dir_pin,
        })
    }

    // Control motor direction and speed
    // motor 0 or 1,
    // dir   0 or 1
    // speed 0 ~ 100
    pub fn wheel(&mut self, speed: f32, motor: i32) {
        let dir = if speed > 0.0 { Level::High } else { Level::Low };
        let mut speed = speed.abs();
        if speed != 0.0 {
            speed = speed / 2.0 + 50.0;
        }

        match motor {
            0 => {
                self.left_rear_dir_pin.write(dir);
                let _ = self.left_rear_pwm_pin.pulse_width_percent(speed);
            }
            1 => {
                self.right_rear_dir_pin.write(dir);
                let _ = self.right_rear_pwm_pin.pulse_width_percent(speed);
            }
            -1 => {
                self.left_rear_dir_pin.write(dir);
                let _ = self.left_rear_pwm_pin.pulse_width_percent(speed);
                self.right_rear_dir_pin.write(!dir);
                let _ = self.right_rear_pwm_pin.pulse_width_percent(speed);
            }
            _ => panic!("MOTOR SUCKS !!"),
        }
    }
}

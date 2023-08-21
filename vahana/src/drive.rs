use anyhow::{Context, Result};
use std::{thread::sleep, time::Duration};

use rppal::{
    gpio::{self, Gpio, OutputPin},
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
    sleep(Duration::from_secs(5));

    i2c.set_slave_address(SLAVE_ADDR)?;
    i2c.smbus_send_byte(0x2C)?;
    i2c.smbus_send_byte(0)?;
    i2c.smbus_send_byte(0)?;

    Ok(i2c)
}

pub struct PWM {
    channel: u8,
    timer: u8,
    bus: I2c,
    _pulse_width: u8,
    _freq: u16,
    _prescaler: u8,
    _pulse_width_percent: f32,
}

impl PWM {
    pub fn new(channel: u8) -> Result<Self> {
        let freq = 50;
        let i2c = init_i2c().context("PWM I2C INIT FAILED")?;
        let mut pwm = Self {
            channel,
            timer: channel / 4_u8,
            bus: i2c,
            _pulse_width: 0,
            _freq: freq,
            _prescaler: 0,
            _pulse_width_percent: 0.0,
        };

        pwm.freq(50).context("PWM FREQ INIT FAILED")?;

        Ok(pwm)
    }

    pub fn freq(&mut self, freq: u16) -> Result<()> {
        self._freq = freq;
        let mut result_ap = vec![]; // Create a vector for prescaler
        let mut result_acy = vec![]; // Create a vector for accuracy

        let st = (CLOCK as f32 / self._freq as f32).sqrt() as u32 - 5;
        let st = st.max(1); // Prevent negative value

        for psc in st..st + 10 {
            let arr = (CLOCK / (self._freq * psc as u16) as u32) as u8;
            result_ap.push((psc, arr));
            result_acy.push(f32::abs(
                self._freq as f32 - CLOCK as f32 / (psc * arr as u32) as f32,
            ));
        }

        let i = result_acy
            .iter()
            .position(|&x| x == result_acy.iter().cloned().fold(f32::INFINITY, f32::min))
            .unwrap();
        let (psc, arr) = result_ap[i];

        self.prescaler(psc as u8)
            .context("PWM PRESCALER INIT FAILED")?;
        self.period(arr).context("PWM PERIOD INIT FAILED")?;

        Ok(())
    }

    pub fn prescaler(&mut self, prescaler: u8) -> Result<()> {
        self._prescaler = prescaler - 1;
        let reg = REG_PSC + self.timer;
        self.bus
            .smbus_write_word(reg, self._prescaler as u16)
            .context("PWM PRESCALER SEND FAILED")?;

        Ok(())
    }

    pub fn period(&mut self, arr: u8) -> Result<()> {
        let timer = arr - 1;
        let reg = REG_ARR + timer;
        self.bus
            .smbus_write_word(reg, timer as u16)
            .context("PWM PERIOD SEND FAILED")?;

        Ok(())
    }

    pub fn pulse_width(&mut self, pulse_width: u8) -> Result<()> {
        self._pulse_width = pulse_width;
        let reg = REG_CHN + self.channel;
        self.bus
            .smbus_write_word(reg, self._pulse_width as u16)
            .context("PWM PULSE WIDTH SEND FAILED")?;

        Ok(())
    }

    pub fn pulse_width_percent(&mut self, pulse_width_percent: f32) -> Result<()> {
        let temp = pulse_width_percent / 100.0;
        let pulse_width = (temp * self.timer as f32) as u8;
        self.pulse_width(pulse_width)?;

        Ok(())
    }
}

pub struct Motor {
    left_rear_pwm_pin: PWM,
    right_rear_pwm_pin: PWM,
    left_rear_dir_pin: OutputPin,
    right_rear_dir_pin: OutputPin,
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
        let dir = if speed > 0.0 {
            gpio::Level::High
        } else {
            gpio::Level::Low
        };
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
                self.right_rear_dir_pin.write(dir);
                let _ = self.right_rear_pwm_pin.pulse_width_percent(speed);
            }
            _ => panic!("MOTOR SUCKS !!"),
        }
    }
}

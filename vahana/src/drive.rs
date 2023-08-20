use anyhow::Result;
use rppal::gpio::{self, Gpio, OutputPin};
use rppal::pwm::{Channel, Polarity, Pwm};
use std::time::Duration;

// Period: 20 ms (50 Hz). Pulse width: min. 1200 µs, neutral 1500 µs, max. 1800 µs.
const PERIOD_MS: u64 = 20;
// const PULSE_MIN_US: u64 = 1000;
// const PULSE_NEUTRAL_US: u64 = 1500;
const PULSE_MAX_US: u64 = 2000;

pub struct Motor {
    left_rear_pwm_pin: Pwm,
    right_rear_pwm_pin: Pwm,
    left_rear_dir_pin: OutputPin,
    right_rear_dir_pin: OutputPin,
}

impl Motor {
    pub fn new() -> Result<Self> {
        let gpio = Gpio::new()?;

        let left_rear_pwm_pin = Pwm::with_period(
            Channel::Pwm0, // P13 (robot-hat)
            Duration::from_millis(PERIOD_MS),
            Duration::from_micros(PULSE_MAX_US),
            Polarity::Normal,
            true,
        )?;
        let right_rear_pwm_pin = Pwm::with_period(
            Channel::Pwm1, // P12 (robot-hat)
            Duration::from_millis(PERIOD_MS),
            Duration::from_micros(PULSE_MAX_US),
            Polarity::Normal,
            true,
        )?;
        let left_rear_dir_pin = gpio.get(23)?.into_output(); // D4 (robot-hat)
        let right_rear_dir_pin = gpio.get(24)?.into_output(); // D5 (robot-hat)

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
    pub fn wheel(&mut self, speed: i32, motor: i32) {
        let dir = if speed > 0 {
            gpio::Level::High
        } else {
            gpio::Level::Low
        };
        let mut speed = speed.abs();
        if speed != 0 {
            speed = speed / 2 + 50;
        }

        match motor {
            0 => {
                self.left_rear_dir_pin.write(dir);
                let _ = self.left_rear_pwm_pin.set_duty_cycle(speed as f64);
            }
            1 => {
                self.right_rear_dir_pin.write(dir);
                let _ = self.right_rear_pwm_pin.set_duty_cycle(speed as f64);
            }
            -1 => {
                self.left_rear_dir_pin.write(dir);
                let _ = self.left_rear_pwm_pin.set_duty_cycle(speed as f64);
                self.right_rear_dir_pin.write(dir);
                let _ = self.right_rear_pwm_pin.set_duty_cycle(speed as f64);
            }
            _ => panic!("MOTOR SUCKS !!"),
        }
    }
}

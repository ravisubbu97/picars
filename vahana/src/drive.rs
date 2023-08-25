use anyhow::{Context, Result};
use rppal::gpio::{Gpio, Level, OutputPin};

use crate::{mapping, PWM};

// Servo and Motor Constants
const PERIOD: u16 = 4095;
const PRESCALER: u16 = 10;
const FREQ: u16 = 50;
const MAX_PW: u16 = 2500;
const MIN_PW: u16 = 500;
const CLOCK: u32 = 72_000_000;

pub struct Motor {
    pwm: PWM,
    dir: OutputPin,
    pub speed: f32,
}

impl Motor {
    pub fn new(pwm_pin: u8, dir_pin: u8) -> Result<Self> {
        let gpio = Gpio::new().context("Gpio init failed (drive)")?;

        let mut pwm = PWM::new(pwm_pin).context("PWM init failed")?;
        let dir = gpio.get(dir_pin).context("Gpio init failed")?.into_output();
        let speed = 0.0;

        pwm.period(PERIOD)?;
        pwm.prescaler(PRESCALER)?;
        Ok(Self { pwm, dir, speed })
    }

    pub fn speed(&mut self, speed: f32) -> Result<()> {
        let dir: Level = if speed > 0.0 { Level::High } else { Level::Low };
        let speed: f32 = speed.abs();

        self.pwm.pulse_width_percent(speed)?;
        self.dir.write(dir);

        Ok(())
    }
}

pub struct Motors {
    left_motor: Motor,
    right_motor: Motor,
}

impl Motors {
    pub fn new() -> Result<Self> {
        let left_motor_pwm_pin: u8 = 13; // P13 (robot-hat)
        let left_motor_dir_pin: u8 = 23; // D4 (robot-hat)
        let right_motor_pwm_pin: u8 = 12; // P12 (robot-hat)
        let right_motor_dir_pin: u8 = 24; // D5 (robot-hat)

        let left_motor =
            Motor::new(left_motor_pwm_pin, left_motor_dir_pin).context("LEFT MOTOR INIT FAILED")?;
        let right_motor = Motor::new(right_motor_pwm_pin, right_motor_dir_pin)
            .context("RIGHT MOTOR INIT FAILED")?;

        Ok(Self {
            left_motor,
            right_motor,
        })
    }

    pub fn stop(&mut self) {
        let _ = self.left_motor.speed(0.0);
        let _ = self.right_motor.speed(0.0);
    }

    pub fn speed(&mut self, left_speed: f32, right_speed: f32) {
        let _ = self.left_motor.speed(left_speed);
        let _ = self.right_motor.speed(right_speed);
    }

    pub fn forward(&mut self, speed: f32) {
        self.speed(speed, speed);
    }

    pub fn backward(&mut self, speed: f32) {
        self.speed(-speed, -speed);
    }

    pub fn turn_left(&mut self, speed: f32) {
        self.speed(-speed, speed);
    }

    pub fn turn_right(&mut self, speed: f32) {
        self.speed(speed, -speed);
    }
}

pub struct Servo {
    pwm: PWM,
}

impl Servo {
    pub fn new(pwm_pin: u8) -> Result<Self> {
        let mut pwm = PWM::new(pwm_pin).context("PWM init failed")?;
        let prescaler: u16 = (CLOCK / FREQ as u32 / PERIOD as u32) as u16;
        pwm.period(PERIOD)?;
        pwm.prescaler(prescaler)?;
        Ok(Self { pwm })
    }

    pub fn pulse_width_time(&mut self, pw_time: f32) -> Result<()> {
        let pw_time = pw_time.clamp(MIN_PW.into(), MAX_PW.into());
        let pwr = pw_time / 20000.0;
        let value = (pwr * PERIOD as f32) as u16;
        self.pwm.pulse_width(value)?;

        Ok(())
    }

    pub fn angle(&mut self, angle: f32) -> Result<()> {
        let angle = angle.clamp(-90.0, 90.0);
        let pw_time = mapping(angle, -90.0, 90.0, MIN_PW.into(), MAX_PW.into());
        let _ = self.pulse_width_time(pw_time);

        Ok(())
    }
}

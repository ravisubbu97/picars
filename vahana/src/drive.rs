use anyhow::{Context, Result};
use rppal::gpio::{Gpio, Level, OutputPin};

use crate::{map_range, PWM};

// Servo and Motor Constants
const PERIOD: u16 = 1200;
const PRESCALER: u16 = 1200;
// const FREQ: u16 = 50;
const MAX_PW: u16 = 2500;
const MIN_PW: u16 = 500;
// const CLOCK: u32 = 72_000_000;

pub struct Motor {
    pub pwm: PWM,
    pub dir: OutputPin,
}

impl Motor {
    pub fn new(pwm_pin: u8, dir_pin: u8) -> Result<Self> {
        let gpio = Gpio::new().context("Gpio init failed (drive)")?;

        let mut pwm = PWM::new(pwm_pin).context("PWM init failed")?;
        let dir = gpio.get(dir_pin).context("Gpio init failed")?.into_output();

        pwm.period(PERIOD)?;
        pwm.prescaler(PRESCALER)?;
        Ok(Self { pwm, dir })
    }

    pub fn speed(&mut self, speed: i32) -> Result<()> {
        let dir: Level = if speed > 0 { Level::High } else { Level::Low };

        self.pwm.pulse_width_percent(speed.unsigned_abs())?;
        self.dir.write(dir);

        Ok(())
    }
}

pub struct Motors {
    pub left_motor: Motor,
    pub right_motor: Motor,
}

impl Motors {
    pub fn new() -> Result<Self> {
        let left_motor_pwm_pin: u8 = 12; // P12 (robot-hat)
        let left_motor_dir_pin: u8 = 23; // D4 (robot-hat)
        let right_motor_pwm_pin: u8 = 13; // P13 (robot-hat)
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
        let _ = self.left_motor.speed(0);
        let _ = self.right_motor.speed(0);
    }

    pub fn speed(&mut self, left_speed: i32, right_speed: i32) {
        let _ = self.left_motor.speed(left_speed);
        let _ = self.right_motor.speed(-right_speed); // Negating as per robot-hat python module
    }

    pub fn forward(&mut self, speed: i32) {
        self.speed(speed, speed);
    }

    pub fn backward(&mut self, speed: i32) {
        self.speed(-speed, -speed);
    }

    pub fn turn_left(&mut self, speed: i32) {
        self.speed(-speed, speed);
    }

    pub fn turn_right(&mut self, speed: i32) {
        self.speed(speed, -speed);
    }
}

pub struct Servo {
    pwm: PWM,
}

impl Servo {
    pub fn new(pwm_pin: u8) -> Result<Self> {
        let mut pwm = PWM::new(pwm_pin).context("PWM init failed")?;
        pwm.period(4095)?; // ref: robot-hat
        pwm.prescaler(351)?; // ref: robot-hat --> (CPU_CLOCK / FREQ / PERIOD )

        Ok(Self { pwm })
    }

    pub fn pulse_width_time(&mut self, pw_time: i32) -> Result<()> {
        let value = ((pw_time * 4095) / 20000) as u16; // 20,000 us --> 20ms (50Hz signal for servo)
        self.pwm.pulse_width(value)?;

        Ok(())
    }

    pub fn angle(&mut self, angle: i32) {
        let angle = angle.clamp(-90, 90);
        let pw_time = map_range((-90, 90), (MIN_PW.into(), MAX_PW.into()), angle);
        let pw_time = pw_time.clamp(MIN_PW.into(), MAX_PW.into());
        let _ = self.pulse_width_time(pw_time);
    }
}

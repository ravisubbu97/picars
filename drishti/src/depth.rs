use std::thread::sleep;
use std::time::{Duration, Instant};

use anyhow::Result;
use rppal::gpio::{Gpio, InputPin, Level, OutputPin};

pub struct Ultrasonic {
    trig: OutputPin,
    echo: InputPin,
    timeout: Duration,
}

impl Ultrasonic {
    pub fn new(trig_pin: u8, echo_pin: u8, timeout: Duration) -> Result<Self> {
        let trig = Gpio::new()?.get(trig_pin)?.into_output();
        let echo = Gpio::new()?.get(echo_pin)?.into_input();

        Ok(Ultrasonic {
            trig,
            echo,
            timeout,
        })
    }

    fn _read(&mut self) -> Result<u64, &'static str> {
        self.trig.set_low();
        sleep(Duration::from_millis(10));

        self.trig.set_high();
        sleep(Duration::from_micros(10));
        self.trig.set_low();

        let pulse_start = Instant::now();
        while self.echo.read() == Level::Low {
            if pulse_start.elapsed() > self.timeout {
                return Err("Timeout");
            }
        }

        let pulse_end = Instant::now();
        while self.echo.read() == Level::High {
            if pulse_end.elapsed() > self.timeout {
                return Err("Timeout");
            }
        }

        let time_taken = pulse_end.duration_since(pulse_start).as_secs();
        let cm = time_taken * 340 / 2 * 100;
        Ok(cm)
    }

    pub fn read(&mut self, times: u32) -> u64 {
        for _ in 0..times {
            if let Ok(a) = self._read() {
                return a;
            }
        }
        0
    }
}

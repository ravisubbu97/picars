pub mod axel;
pub mod drive;
pub mod neck;

use anyhow::{Context, Result};
use std::{process::Command, thread::sleep, time::Duration};

use rppal::i2c::I2c;

const I2C_BUS: u8 = 1;
const REG_PW: u8 = 0x20; // REG_CHN
const REG_PSC: u8 = 0x40; // REG_PSC
const REG_PER: u8 = 0x44; // REG_ARR
const SLAVE_ADDR: u16 = 0x14;
const CLOCK: u32 = 72_000_000;

fn mapping(x: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min
}

pub fn init_i2c() -> Result<I2c> {
    let mut i2c = I2c::with_bus(I2C_BUS)?;
    // wait after I2C init to avopid 121 IO error
    sleep(Duration::from_secs(1));

    i2c.set_slave_address(SLAVE_ADDR)?;
    i2c.smbus_send_byte(0x2C)?;
    i2c.smbus_send_byte(0x00)?;
    i2c.smbus_send_byte(0x00)?;

    Ok(i2c)
}

fn run_command(cmd: &str) -> Result<(i32, String), Box<dyn std::error::Error>> {
    let output = Command::new("sh") // You can use "sh" to execute shell commands
        .arg("-c") // Use the -c flag to run the provided command
        .arg(cmd) // The command you want to run
        .output()?;

    let status = output.status.code().unwrap_or(-1);
    let result = String::from_utf8_lossy(&output.stdout).to_string();

    Ok((status, result))
}

pub fn scan_i2c(i2c: I2c) -> Vec<u16> {
    let cmd = format!("i2cdetect -y {}", i2c.bus());
    let output = match run_command(&cmd) {
        Ok((status, result)) => {
            println!("Exit Status: {}", status);
            println!("Command Output:\n{}", result);
            result
        }
        Err(err) => err.to_string(),
    };

    let mut addresses = vec![];

    for line in output.lines().skip(1) {
        let tmp_addresses = line.split(':').nth(1).unwrap_or("").trim();
        for address in tmp_addresses.split_whitespace() {
            if address != "--" {
                if let Ok(address) = u16::from_str_radix(address, 16) {
                    addresses.push(address);
                }
            }
        }
    }

    addresses
}

pub struct PWM {
    channel: u8,
    bus: I2c,
}

impl PWM {
    pub fn new(channel: u8) -> Result<Self> {
        let bus = init_i2c().context("PWM I2C INIT FAILED")?;
        let mut pwm = Self { channel, bus };

        pwm.freq(50).context("PWM FREQ INIT FAILED")?;

        Ok(pwm)
    }

    pub fn freq(&mut self, freq: u16) -> Result<()> {
        /*  Buggy code: For now, we hardcode the values
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
        */
        let psc: u16 = freq * 24; // 1200
        let per: u16 = freq * 24; // 1200

        self.prescaler(psc).context("PWM PRESCALER INIT FAILED")?;
        self.period(per).context("PWM PERIOD INIT FAILED")?;

        Ok(())
    }

    pub fn prescaler(&mut self, prescaler: u16) -> Result<()> {
        let timer = self.channel / 4_u8;
        let reg = REG_PSC + timer;
        self.bus
            .smbus_write_word(reg, (prescaler - 1).swap_bytes())
            .context("PWM PRESCALER SEND FAILED")?;

        Ok(())
    }

    pub fn period(&mut self, per: u16) -> Result<()> {
        let timer = self.channel / 4_u8;
        let reg = REG_PER + timer;
        self.bus
            .smbus_write_word(reg, per.swap_bytes())
            .context("PWM PERIOD SEND FAILED")?;

        Ok(())
    }

    pub fn pulse_width(&mut self, pw: u16) -> Result<()> {
        let reg = REG_PW + self.channel;
        self.bus
            .smbus_write_word(reg, pw.swap_bytes())
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

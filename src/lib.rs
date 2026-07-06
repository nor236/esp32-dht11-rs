#![no_std]

use embedded_hal::delay::DelayNs;
use esp_hal::gpio::{DriveMode, Flex, InputConfig, OutputConfig, Pin};
use esp_hal::time::Instant;

#[derive(Debug)]
pub enum SensorError {
    ChecksumMismatch,
    Timeout,
    PinError,
}

#[derive(Debug, Copy, Clone)]
pub struct Reading {
    pub humidity: u8,
    pub temperature: i8,
}

pub struct DHT11<'a, D> {
    pub pin: Flex<'a>,
    pub delay: D,
}

const TIMEOUT_DURATION: u64 = 1000;
const BIT_TIMEOUT_US: u64 = 1000;

impl<'a, D> DHT11<'a, D>
where
    D: DelayNs,
{
    pub fn new(pin: impl Pin + 'a, delay: D) -> Self {
        let mut pin = Flex::new(pin);
        let out_config = OutputConfig::default().with_drive_mode(DriveMode::OpenDrain);
        pin.apply_output_config(&out_config);
        let input_config = InputConfig::default();
        pin.apply_input_config(&input_config);
        Self { pin, delay }
    }

    pub fn read(&mut self) -> Result<Reading, SensorError> {
        let data = self.read_raw()?;
        let rh = data[0];
        let temp_signed = data[2];
        let signed = temp_signed & 0x80 != 0;
        let magnitude = temp_signed & 0x7F;
        let temp = if signed {
            -(magnitude as i8)
        } else {
            magnitude as i8
        };

        Ok(Reading {
            temperature: temp,
            humidity: rh,
        })
    }

    fn read_raw(&mut self) -> Result<[u8; 5], SensorError> {
        self.pin.set_output_enable(true);
        self.delay.delay_ms(200);
        self.pin.set_low();
        self.delay.delay_ms(20);
        self.pin.set_high();
        self.delay.delay_us(40);
        self.pin.set_output_enable(false);
        self.pin.set_input_enable(true);

        let start = Instant::now();
        while self.pin.is_high() {
            if start.elapsed().as_millis() > TIMEOUT_DURATION {
                return Err(SensorError::Timeout);
            }
        }

        if self.pin.is_low() {
            self.delay.delay_us(80);
            if self.pin.is_low() {
                return Err(SensorError::Timeout);
            }
        }
        self.delay.delay_us(80);
        let mut buf = [0; 5];
        for idx in 0..5 {
            buf[idx] = self.read_byte()?;
        }
        let sum = buf[0]
            .wrapping_add(buf[1])
            .wrapping_add(buf[2])
            .wrapping_add(buf[3]);

        if buf[4] == (sum & 0xFF) {
            Ok(buf)
        } else {
            Err(SensorError::ChecksumMismatch)
        }
    }

    fn read_byte(&mut self) -> Result<u8, SensorError> {
        let mut buf = 0u8;
        for idx in 0..8u8 {
            let bit_start = Instant::now();
            while self.pin.is_low() {
                if bit_start.elapsed().as_micros() > BIT_TIMEOUT_US {
                    return Err(SensorError::Timeout);
                }
            }
            self.delay.delay_us(30);
            if self.pin.is_high() {
                buf |= 1 << (7 - idx);
            }
            let bit_start = Instant::now();
            while self.pin.is_high() {
                if bit_start.elapsed().as_micros() > BIT_TIMEOUT_US {
                    return Err(SensorError::Timeout);
                }
            }
        }
        Ok(buf)
    }
}

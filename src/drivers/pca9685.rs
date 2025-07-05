use embedded_hal::delay::DelayNs;
use embedded_hal::i2c::{I2c, Operation as I2cOperation};

use linux_embedded_hal::{Delay, I2cdev};

const SUBADR1: u8 = 0x02;
const SUBADR2: u8 = 0x03;
const SUBADR3: u8 = 0x04;
const MODE1: u8 = 0x00;
const PRESCALE: u8 = 0xFE;
const LED0_ON_L: u8 = 0x06;
const LED0_ON_H: u8 = 0x07;
const LED0_OFF_L: u8 = 0x08;
const LED0_OFF_H: u8 = 0x09;
const ALLLED_ON_L: u8 = 0xFA;
const ALLLED_ON_H: u8 = 0xFB;
const ALLLED_OFF_L: u8 = 0xFC;
const ALLLED_OFF_H: u8 = 0xFD;

const ADDR: u8 = 0x40;

pub struct Driver<I2C> {
    i2c: I2C,
}

impl<I2C> Driver<I2C>
where
    I2C: I2c,
{
    pub fn new(i2c: I2C) -> Self {
        Driver { i2c }
    }

    fn read_something(&mut self) -> Result<u8, I2C::Error> {
        let mut read_buffer = [0];
        let mut ops = [
            I2cOperation::Write(&[0xAB]),
            I2cOperation::Read(&mut read_buffer),
        ];
        self.i2c.transaction(ADDR, &mut ops).and(Ok(read_buffer[0]))
    }

    pub fn set_pwm_freq<D>(&mut self, freq: f32, delay: &mut D)
    where
        D: DelayNs,
    {
        let mut pre_scale_val = 25000000.0;
        pre_scale_val /= 4096.0;
        pre_scale_val /= freq;
        pre_scale_val -= 1.0;
        let prescale = (pre_scale_val + 0.5).floor() as u8;

        let mut read_buffer = [0u8];
        self.i2c.write_read(ADDR, &[MODE1], &mut read_buffer);
        let old_mode = read_buffer[0];
        let sleep_mode = (old_mode & 0x7F) | 0x10;
        self.i2c.write(ADDR, &[MODE1, sleep_mode]);
        self.i2c.write(ADDR, &[PRESCALE, prescale]);
        self.i2c.write(ADDR, &[MODE1, old_mode]);
        delay.delay_us(500);
        self.i2c.write(ADDR, &[MODE1, old_mode | 0x80]);
    }

    pub fn set_pwm(&mut self, channel: u8, on: u16, off: u16) {
        let base = LED0_ON_L + 4 * channel;
        let data = [
            base,
            (on & 0xFF) as u8,
            (on >> 8) as u8,
            (off & 0xFF) as u8,
            (off >> 8) as u8,
        ];
        self.i2c.write(ADDR, &data);
    }
}

fn main() {
    let dev = I2cdev::new("/dev/i2c-1").unwrap();
    let mut driver = Driver::new(dev);
    let value = driver.read_something().unwrap();
    println!("Read value: {}", value);
}

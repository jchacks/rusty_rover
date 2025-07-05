use embedded_hal::delay::DelayNs;
use embedded_hal::i2c::I2c;

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

    pub fn set_pwm_freq<D>(&mut self, freq: f32, delay: &mut D) -> Result<(), I2C::Error>
    where
        D: DelayNs,
    {
        let prescale = ((25_000_000.0 / (4096.0 * freq)) + 0.5).floor() as u8 - 1;
        let mut read_buffer = [0u8];
        self.i2c.write_read(ADDR, &[MODE1], &mut read_buffer)?;
        let old_mode = read_buffer[0];
        let sleep_mode = (old_mode & 0x7F) | 0x10;
        self.i2c.write(ADDR, &[MODE1, sleep_mode])?;
        self.i2c.write(ADDR, &[PRESCALE, prescale])?;
        self.i2c.write(ADDR, &[MODE1, old_mode])?;
        delay.delay_us(500);
        self.i2c.write(ADDR, &[MODE1, old_mode | 0x80])?;
        Ok(())
    }

    pub fn set_pwm(&mut self, channel: u8, on: u16, off: u16) -> Result<(), I2C::Error> {
        let base = LED0_ON_L + 4 * channel;
        let data = [
            base,
            (on & 0xFF) as u8,
            (on >> 8) as u8,
            (off & 0xFF) as u8,
            (off >> 8) as u8,
        ];
        self.i2c.write(ADDR, &data)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_hal_mock::eh1::{
        delay::NoopDelay,
        i2c::{Mock as I2cMock, Transaction as T},
    };

    /* ------------------------------------------------- */
    /* 1. 50 Hz frequency – typical servo setting        */
    /* ------------------------------------------------- */
    #[test]
    fn set_pwm_freq_50_hz() {
        // MODE1 reads 0x01 after power-on
        let expectations = [
            /* read current MODE1 */
            T::write_read(ADDR, vec![MODE1], vec![0x01]),
            /* go to sleep */
            T::write(ADDR, vec![MODE1, 0x11]),
            /* prescale ≈ 121 (0x79) */
            T::write(ADDR, vec![PRESCALE, 0x79]),
            /* wake */
            T::write(ADDR, vec![MODE1, 0x01]),
            /* restart */
            T::write(ADDR, vec![MODE1, 0x81]),
        ];

        let i2c = I2cMock::new(&expectations);
        let mut drv = Driver::new(i2c);
        let mut delay = NoopDelay;

        drv.set_pwm_freq(50.0, &mut delay).unwrap();
        drv.i2c.done();
    }

    /* ------------------------------------------------- */
    /* 2. 1 kHz frequency – check rounding branch        */
    /*    prescale should be 5                           */
    /* ------------------------------------------------- */
    #[test]
    fn set_pwm_freq_1khz() {
        // prescale = round(25 MHz / 4096 / 1 000 − 1) ≈ 6
        let expectations = [
            T::write_read(ADDR, vec![MODE1], vec![0x01]),
            T::write(ADDR, vec![MODE1, 0x11]),
            T::write(ADDR, vec![PRESCALE, 0x05]),
            T::write(ADDR, vec![MODE1, 0x01]),
            T::write(ADDR, vec![MODE1, 0x81]),
        ];

        let i2c = I2cMock::new(&expectations);
        let mut drv = Driver::new(i2c);
        let mut delay = NoopDelay;

        drv.set_pwm_freq(1_000.0, &mut delay).unwrap();
        drv.i2c.done();
    }

    /* ------------------------------------------------- */
    /* 3. set_pwm on channel 0                           */
    /* ------------------------------------------------- */
    #[test]
    fn set_pwm_channel0() {
        // on = 0x0000, off = 0x0100
        let expectations = [T::write(ADDR, vec![LED0_ON_L, 0x00, 0x00, 0x00, 0x01])];
        let i2c = I2cMock::new(&expectations);
        let mut drv = Driver::new(i2c);
        drv.set_pwm(0, 0x0000, 0x0100).unwrap();
        drv.i2c.done();
    }

    /* ------------------------------------------------- */
    /* 4. set_pwm on channel 15 (highest valid ch.)      */
    /*    LED0_ON_L + 4·15 = 0x06 + 0x3C = 0x42          */
    /* ------------------------------------------------- */
    #[test]
    fn set_pwm_channel15() {
        let base = LED0_ON_L + 4 * 15; // 0x42
        // on = 0x1234, off = 0xABCD
        let expectations = [T::write(ADDR, vec![base, 0x34, 0x12, 0xCD, 0xAB])];
        let i2c = I2cMock::new(&expectations);
        let mut drv = Driver::new(i2c);

        drv.set_pwm(15, 0x1234, 0xABCD).unwrap();
        drv.i2c.done();
    }
}

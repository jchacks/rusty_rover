use embedded_hal::delay::DelayNs;
use embedded_hal::i2c::I2c;

const SUBADR1: u8 = 0x02;
const SUBADR2: u8 = 0x03;
const SUBADR3: u8 = 0x04;
const MODE1: u8 = 0x00;
const MODE2: u8 = 0x01;
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

pub struct Driver<I2C>
where
    I2C: I2c,
{
    i2c: I2C,
}

impl<I2C> Driver<I2C>
where
    I2C: I2c,
{
    pub fn new(i2c: I2C) -> Self {
        Self { i2c }
    }

    /// Bring the PCA9685 into a known state after power-on.
    pub fn reset<D>(&mut self, delay: &mut D) -> Result<(), I2C::Error>
    where
        D: embedded_hal::delay::DelayNs,
    {
        self.i2c.write(ADDR, &[MODE1, 0x00])?;
        delay.delay_us(500);

        Ok(())
    }

    pub fn auto_increment_enabled(&mut self) -> Result<bool, I2C::Error> {
        let mut buf = [0u8];
        self.i2c.write_read(ADDR, &[MODE1], &mut buf)?;
        Ok(buf[0] & 0x20 != 0)
    }

    pub fn ensure_auto_increment(&mut self) -> Result<(), I2C::Error> {
        if !self.auto_increment_enabled()? {
            let mut mode1 = [0u8];
            self.i2c.write_read(ADDR, &[MODE1], &mut mode1)?;
            self.i2c.write(ADDR, &[MODE1, mode1[0] | 0x20])?;
        }
        Ok(())
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

    pub fn set_pwm(&mut self, ch: u8, on: u16, off: u16) -> Result<(), I2C::Error> {
        let base = LED0_ON_L + 4 * ch;
        let data = [
            base,
            (on & 0xFF) as u8,
            (on >> 8) as u8,
            (off & 0xFF) as u8,
            (off >> 8) as u8,
        ];
        println!("Sending: {:?}", data);
        self.i2c.write(ADDR, &data)?;
        Ok(())
    }

    pub fn enable_channel(&mut self, ch: u8) -> Result<(), I2C::Error> {
        let off_h = LED0_OFF_H + 4 * ch;
        self.i2c.write(ADDR, &[off_h, 0x00])?;
        Ok(())
    }

    pub fn disable_channel(&mut self, ch: u8) -> Result<(), I2C::Error> {
        let on_h = LED0_ON_H + 4 * ch; // 0x07 + 4·ch
        let off_h = LED0_OFF_H + 4 * ch; // 0x09 + 4·ch

        self.i2c.write(ADDR, &[on_h, 0x00])?; // bit 4 = 0
        self.i2c.write(ADDR, &[off_h, 0x10])?; // bit 4 = 1
        Ok(())
    }

    /// Force ALL outputs low (FULL_OFF) until re-enabled.
    pub fn disable_all(&mut self) -> Result<(), I2C::Error> {
        self.i2c.write(ADDR, &[ALLLED_ON_H, 0x00])?;
        self.i2c.write(ADDR, &[ALLLED_OFF_H, 0x10])?;
        Ok(())
    }
}

impl<I2C> Drop for Driver<I2C>
where
    I2C: I2c,
{
    fn drop(&mut self) {
        self.disable_all().expect("could not disable servos");
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

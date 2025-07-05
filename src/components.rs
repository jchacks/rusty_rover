use embedded_hal::i2c::I2c;

use crate::drivers::pca9685;

const MIN_ANGLE: f32 = 18.0;
const MAX_ANGLE: f32 = 162.0;
// === Servo angle → PCA9685 “off-count” mapping ===
//
// 50 Hz PWM period  = 20 ms
// 12-bit counter    = 4096 steps  →  1 step ≈ 20 ms / 4096 ≈ 4.88 µs
//
// Pick 0.5 ms … 2.5 ms pulse-width range for full 0°–180° travel:
//   counts_min = 0.5 ms  / 4.88 µs ≈ 102
//   counts_max = 2.5 ms  / 4.88 µs ≈ 512
const COUNTS_MIN: f32 = 102.0;
const COUNTS_MAX: f32 = 512.0;

pub struct Servo {
    channel: u8,
}

impl Servo {
    pub fn new(channel: u8) -> Self {
        Self { channel }
    }

    pub fn set_angle<I2C>(&mut self, pwm: &mut pca9685::Driver<I2C>, deg: f32)
    where
        I2C: I2c,
    {
        let data = (deg.clamp(MIN_ANGLE, MAX_ANGLE) * (COUNTS_MAX - COUNTS_MIN) / 180.0) as u16;
        pwm.set_pwm(self.channel, 0, data);
    }
}

use std::{cell::RefCell, rc::Rc};

use crate::{
    components::Servo,
    drivers::{ads7830, pca9685},
};
use embedded_hal_bus::i2c::RcDevice;
use linux_embedded_hal::{Delay, I2cdev};

mod components;
mod drivers;

pub struct RoboDog {
    ads7830: ads7830::Driver<RcDevice<I2cdev>>,
    pca9685: pca9685::Driver<RcDevice<I2cdev>>,
    servos: [Servo; 8],
}

impl RoboDog {
    pub fn new() -> Self {
        let bus = Rc::new(RefCell::new(I2cdev::new("/dev/i2c-1").unwrap()));

        let mut ads7830 = ads7830::Driver::new(RcDevice::new(Rc::clone(&bus)));
        let mut pca9685 = pca9685::Driver::new(RcDevice::new(Rc::clone(&bus)));
        let mut delay = Delay;
        pca9685.set_pwm_freq(50.0f32, &mut delay);

        let servos: [Servo; 8] = core::array::from_fn(move |i| Servo::new(i as u8));

        Self {
            ads7830,
            pca9685,
            servos,
        }
    }

    pub fn get_battery_voltage(&mut self) -> f32 {
        self.ads7830.battery_voltage().unwrap()
    }
}

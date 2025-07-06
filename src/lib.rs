use std::{cell::RefCell, rc::Rc};

use crate::{
    drivers::{Imu, Servo},
    hal::{ads7830, pca9685},
};
use embedded_hal::delay::DelayNs;
use embedded_hal_bus::i2c::RcDevice;
use linux_embedded_hal::{Delay, I2cdev};
use mpu6050;
mod drivers;
mod hal;

pub struct RoboDog {
    ads7830: ads7830::Driver<RcDevice<I2cdev>>,
    pca9685: pca9685::Driver<RcDevice<I2cdev>>,
    mpu6050: mpu6050::Mpu6050<RcDevice<I2cdev>>,
    servos: [Servo; 16],
    imu: Imu,
}

impl RoboDog {
    pub fn new() -> Self {
        let bus = Rc::new(RefCell::new(I2cdev::new("/dev/i2c-1").unwrap()));

        let ads7830 = ads7830::Driver::new(RcDevice::new(Rc::clone(&bus)));
        let mut pca9685 = pca9685::Driver::new(RcDevice::new(Rc::clone(&bus)));
        let mut delay = Delay;
        pca9685.reset(&mut delay).unwrap();
        pca9685
            .set_pwm_freq(50.0f32, &mut delay)
            .expect("setting pwm freq to 50Hz");
        pca9685
            .ensure_auto_increment()
            .expect("could not set autoincrement");

        let mut mpu6050 = mpu6050::Mpu6050::new(RcDevice::new(Rc::clone(&bus)));
        mpu6050.init(&mut delay).expect("init mpu6050");
        let imu = Imu::new(&mut mpu6050).expect("create IMU");
        let servos: [Servo; 16] = core::array::from_fn(move |i| Servo::new(i as u8));

        Self {
            ads7830,
            pca9685,
            mpu6050,
            servos,
            imu,
        }
    }

    pub fn get_battery_voltage(&mut self) -> f32 {
        self.ads7830.battery_voltage().unwrap()
    }

    pub fn set_servos_to_90(&mut self) {
        for servo in self.servos.iter_mut() {
            servo.set_angle(&mut self.pca9685, 90.0).unwrap();
        }
    }

    pub fn enable_servos(&mut self) {
        for servo in self.servos.iter_mut() {
            self.pca9685.enable_channel(servo.channel).unwrap();
        }
    }
    pub fn disable_servos(&mut self) {
        for servo in self.servos.iter_mut() {
            self.pca9685.disable_channel(servo.channel).unwrap();
        }
    }

    /// Returns (pitch, roll, yaw)  in degrees
    pub fn get_angles(&mut self) -> (f32, f32, f32) {
        self.imu.update(&mut self.mpu6050)
    }
}

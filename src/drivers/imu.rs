use std::time::Instant;

use embedded_hal::i2c::I2c;
use linux_embedded_hal::I2CError;
use mpu6050::{
    Mpu6050,
    device::{AccelRange, GyroRange},
};
use nalgebra::{Quaternion, Vector3};

const G: f32 = 9.81;

#[derive(Debug, Clone, Copy)]
pub struct Kalman {
    pub q: f32,
    pub r: f32,
    p: f32,
    x: f32,
}

impl Kalman {
    pub const fn new(q: f32, r: f32) -> Self {
        Self {
            q,
            r,
            p: 1.0,
            x: 0.0,
        }
    }

    pub fn update(&mut self, z: f32) -> f32 {
        self.p += self.q;
        let k = self.p / (self.p + self.r);
        self.x += k * (z - self.x);
        self.p *= 1.0 - k;
        self.x
    }
}

pub struct Imu {
    accel_err: Vector3<f32>,
    gyro_err: Vector3<f32>,
    kalman_acc: [Kalman; 3],
    kalman_gyro: [Kalman; 3],
    q: Quaternion<f32>,
    k_p: f32,
    k_i: f32,
    integral: Vector3<f32>,
    last_ts: Instant,
}

impl Imu {
    pub fn new<I2C>(mpu: &mut Mpu6050<I2C>) -> Result<Self, I2CError>
    where
        I2C: I2c,
    {
        mpu.set_accel_range(AccelRange::G2)
            .expect("set accel range");
        mpu.set_gyro_range(GyroRange::D250).expect("set gyro range");

        // one-shot static average, identical to Python ‘average_filter’
        let (acc_err, gyro_err) = Self::average_bias(mpu);
        println!("Initial accel_error={acc_err} gyro_error={gyro_err}");

        let last_ts = Instant::now();

        Ok(Self {
            accel_err: acc_err,
            gyro_err: gyro_err,
            kalman_acc: [Kalman::new(0.001, 0.1); 3],
            kalman_gyro: [Kalman::new(0.001, 0.1); 3],
            q: Quaternion::identity(),
            k_p: 100.0,
            k_i: 0.002,
            integral: Vector3::zeros(),
            last_ts,
        })
    }

    fn average_bias<I2C>(mpu: &mut Mpu6050<I2C>) -> (Vector3<f32>, Vector3<f32>)
    where
        I2C: I2c,
    {
        let mut acc = Vector3::zeros();
        let mut gyr = Vector3::zeros();

        for _ in 0..100 {
            let (a, g) = (
                mpu.get_acc().expect("get accel"),
                mpu.get_gyro().expect("get gyro"),
            );
            acc += Vector3::new(a.x, a.y, a.z);
            gyr += Vector3::new(g.x, g.y, g.z);
        }
        acc /= 100.0;
        gyr /= 100.0;
        acc.z -= 9.8; // subtract g

        (acc, gyr)
    }

    /// Returns (pitch, roll, yaw) in degrees
    pub fn update<I2C>(&mut self, mpu: &mut Mpu6050<I2C>) -> (f32, f32, f32)
    where
        I2C: I2c,
    {
        use nalgebra::{Unit, Vector4};
        let now = Instant::now();
        let dt = now.duration_since(self.last_ts).as_secs_f32();
        self.last_ts = now;

        let (raw_a, raw_g) = (
            mpu.get_acc().expect("get accel"),
            mpu.get_gyro().expect("get gyro"),
        );
        println!("raw {raw_a:?} {raw_g:?}");

        let a_vec = Vector3::new(raw_a.x, raw_a.y, raw_a.z) - self.accel_err;
        let g_vec = Vector3::new(raw_g.x, raw_g.y, raw_g.z) - self.gyro_err;

        let acc = Vector3::new(
            self.kalman_acc[0].update(a_vec.x),
            self.kalman_acc[1].update(a_vec.y),
            self.kalman_acc[2].update(a_vec.z),
        );
        let gyro = Vector3::new(
            self.kalman_gyro[0].update(g_vec.x),
            self.kalman_gyro[1].update(g_vec.y),
            self.kalman_gyro[2].update(g_vec.z),
        );
        println!("kalman {acc:?} {gyro:?}");

        let acc = acc / acc.norm();
        let v = Vector3::new(
            2.0 * (self.q.i * self.q.k - self.q.w * self.q.j),
            2.0 * (self.q.w * self.q.i + self.q.j * self.q.k),
            self.q.w * self.q.w - self.q.i * self.q.i - self.q.j * self.q.j + self.q.k * self.q.k,
        );

        let e = acc.cross(&v);
        self.integral += e * self.k_i;
        let gyro = gyro + e * self.k_p + self.integral;
        let q_dot = Quaternion::from(Vector4::new(0.0, gyro.x, gyro.y, gyro.z)) * self.q * 0.5;

        self.q += q_dot * dt;
        self.q = Unit::new_normalize(self.q).into_inner();

        let pitch = (-2.0 * (self.q.i * self.q.k - self.q.w * self.q.j))
            .asin()
            .to_degrees();
        let roll = (2.0 * (self.q.w * self.q.i + self.q.j * self.q.k))
            .atan2(1.0 - 2.0 * (self.q.i * self.q.i + self.q.j * self.q.j))
            .to_degrees();
        let yaw = (2.0 * (self.q.i * self.q.j + self.q.w * self.q.k))
            .atan2(
                self.q.w * self.q.w + self.q.i * self.q.i
                    - self.q.j * self.q.j
                    - self.q.k * self.q.k,
            )
            .to_degrees();

        (pitch, roll, yaw)
    }
}

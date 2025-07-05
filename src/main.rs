mod components;
mod drivers;
use i2c_linux::I2c;
use robotdog_rust::RoboDog;
use std::{error::Error, usize};

const ADS7830_DEFAULT_ADDRESS: u8 = 0x48;
const ADS7830_CMD: u8 = 0x84;
struct ADS7830 {}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // i2c-1 SMBus(1)

    let dog = RoboDog::new();
    let mut i2c = I2c::from_path("/dev/i2c-1").unwrap();
    println!("Hello, world!");
    Ok(())
}

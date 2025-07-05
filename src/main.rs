use robotdog_rust::RoboDog;

const ADS7830_DEFAULT_ADDRESS: u8 = 0x48;
const ADS7830_CMD: u8 = 0x84;
struct ADS7830 {}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // i2c-1 SMBus(1)

    let dog = RoboDog::new();
    println!("Hello, world!");
    Ok(())
}

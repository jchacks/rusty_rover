use robotdog_rust::RoboDog;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut dog = RoboDog::new();
    println!("Hello, world!");
    let voltage = dog.get_battery_voltage();
    println!("voltage {voltage}");
    Ok(())
}

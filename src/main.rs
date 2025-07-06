use std::time::Duration;

use robotdog_rust::RoboDog;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut dog = RoboDog::new();

    // let voltage = dog.get_battery_voltage();
    // println!("voltage {voltage}");

    // println!("Enabling servos");
    // dog.enable_servos();
    // let voltage = dog.get_battery_voltage();
    // println!("voltage {voltage}");
    // println!("Setting servos to 90");
    // dog.set_servos_to_90();
    let voltage = dog.get_battery_voltage();
    println!("voltage {voltage}");

    loop {
        let (pitch, roll, yaw) = dog.get_angles();
        println!("p={pitch} r={roll} y={yaw}");
        std::thread::sleep(Duration::from_millis(2));
    }

    Ok(())
}

use battery::Manager;

fn main() -> Result<(), battery::Error> {
    let manager = Manager::new()?;
    for battery in manager.batteries()? {
        let battery = battery?;
        println!("Voltage: {:.2} V", battery.voltage().value);
        println!("Energy: {:.2} Wh", battery.energy().value / 3600.0);  // This is off by quite a lot
        println!("Full Design Energy: {:.2} Wh", battery.energy_full_design().value / 3600.0);
        // Print the IoKitDevice self.source.current_capacity()
        // println!("Current Capacity: {:.2} Wh", battery.current_capacity().value);
    }
    Ok(())
}
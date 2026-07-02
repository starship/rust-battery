#[cfg(test)]
mod tests {
    use crate::platform::traits::BatteryDevice;
    use crate::platform::termux::manager::BatteryStatus;
    use crate::platform::termux::device::TermuxDevice;

    #[test]
    fn test_device_mapping() {
        let status = BatteryStatus {
            present: true,
            technology: "Li-ion".to_string(),
            health: "GOOD".to_string(),
            plugged: "PLUGGED_AC".to_string(),
            status: "CHARGING".to_string(),
            temperature: 30.0,
            voltage: 4000,
            current: 1000,
            percentage: 50,
            level: 50,
            scale: 100,
            charge_counter: 2500000,
        };
        let device = TermuxDevice::new(status);
        assert_eq!(device.technology(), crate::Technology::LithiumIon);
        assert_eq!(device.state(), crate::State::Charging);
    }
}

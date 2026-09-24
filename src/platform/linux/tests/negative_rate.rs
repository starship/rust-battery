use approx::assert_abs_diff_eq;

use super::super::SysFsDevice;
use crate::platform::traits::BatteryDevice;
use crate::State;

// https://github.com/6543/batmon/issues/13
//
// Drivers following the current sysfs ABI (e.g. Asahi Linux `macsmc-power`)
// report `power_now` and `current_now` as negative values while discharging.
// The energy rate must use the magnitude instead of dropping it to zero.
#[test]
fn test_negative_power_now() -> std::io::Result<()> {
    let root = sysfs_test_suite!(
        "capacity" => 80,
        "current_now" => -1_200_000,
        "energy_full" => 97_000_000,
        "energy_full_design" => 100_000_000,
        "energy_now" => 77_600_000,
        "power_now" => -15_000_000,
        "status" => "Discharging",
        "type" => "Battery",
        "voltage_now" => 12_500_000
    );

    let device = SysFsDevice::try_from(root.path().to_owned()).unwrap();

    assert_eq!(device.state(), State::Discharging);
    assert_abs_diff_eq!(device.energy_rate().value, 15.0, epsilon = 1e-4);

    root.close()
}

#[test]
fn test_negative_current_now() -> std::io::Result<()> {
    let root = sysfs_test_suite!(
        "capacity" => 80,
        "charge_full" => 7_000_000,
        "charge_full_design" => 7_500_000,
        "charge_now" => 5_600_000,
        "current_now" => -1_200_000,
        "status" => "Discharging",
        "type" => "Battery",
        "voltage_min_design" => 12_500_000,
        "voltage_now" => 12_500_000
    );

    let device = SysFsDevice::try_from(root.path().to_owned()).unwrap();

    assert_eq!(device.state(), State::Discharging);
    assert_abs_diff_eq!(device.energy_rate().value, 15.0, epsilon = 1e-4);

    root.close()
}

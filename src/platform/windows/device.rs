use std::convert::AsRef;
use std::fmt;

use windows_sys::Win32::System::Power::BATTERY_QUERY_INFORMATION;

use super::ffi::DeviceHandle;
use crate::platform::traits::BatteryDevice;
use crate::units::{ElectricPotential, Energy, Power, ThermodynamicTemperature};
use crate::{Error, Result, State, Technology};

pub struct PowerDevice {
    // Used later for information refreshing
    tag: BATTERY_QUERY_INFORMATION,

    technology: Technology,
    state: State,
    voltage: ElectricPotential,
    energy_rate: Power,
    capacity: Energy,
    design_capacity: Energy,
    full_charged_capacity: Energy,
    temperature: Option<ThermodynamicTemperature>,
    cycle_count: Option<u32>,
    device_name: Option<String>,
    manufacturer: Option<String>,
    serial_number: Option<String>,
}

impl Default for PowerDevice {
    fn default() -> Self {
        Self {
            technology: Default::default(),
            state: Default::default(),
            voltage: Default::default(),
            energy_rate: Default::default(),
            capacity: Default::default(),
            design_capacity: Default::default(),
            full_charged_capacity: Default::default(),
            temperature: None,
            cycle_count: None,
            device_name: None,
            manufacturer: None,
            serial_number: None,
            tag: unsafe { std::mem::zeroed() },
        }
    }
}

impl PowerDevice {
    pub fn try_from(mut handle: DeviceHandle) -> Result<Option<PowerDevice>> {
        let info = handle.information()?;
        if info.is_relative() {
            // We can't support batteries with relative data so far
            return Ok(None);
        }

        let device_name = handle.device_name().ok();
        let manufacturer = handle.manufacture_name().ok();
        let serial_number = handle.serial_number().ok();

        let mut device = PowerDevice {
            tag: handle.tag,
            technology: info.technology(),
            device_name,
            manufacturer,
            serial_number,
            ..Default::default()
        };
        device.refresh(handle)?;
        Ok(Some(device))
    }

    pub fn refresh(&mut self, mut handle: DeviceHandle) -> Result<()> {
        let info = handle.information()?;

        let status = handle.status()?;
        let rate = match status.rate() {
            // Battery neither charging nor discharging, energy rate is set to zero
            None => watt!(0.0),
            Some(value) => milliwatt!(value),
        };
        let capacity = match status.capacity() {
            None => return Err(Error::invalid_data("Device capacity value is unknown")),
            Some(value) => milliwatt_hour!(value),
        };
        let voltage = match status.voltage() {
            None => return Err(Error::invalid_data("Device voltage value is unknown")),
            Some(value) => millivolt!(value),
        };
        let temperature = match handle.temperature() {
            Ok(value) => Some(decikelvin!(value)),
            Err(_) => None,
        };

        self.state = status.state();
        self.energy_rate = rate;
        self.design_capacity = milliwatt_hour!(info.designed_capacity());
        self.full_charged_capacity = milliwatt_hour!(info.full_charged_capacity());
        self.cycle_count = info.cycle_count();
        self.capacity = capacity;
        self.voltage = voltage;
        self.temperature = temperature;

        Ok(())
    }

    pub fn tag(&self) -> &BATTERY_QUERY_INFORMATION {
        &self.tag
    }
}

impl BatteryDevice for PowerDevice {
    fn energy(&self) -> Energy {
        self.capacity
    }

    fn energy_full(&self) -> Energy {
        self.full_charged_capacity
    }

    fn energy_full_design(&self) -> Energy {
        self.design_capacity
    }

    fn energy_rate(&self) -> Power {
        self.energy_rate
    }

    fn state(&self) -> State {
        self.state
    }

    fn voltage(&self) -> ElectricPotential {
        self.voltage
    }

    fn temperature(&self) -> Option<ThermodynamicTemperature> {
        self.temperature
    }

    fn vendor(&self) -> Option<&str> {
        self.manufacturer.as_ref().map(AsRef::as_ref)
    }

    fn model(&self) -> Option<&str> {
        self.device_name.as_ref().map(AsRef::as_ref)
    }

    fn serial_number(&self) -> Option<&str> {
        self.serial_number.as_ref().map(AsRef::as_ref)
    }

    fn technology(&self) -> Technology {
        self.technology
    }

    fn cycle_count(&self) -> Option<u32> {
        self.cycle_count
    }
}

impl fmt::Debug for PowerDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PowerDevice")
            .field("technology", &self.technology)
            .field("state", &self.state)
            .field("voltage", &self.voltage)
            .field("energy_rate", &self.energy_rate)
            .field("capacity", &self.capacity)
            .field("design_capacity", &self.design_capacity)
            .field("full_charged_capacity", &self.full_charged_capacity)
            .field("temperature", &self.temperature)
            .field("cycle_count", &self.cycle_count)
            .field("device_name", &self.device_name)
            .field("manufacturer", &self.manufacturer)
            .field("serial_number", &self.serial_number)
            .finish()
    }
}

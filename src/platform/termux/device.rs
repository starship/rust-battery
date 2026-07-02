use crate::platform::traits::BatteryDevice;
use crate::platform::termux::manager::BatteryStatus;
use crate::units::{
    ElectricPotential, Energy, Power, ThermodynamicTemperature,
};
use crate::{State, Technology};

#[derive(Debug)]
pub struct TermuxDevice {
    status: BatteryStatus,
}

impl TermuxDevice {
    pub fn new(status: BatteryStatus) -> Self {
        Self { status }
    }

    pub fn refresh(&mut self) -> crate::Result<()> {
        // In a real implementation, we would re-run termux-battery-status here.
        // For now, let's keep it simple.
        Ok(())
    }
}

impl BatteryDevice for TermuxDevice {
    fn energy(&self) -> Energy {
        // Estimate energy in mWh
        milliwatt_hour!((self.status.percentage as f32 / 100.0) * (self.status.charge_counter as f32 / 1000.0) * (self.status.voltage as f32 / 1000.0))
    }

    fn energy_full(&self) -> Energy {
        milliwatt_hour!((self.status.charge_counter as f32 / 1000.0) * (self.status.voltage as f32 / 1000.0))
    }

    fn energy_full_design(&self) -> Energy {
        milliwatt_hour!((self.status.charge_counter as f32 / 1000.0) * (self.status.voltage as f32 / 1000.0))
    }

    fn energy_rate(&self) -> Power {
        // current (µA) * voltage (mV) -> µW
        // 1000 µW = 1 mW
        milliwatt!((self.status.current as f32 / 1000.0) * (self.status.voltage as f32 / 1000.0))
    }

    fn state(&self) -> State {
        match self.status.status.as_str() {
            "CHARGING" => State::Charging,
            "DISCHARGING" => State::Discharging,
            "FULL" => State::Full,
            "NOT_CHARGING" => State::Unknown,
            _ => State::Unknown,
        }
    }

    fn voltage(&self) -> ElectricPotential {
        millivolt!(self.status.voltage as f32)
    }

    fn temperature(&self) -> Option<ThermodynamicTemperature> {
        Some(celsius!(self.status.temperature))
    }

    fn vendor(&self) -> Option<&str> {
        None
    }

    fn model(&self) -> Option<&str> {
        None
    }

    fn serial_number(&self) -> Option<&str> {
        None
    }

    fn technology(&self) -> Technology {
        match self.status.technology.as_str() {
            "Li-ion" => Technology::LithiumIon,
            "Li-poly" => Technology::LithiumPolymer,
            _ => Technology::Unknown,
        }
    }

    fn cycle_count(&self) -> Option<u32> {
        None
    }
}

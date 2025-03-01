// For keys reference see: https://developer.apple.com/documentation/kernel/iopmpowersource?language=objc
// Additional keys worth to implement later:
//  * "ChargerData" ->
//    - ChargingVoltage
//    - ChargingCurrent
//    - NotChargingReason (?)

use num_traits::identities::Zero;
use std::boxed::Box;
use std::fmt;
use std::str;

use super::traits::DataSource;
use crate::platform::traits::BatteryDevice;
use crate::types::{State, Technology};
use crate::units::{
    Bound, ElectricPotential, Energy, Power, Ratio, ThermodynamicTemperature, Time,
};
use crate::Result;

pub struct IoKitDevice {
    source: Box<dyn DataSource>,
}

impl IoKitDevice {
    pub fn get_mut_ref(&mut self) -> &mut dyn DataSource {
        &mut self.source
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.source.refresh()
    }
}

impl BatteryDevice for IoKitDevice {
    fn energy(&self) -> Energy {
        self.source.current_capacity_raw() * self.source.voltage()
    }

    fn energy_full(&self) -> Energy {
        self.source.max_capacity_raw() * self.source.voltage()
    }

    fn energy_full_design(&self) -> Energy {
        self.source.design_capacity() * self.source.voltage()
    }

    fn energy_rate(&self) -> Power {
        self.source.amperage() * self.source.voltage()
    }

    fn state_of_charge(&self) -> Ratio {
        // It it possible to get values greater that `1.0`, which is logical nonsense,
        // forcing the value to be in `0.0..=1.0` range
        (self.source.current_capacity() / self.source.max_capacity()).into_bounded()
    }

    fn state(&self) -> State {
        match () {
            _ if !self.source.external_connected() => State::Discharging,
            _ if self.source.is_charging() => State::Charging,
            _ if self.source.current_capacity().is_zero() => State::Empty,
            _ if self.source.fully_charged() => State::Full,
            _ => State::Unknown,
        }
    }

    fn voltage(&self) -> ElectricPotential {
        self.source.voltage()
    }

    fn temperature(&self) -> Option<ThermodynamicTemperature> {
        self.source.temperature()
    }

    fn vendor(&self) -> Option<&str> {
        self.source.manufacturer()
    }

    fn model(&self) -> Option<&str> {
        self.source.device_name()
    }

    fn serial_number(&self) -> Option<&str> {
        self.source.serial_number()
    }

    fn technology(&self) -> Technology {
        Technology::Unknown
    }

    fn cycle_count(&self) -> Option<u32> {
        self.source.cycle_count()
    }

    fn time_to_full(&self) -> Option<Time> {
        if self.state() == State::Charging {
            self.source.time_remaining()
        } else {
            None
        }
    }

    fn time_to_empty(&self) -> Option<Time> {
        if self.state() == State::Discharging {
            self.source.time_remaining()
        } else {
            None
        }
    }
}

impl<T> From<T> for IoKitDevice
where
    T: DataSource,
{
    fn from(ds: T) -> IoKitDevice {
        IoKitDevice {
            source: Box::new(ds),
        }
    }
}

impl fmt::Debug for IoKitDevice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MacOSDevice")
            .field("source", &self.source)
            .finish()
    }
}

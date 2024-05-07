//! https://docs.microsoft.com/en-us/windows/desktop/power/battery-status-str

#![allow(non_snake_case, clippy::unreadable_literal)]

use std::ops;

use windows_sys::Win32::System::Power::*;

use crate::State;

pub struct BatteryStatus(BATTERY_STATUS);

impl Default for BatteryStatus {
    fn default() -> Self {
        Self(unsafe { std::mem::zeroed() })
    }
}

impl std::fmt::Debug for BatteryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("PowerState")
            .field("PowerState", &self.0.PowerState)
            .field("Capacity", &self.capacity())
            .field("Voltage", &self.voltage())
            .field("Rate", &self.rate())
            .finish()
    }
}

impl ops::Deref for BatteryStatus {
    type Target = BATTERY_STATUS;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl ops::DerefMut for BatteryStatus {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<BATTERY_STATUS> for BatteryStatus {
    fn from(status: BATTERY_STATUS) -> Self {
        Self(status)
    }
}

impl BatteryStatus {
    #[inline]
    pub fn is_charging(&self) -> bool {
        (self.0.PowerState & BATTERY_CHARGING) != 0
    }

    #[inline]
    pub fn is_critical(&self) -> bool {
        (self.0.PowerState & BATTERY_CRITICAL) != 0
    }

    #[inline]
    pub fn is_discharging(&self) -> bool {
        (self.0.PowerState & BATTERY_DISCHARGING) != 0
    }

    #[inline]
    pub fn is_power_on_line(&self) -> bool {
        (self.0.PowerState & BATTERY_POWER_ON_LINE) != 0
    }

    pub fn state(&self) -> State {
        match () {
            _ if self.is_charging() => State::Charging,
            _ if self.is_critical() => State::Empty,
            _ if self.is_discharging() => State::Discharging,
            _ if self.is_power_on_line() && !self.is_charging() => State::Full,
            _ => State::Unknown,
        }
    }

    pub fn voltage(&self) -> Option<u32> {
        if self.0.Voltage == BATTERY_UNKNOWN_VOLTAGE {
            None
        } else {
            Some(self.0.Voltage)
        }
    }

    pub fn capacity(&self) -> Option<u32> {
        if self.0.Capacity == BATTERY_UNKNOWN_CAPACITY {
            None
        } else {
            Some(self.0.Capacity)
        }
    }

    /// The current rate of battery charge or discharge.
    pub fn rate(&self) -> Option<i32> {
        if self.0.Rate == BATTERY_UNKNOWN_RATE as i32 {
            None
        } else {
            Some(self.0.Rate.abs())
        }
    }
}

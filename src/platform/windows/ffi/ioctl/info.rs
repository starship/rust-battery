//! https://docs.microsoft.com/en-us/windows/desktop/power/battery-information-str

#![allow(non_snake_case, clippy::unreadable_literal)]

use crate::Technology;
use std::str::FromStr;
use windows_sys::Win32::System::Power::*;

pub struct BatteryInformation(BATTERY_INFORMATION);

impl Default for BatteryInformation {
    fn default() -> Self {
        Self(unsafe { std::mem::zeroed() })
    }
}

impl std::fmt::Debug for BatteryInformation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("BatteryInformation")
            .field("Capabilities", &self.0.Capabilities)
            .field("Technology", &self.technology())
            .field("DesignedCapacity", &self.0.DesignedCapacity)
            .field("FullChargedCapacity", &self.0.FullChargedCapacity)
            .field("CycleCount", &self.0.CycleCount)
            .finish()
    }
}

impl From<BATTERY_INFORMATION> for BatteryInformation {
    fn from(info: BATTERY_INFORMATION) -> Self {
        Self(info)
    }
}

impl std::ops::Deref for BatteryInformation {
    type Target = BATTERY_INFORMATION;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for BatteryInformation {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl BatteryInformation {
    #[inline]
    pub fn is_system_battery(&self) -> bool {
        (self.0.Capabilities & BATTERY_SYSTEM_BATTERY) != 0
    }

    #[inline]
    pub fn is_relative(&self) -> bool {
        (self.0.Capabilities & BATTERY_CAPACITY_RELATIVE) != 0
    }

    pub fn technology(&self) -> Technology {
        let raw = String::from_utf8_lossy(&self.0.Chemistry);
        match Technology::from_str(&raw) {
            Ok(tech) => tech,
            Err(_) => Technology::Unknown,
        }
    }

    // Originally `mWh`, matches `Battery::energy_full_design` result
    #[inline]
    pub fn designed_capacity(&self) -> u32 {
        self.0.DesignedCapacity
    }

    // Originally `mWh`, matches `Battery::energy_full` result
    #[inline]
    pub fn full_charged_capacity(&self) -> u32 {
        self.0.FullChargedCapacity
    }

    pub fn cycle_count(&self) -> Option<u32> {
        if self.0.CycleCount == 0 {
            None
        } else {
            Some(self.0.CycleCount)
        }
    }
}

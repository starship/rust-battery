use std::ffi::CStr;
use std::fmt;
use std::fmt::Debug;

use objc2_core_foundation::{CFBoolean, CFDictionary, CFNumber, CFRetained, CFString, CFType};
use objc2_io_kit::{
    kIOPMDeviceNameKey, kIOPMFullyChargedKey, kIOPMPSAmperageKey, kIOPMPSBatteryTemperatureKey,
    kIOPMPSCurrentCapacityKey, kIOPMPSCycleCountKey, kIOPMPSDesignCapacityKey,
    kIOPMPSExternalConnectedKey, kIOPMPSIsChargingKey, kIOPMPSManufacturerKey,
    kIOPMPSMaxCapacityKey, kIOPMPSSerialKey, kIOPMPSTimeRemainingKey, kIOPMPSVoltageKey,
};

use super::super::traits::DataSource;
use super::IoObject;
use crate::units::{
    ElectricCharge, ElectricCurrent, ElectricPotential, ThermodynamicTemperature, Time,
};
use crate::{Error, Result};

type Properties = CFDictionary<CFString, CFType>;

// MaxCapacity and CurrentCapacity returns percentage in M series chips, ranging from 1 to 100.
// Have to use AppleRawMaxCapacity and AppleRawCurrentCapacity to get actual mAh.
// No idea if Intel-chip mac need to be changed as well.
static MAX_CAPACITY_KEY_RAW: &CStr = c"AppleRawMaxCapacity";
static CURRENT_CAPACITY_KEY_RAW: &CStr = c"AppleRawCurrentCapacity";

#[derive(Debug)]
pub struct InstantData {
    fully_charged: Option<bool>,
    external_connected: bool,
    is_charging: bool,
    voltage: ElectricPotential,
    amperage: ElectricCurrent,
    design_capacity: Option<ElectricCharge>,
    max_capacity: Option<ElectricCharge>,
    current_capacity: Option<ElectricCharge>,
    max_capacity_raw: Option<ElectricCharge>,
    current_capacity_raw: Option<ElectricCharge>,
    temperature: Option<ThermodynamicTemperature>,
    cycle_count: Option<u32>,
    time_remaining: Option<Time>,
}

impl InstantData {
    pub fn try_from(props: &Properties) -> Result<InstantData> {
        Ok(Self {
            fully_charged: Self::get_bool(props, kIOPMFullyChargedKey).ok(),
            external_connected: Self::get_bool(props, kIOPMPSExternalConnectedKey)?,
            is_charging: Self::get_bool(props, kIOPMPSIsChargingKey)?,
            voltage: millivolt!(Self::get_u32(props, kIOPMPSVoltageKey)?),
            amperage: milliampere!(Self::get_i32(props, kIOPMPSAmperageKey)?.abs()),
            design_capacity: Self::get_u32(props, kIOPMPSDesignCapacityKey)
                .ok()
                .map(|capacity| milliampere_hour!(capacity)),
            max_capacity: Self::get_u32(props, kIOPMPSMaxCapacityKey)
                .ok()
                .map(|capacity| milliampere_hour!(capacity)),
            current_capacity: Self::get_u32(props, kIOPMPSCurrentCapacityKey)
                .ok()
                .map(|capacity| milliampere_hour!(capacity)),
            max_capacity_raw: Self::get_u32(props, MAX_CAPACITY_KEY_RAW)
                .ok()
                .map(|capacity| milliampere_hour!(capacity)),
            current_capacity_raw: Self::get_u32(props, CURRENT_CAPACITY_KEY_RAW)
                .or_else(|_| Self::get_u32(props, kIOPMPSCurrentCapacityKey))
                .ok()
                .map(|capacity| milliampere_hour!(capacity)),
            temperature: Self::get_i32(props, kIOPMPSBatteryTemperatureKey)
                .map(|value| celsius!(value as f32 / 100.0))
                .ok(),
            cycle_count: Self::get_u32(props, kIOPMPSCycleCountKey).ok(),
            time_remaining: Self::get_i32(props, kIOPMPSTimeRemainingKey)
                .ok()
                .and_then(|val| {
                    if val == i32::MAX {
                        None
                    } else {
                        Some(minute!(val))
                    }
                }),
        })
    }

    fn get_bool(props: &Properties, raw_key: &CStr) -> Result<bool> {
        let key_str = raw_key
            .to_str()
            .map_err(|e| Error::invalid_data(e.to_string()))?;

        let key = CFString::from_str(key_str);
        let value = props
            .get(&key)
            .ok_or(Error::not_found(key_str.to_string()))?;

        CFRetained::downcast::<CFBoolean>(value)
            .map(|b| b.as_bool())
            .map_err(|e| Error::invalid_data(format!("{:?} is not a valid bool value", e)))
    }

    fn get_u32(props: &Properties, raw_key: &CStr) -> Result<u32> {
        // TODO: We can lose data here actually,
        // but with currently used keys it seems to be impossible
        Self::get_i32(props, raw_key).map(|n| n as u32)
    }

    fn get_i32(props: &Properties, raw_key: &CStr) -> Result<i32> {
        let key_str = raw_key
            .to_str()
            .map_err(|e| Error::invalid_data(e.to_string()))?;

        let key = CFString::from_str(key_str);
        let value = props
            .get(&key)
            .ok_or(Error::not_found(key_str.to_string()))?;

        CFRetained::downcast::<CFNumber>(value)
            .map_err(|e| Error::invalid_data(format!("{:?} is not a valid number value", e)))?
            .as_i32()
            .ok_or(Error::invalid_data("Cannot convert number to i32"))
    }

    fn get_string(props: &Properties, raw_key: &CStr) -> Result<String> {
        let key_str = raw_key
            .to_str()
            .map_err(|e| Error::invalid_data(e.to_string()))?;

        let key = CFString::from_str(key_str);
        let value = props
            .get(&key)
            .ok_or(Error::not_found(key_str.to_string()))?;

        CFRetained::downcast::<CFString>(value)
            .map(|s| s.to_string())
            .map_err(|e| Error::invalid_data(format!("{:?} is not a valid string value", e)))
    }
}

pub struct PowerSource {
    object: IoObject,
    data: InstantData,

    manufacturer: Option<String>,
    device_name: Option<String>,
    serial_number: Option<String>,
}

impl PowerSource {
    pub fn try_from(io_obj: IoObject) -> Result<PowerSource> {
        let props = io_obj.properties()?;
        let data = InstantData::try_from(&props)?;

        let manufacturer = InstantData::get_string(&props, kIOPMPSManufacturerKey).ok();
        let device_name = InstantData::get_string(&props, kIOPMDeviceNameKey).ok();
        let serial_number = InstantData::get_string(&props, kIOPMPSSerialKey).ok();

        Ok(PowerSource {
            object: io_obj,
            data,
            manufacturer,
            device_name,
            serial_number,
        })
    }
}

impl DataSource for PowerSource {
    fn refresh(&mut self) -> Result<()> {
        let props = self.object.properties()?;
        self.data = InstantData::try_from(&props)?;

        Ok(())
    }

    fn fully_charged(&self) -> bool {
        self.data.fully_charged.unwrap_or(true)
    }

    fn external_connected(&self) -> bool {
        self.data.external_connected
    }

    fn is_charging(&self) -> bool {
        self.data.is_charging
    }

    fn voltage(&self) -> ElectricPotential {
        self.data.voltage
    }

    fn amperage(&self) -> ElectricCurrent {
        self.data.amperage
    }

    fn design_capacity(&self) -> ElectricCharge {
        self.data.design_capacity.unwrap_or_default()
    }

    fn max_capacity_raw(&self) -> ElectricCharge {
        self.data
            .max_capacity_raw
            .or(self.data.max_capacity)
            .unwrap_or_default()
    }

    fn current_capacity_raw(&self) -> ElectricCharge {
        self.data
            .current_capacity_raw
            .or(self.data.current_capacity)
            .unwrap_or_default()
    }

    fn max_capacity(&self) -> ElectricCharge {
        self.data.max_capacity.unwrap_or_default()
    }

    fn current_capacity(&self) -> ElectricCharge {
        self.data.current_capacity.unwrap_or_default()
    }

    fn temperature(&self) -> Option<ThermodynamicTemperature> {
        self.data.temperature
    }

    fn cycle_count(&self) -> Option<u32> {
        self.data.cycle_count
    }

    fn time_remaining(&self) -> Option<Time> {
        self.data.time_remaining
    }

    fn manufacturer(&self) -> Option<&str> {
        self.manufacturer.as_ref().map(AsRef::as_ref)
    }

    fn device_name(&self) -> Option<&str> {
        self.device_name.as_ref().map(AsRef::as_ref)
    }

    fn serial_number(&self) -> Option<&str> {
        self.serial_number.as_ref().map(AsRef::as_ref)
    }
}

impl fmt::Debug for PowerSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PowerSource")
            .field("io_object", &self.object)
            .finish()
    }
}

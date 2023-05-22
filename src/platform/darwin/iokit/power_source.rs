use std::fmt;
use std::i32;

use core_foundation::base::{CFType, TCFType};
use core_foundation::boolean::{CFBoolean, CFBooleanGetTypeID};
use core_foundation::dictionary::CFDictionary;
use core_foundation::number::{CFNumber, CFNumberGetTypeID};
use core_foundation::string::{CFString, CFStringGetTypeID};

use super::super::traits::DataSource;
use super::IoObject;
use crate::units::{ElectricCharge, ElectricCurrent, ElectricPotential, ThermodynamicTemperature, Time};
use crate::{Error, Result};

type Properties = CFDictionary<CFString, CFType>;

static FULLY_CHARGED_KEY: &str = "FullyCharged";
static EXTERNAL_CONNECTED_KEY: &str = "ExternalConnected";
static IS_CHARGING_KEY: &str = "IsCharging";
static VOLTAGE_KEY: &str = "Voltage";
static AMPERAGE_KEY: &str = "Amperage";
static DESIGN_CAPACITY_KEY: &str = "DesignCapacity";

// aarch64 means M series chips.
// MaxCapacity and CurrentCapacity returns percentage in M series chips, ranging from 1 to 100.
// Have to use AppleRawMaxCapacity and AppleRawCurrentCapacity to get actual mAh.
// No idea if Intel-chip mac need to be changed as well.
static MAX_CAPACITY_KEY: &str = if cfg!(target_arch = "aarch64") {"AppleRawMaxCapacity"} else {"MaxCapacity"};
static CURRENT_CAPACITY_KEY: &str = if cfg!(target_arch = "aarch64") {"AppleRawCurrentCapacity"} else {"CurrentCapacity"};

static TEMPERATURE_KEY: &str = "Temperature";
static CYCLE_COUNT_KEY: &str = "CycleCount";
static TIME_REMAINING_KEY: &str = "TimeRemaining";
static MANUFACTURER_KEY: &str = "Manufacturer";
static DEVICE_NAME_KEY: &str = "DeviceName";
static BATTERY_SERIAL_NUMBER_KEY: &str = "BatterySerialNumber";

#[derive(Debug)]
pub struct InstantData {
    fully_charged: Option<bool>,
    external_connected: bool,
    is_charging: bool,
    voltage: ElectricPotential,
    amperage: ElectricCurrent,
    design_capacity: Option<ElectricCharge>,
    max_capacity: ElectricCharge,
    current_capacity: ElectricCharge,
    temperature: Option<ThermodynamicTemperature>,
    cycle_count: Option<u32>,
    time_remaining: Option<Time>,
}

impl InstantData {
    pub fn try_from(props: &Properties) -> Result<InstantData> {
        Ok(Self {
            fully_charged: Self::get_bool(props, FULLY_CHARGED_KEY).ok(),
            external_connected: Self::get_bool(props, EXTERNAL_CONNECTED_KEY)?,
            is_charging: Self::get_bool(props, IS_CHARGING_KEY)?,
            voltage: millivolt!(Self::get_u32(props, VOLTAGE_KEY)?),
            amperage: milliampere!(Self::get_i32(props, AMPERAGE_KEY)?.abs()),
            design_capacity: Self::get_u32(props, DESIGN_CAPACITY_KEY)
                .ok()
                .map(|capacity| milliampere_hour!(capacity)),
            max_capacity: milliampere_hour!(Self::get_u32(props, MAX_CAPACITY_KEY)?),
            current_capacity: milliampere_hour!(Self::get_u32(props, CURRENT_CAPACITY_KEY)?),
            temperature: Self::get_i32(props, TEMPERATURE_KEY)
                .map(|value| celsius!(value as f32 / 100.0))
                .ok(),
            cycle_count: Self::get_u32(props, CYCLE_COUNT_KEY).ok(),
            time_remaining: Self::get_i32(props, TIME_REMAINING_KEY).ok().and_then(|val| {
                if val == i32::MAX {
                    None
                } else {
                    Some(minute!(val))
                }
            }),
        })
    }

    fn get_bool(props: &Properties, raw_key: &'static str) -> Result<bool> {
        let key = CFString::from_static_string(raw_key);

        props
            .find(&key)
            .and_then(|value_ref| {
                unsafe {
                    debug_assert!(value_ref.type_of() == CFBooleanGetTypeID());
                }

                value_ref.downcast::<CFBoolean>()
            })
            .map(Into::into)
            .ok_or_else(|| Error::not_found(raw_key))
    }

    fn get_u32(props: &Properties, raw_key: &'static str) -> Result<u32> {
        let key = CFString::from_static_string(raw_key);

        props
            .find(&key)
            .and_then(|value_ref| {
                unsafe {
                    debug_assert!(value_ref.type_of() == CFNumberGetTypeID());
                }

                value_ref.downcast::<CFNumber>()
            })
            // TODO: We can lose data here actually,
            // but with currently used keys it seems to be impossible
            .and_then(|number| number.to_i32())
            .map(|value| value as u32)
            .ok_or_else(|| Error::not_found(raw_key))
    }

    fn get_i32(props: &Properties, raw_key: &'static str) -> Result<i32> {
        let key = CFString::from_static_string(raw_key);

        props
            .find(&key)
            .and_then(|value_ref| {
                unsafe {
                    debug_assert!(value_ref.type_of() == CFNumberGetTypeID());
                }

                value_ref.downcast::<CFNumber>()
            })
            .and_then(|number| number.to_i32())
            .ok_or_else(|| Error::not_found(raw_key))
    }

    fn get_string(props: &Properties, raw_key: &'static str) -> Result<String> {
        let key = CFString::from_static_string(raw_key);

        props
            .find(&key)
            .and_then(|value_ref| {
                unsafe {
                    debug_assert!(value_ref.type_of() == CFStringGetTypeID());
                }

                value_ref.downcast::<CFString>()
            })
            .map(|cf_string| cf_string.to_string())
            .ok_or_else(|| Error::not_found(raw_key))
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

        let manufacturer = InstantData::get_string(&props, MANUFACTURER_KEY).ok();
        let device_name = InstantData::get_string(&props, DEVICE_NAME_KEY).ok();
        let serial_number = InstantData::get_string(&props, BATTERY_SERIAL_NUMBER_KEY).ok();

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

    fn max_capacity(&self) -> ElectricCharge {
        self.data.max_capacity
    }

    fn current_capacity(&self) -> ElectricCharge {
        self.data.current_capacity
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
        f.debug_struct("PowerSource").field("io_object", &self.object).finish()
    }
}

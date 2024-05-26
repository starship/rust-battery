use crate::platform::traits::BatteryDevice;
use crate::plist;
use crate::units::{ElectricPotential, Energy, Power, ThermodynamicTemperature};
use crate::{Error, Result};
use crate::{State, Technology};

use super::utils::{AsResult, GetResult};

// It is assumed that devices use the same unit for the same type of measures.
// In other words it is assumed that if a device exposes energy_full is in ampere hours, it  won't
// report energy_full_design in watt hours.

// Only acpibat battery driver is supported.
// https://www.unitedbsd.com/d/1311-battery-data-such-as-vendor-model/25

// https://github.com/NetBSD/src/blob/trunk/sys/dev/acpi/acpi_bat.c#L101
const ACPIBAT_PRESENT: usize = 0;
const ACPIBAT_DVOLTAGE: usize = 1;
const ACPIBAT_VOLTAGE: usize = 2;
const ACPIBAT_DCAPACITY: usize = 3;
const ACPIBAT_LFCCAPACITY: usize = 4;
const ACPIBAT_CAPACITY: usize = 5;
const ACPIBAT_CHARGERATE: usize = 6;
const ACPIBAT_DISCHARGERATE: usize = 7;
const ACPIBAT_CHARGING: usize = 8;

#[derive(Debug, Default)]
pub struct EnvSysDevice<'a> {
    name: String,
    energy: u64,
    energy_full: u64,
    energy_full_design: u64,
    charge_rate: i64,
    discharge_rate: i64,
    voltage: u64,
    design_voltage: u64,
    charging: i64,
    punit: &'a str,
    eunit: &'a str,
}

impl<'a> EnvSysDevice<'a> {
    pub fn new(name: String, sensor: &'a plist::Value) -> Result<Self> {
        // We only support acpibat driver.
        // Early check avoid other evaluations.
        if !name.starts_with("acpibat") {
            return Err(Error::invalid_data("Not a valid battery"));
        }

        let sensor_slice = sensor.as_rslice()?;

        let mut data = Self {
            name,
            ..Self::default()
        };

        // Man 4 envsys from NetBSD 10 tells the last one is a "special dictionary"
        // for device properties.
        if sensor_slice
            .last()
            .ok_or(Error::invalid_data("Cannot read sensor property"))?
            .get_rdict("device-properties")?
            .get_rstring("device-class")?
            != "battery"
        {
            return Err(Error::invalid_data("Not a valid battery"));
        }

        if Self::val_cur_value(&sensor_slice[ACPIBAT_PRESENT])? != 1 {
            return Err(Error::not_found("Battery absent"));
        }

        if !Self::validate(&sensor_slice[ACPIBAT_CAPACITY])?.get_rbool("want-percentage")? {
            return Err(Error::invalid_data("Not a valid battery"));
        }

        data.design_voltage = Self::val_cur_value(&sensor_slice[ACPIBAT_DVOLTAGE])?;
        data.voltage = Self::val_cur_value(&sensor_slice[ACPIBAT_VOLTAGE])?;
        data.energy_full_design = Self::val_cur_value(&sensor_slice[ACPIBAT_DCAPACITY])?;
        data.energy_full = Self::val_cur_value(&sensor_slice[ACPIBAT_LFCCAPACITY])?;
        data.energy = Self::val_cur_value(&sensor_slice[ACPIBAT_CAPACITY])?;
        data.eunit = sensor_slice[ACPIBAT_DCAPACITY].get_rstring("type")?;
        data.charge_rate = sensor_slice[ACPIBAT_CHARGERATE].get_ri64("cur-value")?;
        data.discharge_rate = sensor_slice[ACPIBAT_DISCHARGERATE].get_ri64("cur-value")?;
        data.punit = sensor_slice[ACPIBAT_CHARGERATE].get_rstring("type")?;
        data.charging = Self::validate(&sensor_slice[ACPIBAT_CHARGING])?.get_ri64("cur-value")?;

        // Sometimes battery can have problems.
        // Read that in https://github.com/NetBSD/src/blob/trunk/sys/dev/acpi/acpi_bat.c line 555.
        if data.charging == 0
            && sensor_slice[ACPIBAT_DISCHARGERATE].get_rstring("state")? == "invalid"
        {
            data.charging = -1;
        }

        Ok(data)
    }

    fn validate(value: &plist::Value) -> Result<&plist::Value> {
        match value.get_rstring("state")? {
            "valid" => Ok(value),
            _ => Err(Error::invalid_data("Invalid section")),
        }
    }

    fn val_cur_value(value: &plist::Value) -> Result<u64> {
        Self::validate(value)?.get_ru64("cur-value")
    }
}

#[derive(Debug, Default)]
pub struct SysMonDevice {
    pub name: String,

    energy: Energy,
    energy_full: Energy,
    energy_full_design: Energy,
    energy_rate: Power,
    state: State,
    voltage: ElectricPotential,
}

impl SysMonDevice {
    // data.name should match the SysMonDevice object.name already
    pub fn new(data: EnvSysDevice) -> Result<Self> {
        let mut bat = Self {
            ..Default::default()
        };

        match bat.refresh(data) {
            Ok(_) => Ok(bat),
            Err(e) => Err(e),
        }
    }

    // data.name should match the SysMonDevice object.name already
    pub fn refresh(&mut self, data: EnvSysDevice) -> Result<()> {
        let design_voltage = microvolt!(data.design_voltage);

        self.name = data.name;
        self.voltage = microvolt!(data.voltage);

        // Cound not test Watt hour, it is an assumption based on doc
        match data.eunit {
            "Ampere hour" => {
                self.energy = microampere_hour!(data.energy) * design_voltage;
                self.energy_full = microampere_hour!(data.energy_full) * design_voltage;
                self.energy_full_design =
                    microampere_hour!(data.energy_full_design) * design_voltage;
            }
            "Watt hour" => {
                self.energy = microwatt_hour!(data.energy);
                self.energy_full = microwatt_hour!(data.energy_full);
                self.energy_full_design = microwatt_hour!(data.energy_full_design);
            }
            _ => return Err(Error::invalid_data("Unit not supported")),
        }

        // Cound not test Watt, it is an assumption based on doc.
        // Beware NetBSD needs some delay updating fields properly.
        // In tests, the refresh timeout is not always respected.
        self.energy_rate = match (data.charging, data.punit) {
            (1, "Ampere") => microampere!(data.charge_rate.abs()) * design_voltage,
            (1, "Watts") => microwatt!(data.charge_rate.abs()),
            (0, "Ampere") => microampere!(data.discharge_rate.abs()) * design_voltage,
            (0, "Watts") => microwatt!(data.discharge_rate.abs()),
            // The battery has a problem in case of -1, set 0,
            (-1, _) => microwatt!(0),
            _ => return Err(Error::invalid_data("Unit not supported or invalid state")),
        };

        if self.energy >= self.energy_full {
            self.state = State::Full;
        } else if self.energy == microwatt_hour!(0) {
            // Probably won't ever happen or maybe charging after completely empty battery.
            // NetBSD provides warning and critical capacity warning,
            // but this lib does not implement such states.
            self.state = State::Empty;
        } else {
            self.state = match data.charging {
                1 => State::Charging,
                0 => State::Discharging,
                _ => State::Unknown,
            }
        }

        Ok(())
    }
}

// Trait wants the following data.
// NetBSD (10 at the time of the implementation) does not provide all data.
// Thankfully missing fields are Options.

// energy                charge (type for unit)
// energy_full           last full cap
// energy_full_design    design cap
// energy_rate           charge rate or discharge rate.... (state valid or not)
// voltage               voltage
// state                 Can be determined
// technology            N/A
// temperature           N/A
// cycle_count           N/A
// vendor                N/A
// model                 N/A
// serial_number         N/A
// time_to_full          Automatically calculated needs energy_rate and energy and energy_full
// time_to_empty         Automatically calculated needs energy_rate and energy
// state_of_health       Automatically calculated needs energy_full and energy_full_design
// state_of_charge       Automatically calculated needs energy and energy_full

// NetBSD provides some sort of state_of_health with baterry-capacity and charge_state
// but this library provides its own methods of calculation so using them for library consistency.

// Thanks to unitedbsd.com community for this thread.
// https://www.unitedbsd.com/d/486-querying-battery-information-wo-envstat/6

impl BatteryDevice for SysMonDevice {
    fn energy(&self) -> Energy {
        self.energy
    }

    fn energy_full(&self) -> Energy {
        self.energy_full
    }

    fn energy_full_design(&self) -> Energy {
        self.energy_full_design
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
        None
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
        Technology::Unknown
    }

    fn cycle_count(&self) -> Option<u32> {
        None
    }
}

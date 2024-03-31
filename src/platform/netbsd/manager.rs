use crate::platform::traits::BatteryManager;
use crate::{Error, Result};

use super::device::EnvSysDevice;
use super::sysmon::get_system_envsys_plist;
use super::{SysMonDevice, SysMonIterator};

#[derive(Debug)]
pub struct SysMonManager;

impl BatteryManager for SysMonManager {
    type Iterator = SysMonIterator;

    fn new() -> Result<Self> {
        Ok(Self {})
    }

    fn refresh(&self, device: &mut SysMonDevice) -> Result<()> {
        // Sadly NetBSD only have one ioctl that retrieves ALL batteries' data.
        // This means we have to get all data to update only one device as
        // we can't just assume the user will want to update all devices at the same time.
        let envsys = get_system_envsys_plist()?;

        match envsys.get(device.name.as_str()) {
            Some(sensor) => device.refresh(EnvSysDevice::new(device.name.to_owned(), &sensor)?),
            None => Err(Error::not_found("Could not refresh battery")),
        }
    }
}

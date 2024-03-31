use crate::plist::dictionary::IntoIter;

use crate::platform::traits::BatteryIterator;
use crate::Result;

use super::device::EnvSysDevice;
use super::{sysmon::get_system_envsys_plist, SysMonDevice, SysMonManager};

use std::fmt;
use std::sync::Arc;

pub struct SysMonIterator {
    #[allow(dead_code)]
    manager: Arc<SysMonManager>,
    iter: IntoIter,
}

impl Iterator for SysMonIterator {
    type Item = Result<SysMonDevice>;

    fn next(&mut self) -> Option<Self::Item> {
        for (key, sensor) in &mut self.iter {
            // Iterator over any kind of sensors so "Not a valid battery" should just continue.
            // Same for battery absent as it does not mean the battery is invalid, just absent.
            match EnvSysDevice::new(key, &sensor) {
                Ok(envsysdev) => match SysMonDevice::new(envsysdev) {
                    Ok(bat) => return Some(Ok(bat)),
                    Err(e) => return Some(Err(e)),
                },
                Err(e) => match e.to_string().as_str() {
                    "Not a valid battery" => continue,
                    "Battery absent" => continue,
                    _ => return Some(Err(e)),
                },
            }
        }

        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // Can't predict number of batteries so keep default.
        (0, None)
    }
}

impl BatteryIterator for SysMonIterator {
    type Manager = SysMonManager;
    type Device = SysMonDevice;

    fn new(manager: Arc<Self::Manager>) -> Result<Self> {
        Ok(Self {
            manager,
            iter: get_system_envsys_plist()?.into_iter(),
        })
    }
}

impl fmt::Debug for SysMonIterator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "SysMonIterator {{ manager: {:?}, Iter: 'Does not implement Debug'}}",
            self.manager
        )
    }
}

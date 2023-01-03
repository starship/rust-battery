use std::fmt;
use std::sync::Arc;

use super::{ffi, PowerDevice, PowerManager};
use crate::platform::traits::BatteryIterator;
use crate::Result;

pub struct PowerIterator {
    #[allow(dead_code)]
    manager: Arc<PowerManager>,
    inner: ffi::DeviceIterator,
}

impl Iterator for PowerIterator {
    type Item = Result<PowerDevice>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner.next() {
                None => return None,
                Some(handle) => {
                    match PowerDevice::try_from(handle) {
                        Ok(Some(device)) => return Some(Ok(device)),
                        Ok(None) => continue,
                        Err(e) => return Some(Err(e)),
                    };
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl BatteryIterator for PowerIterator {
    type Manager = PowerManager;
    type Device = PowerDevice;

    fn new(manager: Arc<Self::Manager>) -> Result<Self> {
        let inner = ffi::DeviceIterator::new()?;
        Ok(Self { manager, inner })
    }
}

impl fmt::Debug for PowerIterator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (start, end) = self.size_hint();
        f.debug_struct("WindowsIterator")
            .field("start", &start)
            .field("end", &end)
            .finish()
    }
}

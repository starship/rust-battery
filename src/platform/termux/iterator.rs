use std::sync::Arc;
use crate::platform::traits::BatteryIterator;
use crate::platform::termux::device::TermuxDevice;
use crate::platform::termux::manager::TermuxManager;
use crate::Result;

#[derive(Debug)]
pub struct TermuxIterator {
    #[allow(dead_code)]
    manager: Arc<TermuxManager>,
    device: Option<TermuxDevice>,
}

impl Iterator for TermuxIterator {
    type Item = Result<TermuxDevice>;

    fn next(&mut self) -> Option<Self::Item> {
        self.device.take().map(|device| Ok(device))
    }
}

impl BatteryIterator for TermuxIterator {
    type Manager = TermuxManager;
    type Device = TermuxDevice;

    fn new(manager: Arc<TermuxManager>) -> Result<Self> {
        let status = manager.get_status()?;
        let device = TermuxDevice::new(status);
        Ok(Self {
            manager,
            device: Some(device),
        })
    }
}

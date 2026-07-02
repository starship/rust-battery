use std::process::Command;
use crate::platform::traits::*;
use crate::Result;
use super::device::TermuxDevice;
use super::iterator::TermuxIterator;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct BatteryStatus {
    pub present: bool,
    pub technology: String,
    pub health: String,
    pub plugged: String,
    pub status: String,
    pub temperature: f32,
    pub voltage: i32,
    pub current: i32,
    pub percentage: i32,
    pub level: i32,
    pub scale: i32,
    pub charge_counter: i32,
}

#[derive(Debug)]
pub struct TermuxManager;

impl BatteryManager for TermuxManager {
    type Iterator = TermuxIterator;

    fn new() -> Result<Self> {
        Ok(TermuxManager)
    }

    fn refresh(&self, device: &mut TermuxDevice) -> Result<()> {
        device.refresh()
    }
}

impl TermuxManager {
    pub fn get_status(&self) -> Result<BatteryStatus> {
        let output = Command::new("termux-battery-status")
            .output()
            .map_err(|e| crate::Error::from(e))?;
        
        if !output.status.success() {
            return Err(crate::Error::from(std::io::Error::new(std::io::ErrorKind::Other, "Failed to get battery status")));
        }
        
        let status: BatteryStatus = serde_json::from_slice(&output.stdout)
            .map_err(|e| crate::Error::from(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())))?;
        
        Ok(status)
    }
}

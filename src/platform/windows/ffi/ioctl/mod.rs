#![allow(clippy::unreadable_literal)]

// Each sub-module represents a C-level struct to respective IOCTL request
// and idiomatic Rust struct around it.

mod info;
mod status;

pub use self::info::BatteryInformation;
pub use self::status::BatteryStatus;

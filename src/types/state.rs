use std::fmt;
use std::io;
use std::str;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize};
use serde::de::{Error, Unexpected};

/// Possible battery state values.
///
/// Unknown can mean either controller returned unknown,
/// or not able to retrieve state due to some error.
#[derive(Debug, Eq, PartialEq, Copy, Clone, Serialize)]
#[cfg_attr(feature = "config-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum State {
    Unknown,
    Charging,
    Discharging,
    Empty,
    Full,
}

impl<'de> Deserialize<'de> for State {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        String::deserialize(deserializer).and_then(|s| State::from_str(&s).map_err(|_| D::Error::invalid_value(Unexpected::Str(&s), &"State")))
    }
}

impl str::FromStr for State {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO: Support strings that starts with `\0`
        // TODO: Support `not charging` value
        // Ref: `up_device_supply_get_state` function at
        //https://gitlab.freedesktop.org/upower/upower/blob/master/src/linux/up-device-supply.c#L452
        match s {
            _ if s.eq_ignore_ascii_case("Unknown") => Ok(State::Unknown),
            _ if s.eq_ignore_ascii_case("Empty") => Ok(State::Empty),
            _ if s.eq_ignore_ascii_case("Full") => Ok(State::Full),
            _ if s.eq_ignore_ascii_case("Charging") => Ok(State::Charging),
            _ if s.eq_ignore_ascii_case("Discharging") => Ok(State::Discharging),
            _ => Err(io::Error::from(io::ErrorKind::InvalidData)),
        }
    }
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let display = match self {
            State::Unknown => "unknown",
            State::Charging => "charging",
            State::Discharging => "discharging",
            State::Empty => "empty",
            State::Full => "full",
        };

        write!(f, "{}", display)
    }
}

impl Default for State {
    fn default() -> Self {
        State::Unknown
    }
}

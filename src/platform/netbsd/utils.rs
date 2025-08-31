use crate::plist;
use crate::{Error, Result};

// This file contains an extention trait because plist provides options and not results.
// This is here only to bring better readability into device.rs by avoiding chains of conversions.

pub trait AsResult {
    fn as_rdict(&self) -> Result<&plist::Dictionary>;
    fn as_rslice(&self) -> Result<&[plist::Value]>;
}

impl AsResult for plist::Value {
    #[inline]
    fn as_rdict(&self) -> Result<&plist::Dictionary> {
        self.as_dictionary()
            .ok_or(Error::invalid_data("Cannot convert value to Dict"))
    }

    #[inline]
    fn as_rslice(&self) -> Result<&[plist::Value]> {
        Ok(self
            .as_array()
            .ok_or(Error::invalid_data("Cannot convert value to slice"))?
            .as_slice())
    }
}

pub trait GetResult {
    fn get_rdict(&self, key: &str) -> Result<&plist::Dictionary>;
    fn get_ru64(&self, key: &str) -> Result<u64>;
    fn get_ri64(&self, key: &str) -> Result<i64>;
    fn get_rbool(&self, key: &str) -> Result<bool>;
    fn get_rstring(&self, key: &str) -> Result<&str>;
}

impl GetResult for plist::Dictionary {
    #[inline]
    fn get_rdict(&self, key: &str) -> Result<&plist::Dictionary> {
        self.get(key)
            .ok_or(Error::invalid_data("Cannot convert value to Dict"))?
            .as_rdict()
    }

    #[inline]
    fn get_ru64(&self, key: &str) -> Result<u64> {
        self.get(key)
            .and_then(|x| x.as_unsigned_integer())
            .ok_or(Error::invalid_data("Cannot convert value to u64"))
    }

    #[inline]
    fn get_ri64(&self, key: &str) -> Result<i64> {
        self.get(key)
            .and_then(|x| x.as_signed_integer())
            .ok_or(Error::invalid_data("Cannot convert value to i64"))
    }

    #[inline]
    fn get_rbool(&self, key: &str) -> Result<bool> {
        self.get(key)
            .and_then(|x| x.as_boolean())
            .ok_or(Error::invalid_data("Cannot convert value to bool"))
    }

    #[inline]
    fn get_rstring(&self, key: &str) -> Result<&str> {
        self.get(key)
            .and_then(|x| x.as_string())
            .ok_or(Error::invalid_data("Cannot convert value to string"))
    }
}

impl GetResult for plist::Value {
    #[inline]
    fn get_rdict(&self, key: &str) -> Result<&plist::Dictionary> {
        self.as_rdict()?.get_rdict(key)
    }

    #[inline]
    fn get_ru64(&self, key: &str) -> Result<u64> {
        self.as_rdict()?.get_ru64(key)
    }

    #[inline]
    fn get_ri64(&self, key: &str) -> Result<i64> {
        self.as_rdict()?.get_ri64(key)
    }

    #[inline]
    fn get_rbool(&self, key: &str) -> Result<bool> {
        self.as_rdict()?.get_rbool(key)
    }

    #[inline]
    fn get_rstring(&self, key: &str) -> Result<&str> {
        self.as_rdict()?.get_rstring(key)
    }
}

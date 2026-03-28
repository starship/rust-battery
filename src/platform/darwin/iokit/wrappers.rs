#![allow(deprecated)]
use std::ffi::CStr;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

use mach::port::mach_port_t;
use mach::{kern_return, mach_port, port, traps};
use objc2_core_foundation::{
    kCFAllocatorDefault, CFDictionary, CFMutableDictionary, CFRetained, CFString, CFType,
};
use objc2_io_kit::{
    io_iterator_t, io_object_t, kIOMasterPortDefault, IOIteratorNext, IOMasterPort,
    IOObjectRelease, IORegistryEntryCreateCFProperties, IOServiceGetMatchingServices,
    IOServiceMatching,
};

use crate::{Error, Result};

pub const IOPM_SERVICE_NAME: &CStr = c"IOPMPowerSource";

#[derive(Debug)]
pub struct IoMasterPort(mach_port_t);

impl IoMasterPort {
    pub fn new() -> Result<IoMasterPort> {
        let mut master_port: port::mach_port_t = port::MACH_PORT_NULL;

        unsafe {
            // `IOMasterPort` and `kIOMasterPortDefault` are deprecated with macOS 12.
            // TODO: replace with `IOMainPort` and `kIOMainPortDefault` respectively.
            kern_try!(IOMasterPort(kIOMasterPortDefault, &mut master_port));
        }

        Ok(IoMasterPort(master_port))
    }

    pub fn get_services(&self) -> Result<IoIterator> {
        let service =
            unsafe { IOServiceMatching(IOPM_SERVICE_NAME.as_ptr()) }.ok_or(Error::not_found(
                format!("Cannot find IOService named {:?}", IOPM_SERVICE_NAME),
            ));

        let mut iterator = IoIterator::default();

        unsafe {
            kern_try!(IOServiceGetMatchingServices(
                self.0,
                Some(CFRetained::cast_unchecked::<CFDictionary>(service?)),
                &mut *iterator
            ));
        }

        Ok(iterator)
    }
}

impl Drop for IoMasterPort {
    fn drop(&mut self) {
        let result = unsafe { mach_port::mach_port_deallocate(traps::mach_task_self(), self.0) };
        assert_eq!(result, kern_return::KERN_SUCCESS);
    }
}

#[derive(Debug)]
pub struct IoObject(io_object_t);

impl IoObject {
    /// Returns typed dictionary with this object properties.
    /// In our case all keys are CFStrings, so there is no need to return
    /// untyped dict here.
    pub fn properties(&self) -> Result<CFRetained<CFDictionary<CFString, CFType>>> {
        unsafe {
            let mut props = std::ptr::null_mut();

            kern_try!(IORegistryEntryCreateCFProperties(
                self.0,
                &mut props,
                kCFAllocatorDefault,
                0
            ));
            let data = CFRetained::<CFMutableDictionary>::from_raw(NonNull::new_unchecked(props));
            let result = CFRetained::cast_unchecked::<CFDictionary<CFString, CFType>>(data);

            Ok(result)
        }
    }
}

impl Drop for IoObject {
    fn drop(&mut self) {
        let result = IOObjectRelease(self.0);
        assert_eq!(result, kern_return::KERN_SUCCESS);
    }
}

#[derive(Debug)]
pub struct IoIterator(io_iterator_t);

impl Deref for IoIterator {
    type Target = io_iterator_t;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for IoIterator {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Iterator for IoIterator {
    type Item = IoObject;

    fn next(&mut self) -> Option<Self::Item> {
        match IOIteratorNext(self.0) {
            0 => None, // TODO: Should not there be some `NULL`?
            io_object => Some(IoObject(io_object)),
        }
    }
}

impl Drop for IoIterator {
    fn drop(&mut self) {
        let result = IOObjectRelease(self.0);
        assert_eq!(result, kern_return::KERN_SUCCESS);
    }
}

impl Default for IoIterator {
    // It is extremely unsafe and inner field MUST BE initialized
    // before the further `Drop::drop` call
    fn default() -> IoIterator {
        let inner = unsafe { mem::zeroed() };
        IoIterator(inner)
    }
}

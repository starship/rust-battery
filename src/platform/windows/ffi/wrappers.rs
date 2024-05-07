// Wrappers around the FFI things that should be freed later.
// It is better to Drop than free them manually.

use std::ffi::c_void;
use std::ops;
use std::ptr::null_mut;

use windows_sys::Win32::Devices::DeviceAndDriverInstallation as setupapi;
use windows_sys::Win32::Foundation;

#[derive(Debug)]
pub struct InterfaceDetailData(*mut setupapi::SP_DEVICE_INTERFACE_DETAIL_DATA_W);

impl From<*mut setupapi::SP_DEVICE_INTERFACE_DETAIL_DATA_W> for InterfaceDetailData {
    fn from(p: *mut setupapi::SP_DEVICE_INTERFACE_DETAIL_DATA_W) -> Self {
        Self(p)
    }
}

impl ops::Deref for InterfaceDetailData {
    type Target = *mut setupapi::SP_DEVICE_INTERFACE_DETAIL_DATA_W;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for InterfaceDetailData {
    fn drop(&mut self) {
        let res = unsafe { Foundation::LocalFree(self.0 as *mut c_void) };
        debug_assert_eq!(
            res,
            null_mut(),
            "Unable to free device interface detail data"
        );
    }
}

#[derive(Debug)]
pub struct Handle(pub(crate) Foundation::HANDLE);

impl From<Foundation::HANDLE> for Handle {
    fn from(handle: Foundation::HANDLE) -> Self {
        Self(handle)
    }
}

impl ops::Deref for Handle {
    type Target = Foundation::HANDLE;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for Handle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        let res = unsafe { Foundation::CloseHandle(self.0) };
        debug_assert_ne!(res, 0, "Unable to close device handle");
    }
}

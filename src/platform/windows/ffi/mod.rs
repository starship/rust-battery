use std::default::Default;
use std::ffi;
use std::io;
use std::iter;
use std::mem;
use std::mem::MaybeUninit;
use std::os::windows::raw::HANDLE;
use std::ptr;

use windows_sys::core::*;
use windows_sys::Win32::Devices::DeviceAndDriverInstallation as setupapi;
use windows_sys::Win32::Foundation;
use windows_sys::Win32::Storage::FileSystem as fileapi;
use windows_sys::Win32::System::Power as systempower;
use windows_sys::Win32::System::IO as ioapiset;

mod ioctl;
mod wide_string;
mod wrappers;

use self::wide_string::WideString;
use self::wrappers::*;

#[derive(Debug)]
pub struct DeviceIterator {
    device: setupapi::HDEVINFO,
    current: u32,
}

impl DeviceIterator {
    pub fn new() -> io::Result<DeviceIterator> {
        let hdev = unsafe {
            setupapi::SetupDiGetClassDevsW(
                &setupapi::GUID_DEVCLASS_BATTERY,
                ptr::null() as PCWSTR,
                ptr::null_mut(),
                setupapi::DIGCF_PRESENT | setupapi::DIGCF_DEVICEINTERFACE,
            )
        };
        if hdev as HANDLE == Foundation::INVALID_HANDLE_VALUE {
            Err(io::Error::last_os_error())
        } else {
            Ok(DeviceIterator {
                device: hdev,
                current: 0,
            })
        }
    }

    fn get_interface_data(&self) -> io::Result<setupapi::SP_DEVICE_INTERFACE_DATA> {
        let mut data = unsafe { mem::zeroed::<setupapi::SP_DEVICE_INTERFACE_DATA>() };
        data.cbSize = mem::size_of::<setupapi::SP_DEVICE_INTERFACE_DATA>() as u32;
        let result = unsafe {
            setupapi::SetupDiEnumDeviceInterfaces(
                self.device,
                ptr::null_mut(),
                &setupapi::GUID_DEVCLASS_BATTERY,
                self.current,
                &mut data,
            )
        };

        // TODO: Add trace
        if result == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(data)
        }
    }

    fn get_interface_detail(
        &self,
        data: &mut setupapi::SP_DEVICE_INTERFACE_DATA,
    ) -> io::Result<InterfaceDetailData> {
        let mut buf_size = 0;
        unsafe {
            setupapi::SetupDiGetDeviceInterfaceDetailW(
                self.device,
                data,
                ptr::null_mut(),
                0,
                &mut buf_size,
                ptr::null_mut(),
            )
        };
        let result = unsafe { Foundation::GetLastError() };
        if result != Foundation::ERROR_INSUFFICIENT_BUFFER {
            return Err(io::Error::from_raw_os_error(result as i32));
        }

        let pdidd = unsafe {
            windows_sys::Win32::System::Memory::LocalAlloc(
                windows_sys::Win32::System::Memory::LPTR,
                buf_size as _,
            ) as *mut setupapi::SP_DEVICE_INTERFACE_DETAIL_DATA_W
        };
        unsafe {
            (*pdidd).cbSize = mem::size_of::<setupapi::SP_DEVICE_INTERFACE_DETAIL_DATA_W>() as u32;
        }
        unsafe {
            setupapi::SetupDiGetDeviceInterfaceDetailW(
                self.device,
                data,
                pdidd,
                buf_size,
                &mut buf_size,
                ptr::null_mut(),
            )
        };
        let result = unsafe { Foundation::GetLastError() };
        if result != 0 {
            return Err(io::Error::from_raw_os_error(result as i32));
        }

        Ok(pdidd.into())
    }

    fn get_handle(&self, pdidd: &InterfaceDetailData) -> io::Result<Handle> {
        let device_path = unsafe {
            let dp = std::ptr::addr_of!((***pdidd).DevicePath);
            (*dp).as_ptr()
        };

        let file = unsafe {
            fileapi::CreateFileW(
                device_path,
                Foundation::GENERIC_READ | Foundation::GENERIC_WRITE,
                fileapi::FILE_SHARE_READ | fileapi::FILE_SHARE_WRITE,
                ptr::null(),
                fileapi::OPEN_EXISTING,
                fileapi::FILE_ATTRIBUTE_NORMAL,
                ptr::null_mut(),
            )
        };
        if file == Foundation::INVALID_HANDLE_VALUE {
            Err(io::Error::last_os_error())
        } else {
            Ok(file.into())
        }
    }

    fn get_tag(&self, handle: &mut Handle) -> io::Result<systempower::BATTERY_QUERY_INFORMATION> {
        let mut query = unsafe { mem::zeroed::<systempower::BATTERY_QUERY_INFORMATION>() };
        let mut wait_timeout: u32 = 0;
        let mut bytes_returned: u32 = 0;
        let mut battery_tag = { query.BatteryTag };

        let res = unsafe {
            ioapiset::DeviceIoControl(
                handle.0,
                systempower::IOCTL_BATTERY_QUERY_TAG,
                &mut wait_timeout as *mut _ as *mut ffi::c_void,
                mem::size_of_val(&wait_timeout) as _,
                &mut battery_tag as *mut _ as *mut ffi::c_void,
                mem::size_of_val(&bytes_returned) as _,
                &mut bytes_returned as *mut _,
                ptr::null_mut(),
            )
        };

        query.BatteryTag = battery_tag;
        if res == 0 || query.BatteryTag == 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(query)
    }

    pub fn prepare_handle(&self) -> io::Result<Handle> {
        let mut interface_data = self.get_interface_data()?;
        let interface_detail_data = self.get_interface_detail(&mut interface_data)?;

        self.get_handle(&interface_detail_data)
    }
}

impl iter::Iterator for DeviceIterator {
    type Item = DeviceHandle;

    fn next(&mut self) -> Option<Self::Item> {
        let mut handle = self.prepare_handle().ok()?;
        let tag = self.get_tag(&mut handle).ok()?;

        self.current += 1;

        Some(DeviceHandle {
            //            interface_details: interface_detail_data,
            handle,
            tag,
        })
    }
}

impl Drop for DeviceIterator {
    fn drop(&mut self) {
        let res = unsafe { setupapi::SetupDiDestroyDeviceInfoList(self.device) };
        debug_assert_eq!(res, 1, "Unable to destroy DeviceInfoList");
    }
}

// Our inner representation of the battery device.
pub struct DeviceHandle {
    //    interface_details: InterfaceDetailData,
    pub handle: Handle,
    // TODO: Carry only `.BatteryTag` field ?
    pub tag: systempower::BATTERY_QUERY_INFORMATION,
}

impl DeviceHandle {
    pub fn information(&mut self) -> io::Result<ioctl::BatteryInformation> {
        let mut query = unsafe { mem::zeroed::<systempower::BATTERY_QUERY_INFORMATION>() };
        query.BatteryTag = self.tag.BatteryTag;
        let mut out = MaybeUninit::<systempower::BATTERY_INFORMATION>::uninit();
        let mut bytes_returned = 0u32;

        let res = unsafe {
            ioapiset::DeviceIoControl(
                self.handle.0,
                systempower::IOCTL_BATTERY_QUERY_INFORMATION,
                &mut query as *mut _ as *mut ffi::c_void,
                mem::size_of::<systempower::BATTERY_QUERY_INFORMATION>() as _,
                &mut out as *mut _ as *mut ffi::c_void,
                mem::size_of::<ioctl::BatteryInformation>() as _,
                &mut bytes_returned as *mut _,
                ptr::null_mut(),
            )
        };

        if res == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(unsafe { out.assume_init() }.into())
        }
    }

    pub fn status(&mut self) -> io::Result<ioctl::BatteryStatus> {
        let mut query = unsafe { mem::zeroed::<systempower::BATTERY_WAIT_STATUS>() };
        query.BatteryTag = self.tag.BatteryTag;
        let mut out = MaybeUninit::<systempower::BATTERY_STATUS>::uninit();
        let mut bytes_returned = 0u32;

        let res = unsafe {
            ioapiset::DeviceIoControl(
                *self.handle,
                systempower::IOCTL_BATTERY_QUERY_STATUS,
                &mut query as *mut _ as *mut ffi::c_void,
                mem::size_of::<systempower::BATTERY_WAIT_STATUS>() as _,
                &mut out as *mut _ as *mut ffi::c_void,
                mem::size_of::<systempower::BATTERY_STATUS>() as _,
                &mut bytes_returned as *mut _,
                ptr::null_mut(),
            )
        };

        if res == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(unsafe { out.assume_init() }.into())
        }
    }

    // 10ths of a degree Kelvin (or decikelvin)
    pub fn temperature(&mut self) -> io::Result<u64> {
        let mut query = unsafe { mem::zeroed::<systempower::BATTERY_QUERY_INFORMATION>() };
        query.BatteryTag = self.tag.BatteryTag;
        query.InformationLevel = systempower::BatteryTemperature;
        let mut out: u64 = 0;
        let mut bytes_returned: u32 = 0;

        let res = unsafe {
            ioapiset::DeviceIoControl(
                *self.handle,
                systempower::IOCTL_BATTERY_QUERY_INFORMATION,
                &mut query as *mut _ as *mut ffi::c_void,
                // Since wrapper is a newtype struct, `mem::size_of` will be the same as with
                // underline structure. Yet, this might lead to bug if wrapper structure will change.
                // TODO: Get memory size of the underline struct directly
                mem::size_of::<systempower::BATTERY_QUERY_INFORMATION>() as _,
                &mut out as *mut _ as *mut ffi::c_void,
                mem::size_of_val(&out) as _,
                &mut bytes_returned as *mut _,
                ptr::null_mut(),
            )
        };

        if res == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(out)
        }
    }

    pub fn device_name(&mut self) -> io::Result<String> {
        self.query_string(systempower::BatteryDeviceName)
    }

    pub fn manufacture_name(&mut self) -> io::Result<String> {
        self.query_string(systempower::BatteryManufactureName)
    }

    pub fn serial_number(&mut self) -> io::Result<String> {
        self.query_string(systempower::BatterySerialNumber)
    }

    fn query_string(
        &mut self,
        level: systempower::BATTERY_QUERY_INFORMATION_LEVEL,
    ) -> io::Result<String> {
        let mut query = unsafe { mem::zeroed::<systempower::BATTERY_QUERY_INFORMATION>() };
        query.BatteryTag = self.tag.BatteryTag;
        query.InformationLevel = level;
        let mut out = WideString::default();
        let mut bytes_returned = 0u32;

        let res = unsafe {
            ioapiset::DeviceIoControl(
                *self.handle,
                systempower::IOCTL_BATTERY_QUERY_INFORMATION,
                &mut query as *mut _ as *mut ffi::c_void,
                mem::size_of::<systempower::BATTERY_QUERY_INFORMATION>() as _,
                out.as_mut_ptr() as *mut _ as *mut ffi::c_void,
                (out.len() * 2) as _,
                &mut bytes_returned as *mut _,
                ptr::null_mut(),
            )
        };

        out.truncate(bytes_returned as usize);

        if res == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(out.into())
        }
    }
}

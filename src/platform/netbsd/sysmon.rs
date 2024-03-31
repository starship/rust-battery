use std::fs;
use std::os::fd::AsRawFd;
use std::ptr::{addr_of_mut, null_mut, NonNull};
use std::slice;

use libc::{c_void, size_t};

use crate::nix::sys::mman::munmap;
use crate::plist;
use crate::Error;

// https://man.netbsd.org/ioctl.9
// man ioctlprint
// ENVSYS_GETDICTIONARY _IOWR('E', 0, struct plistref)  0xc0104500
// https://github.com/NetBSD/src/blob/trunk/common/include/prop/plistref.h#L43

const SYSMON_PATH: &str = "/dev/sysmon";

// ioctl is rw even if we only need read as per system definition
ioctl_readwrite!(envsys_getdictionary, b'E', 0, Plistref);

#[derive(Debug)]
#[repr(C)]
pub struct Plistref {
    pref_plist: *mut u8,
    pref_len: size_t,
}

impl Default for Plistref {
    fn default() -> Self {
        Self {
            pref_plist: null_mut(),
            pref_len: 0,
        }
    }
}

pub fn get_system_envsys_plist() -> Result<plist::Dictionary, Error> {
    let mut plist_ref: Plistref = Plistref::default();

    let file = fs::OpenOptions::new().read(true).open(SYSMON_PATH)?;
    let fd = file.as_raw_fd();

    unsafe {
        envsys_getdictionary(fd, addr_of_mut!(plist_ref))?;
    }

    if plist_ref.pref_len == 0 || plist_ref.pref_plist == null_mut() {
        return Err(Error::invalid_data("Invalid result of EnvSys ioctl"));
    }

    let dict = plist::from_bytes(unsafe {
        slice::from_raw_parts(plist_ref.pref_plist, plist_ref.pref_len)
    })?;

    unsafe {
        // The netbsd libprog says ioctl returned mmap'ed memory that must be munmap'ed.
        // https://www.unitedbsd.com/d/486-querying-battery-information-wo-envstat/4
        // https://github.com/NetBSD/src/blob/trunk/common/lib/libprop/prop_kern.c
        // Also unwrap is fine as we already check for non_null above.
        munmap(
            NonNull::new(plist_ref.pref_plist as *mut c_void).unwrap(),
            plist_ref.pref_len,
        )?;
    }

    Ok(dict)
}

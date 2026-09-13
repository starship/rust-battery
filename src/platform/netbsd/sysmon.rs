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

    if plist_ref.pref_len == 0 || plist_ref.pref_plist.is_null() {
        return Err(Error::invalid_data("Invalid result of EnvSys ioctl"));
    }

    let dict = parse_envsys_plist(unsafe {
        slice::from_raw_parts(plist_ref.pref_plist, plist_ref.pref_len)
    });

    unsafe {
        // The netbsd libprop says ioctl returned mmap'ed memory that must be munmap'ed.
        // https://www.unitedbsd.com/d/486-querying-battery-information-wo-envstat/4
        // https://github.com/NetBSD/src/blob/trunk/common/lib/libprop/prop_kern.c
        // Also unwrap is fine as we already check for non_null above.
        munmap(
            NonNull::new(plist_ref.pref_plist as *mut c_void).unwrap(),
            plist_ref.pref_len,
        )?;
    }

    dict
}

/// Parse the XML plist the kernel hands back through `ENVSYS_GETDICTIONARY`.
///
/// libprop externalizes the dictionary as a C string and reports its length
/// *including* the terminating NUL (`pref_len = strlen(buf) + 1`, see
/// `common/lib/libprop/prop_kern.c`). Since plist 1.7.4 the XML reader
/// rejects any non-whitespace after the closing `</plist>`, so the NUL has to
/// come off before parsing or every battery on NetBSD reads as
/// "Problem while processing plist".
fn parse_envsys_plist(bytes: &[u8]) -> Result<plist::Dictionary, Error> {
    let end = bytes.iter().rposition(|b| *b != 0).map_or(0, |i| i + 1);
    Ok(plist::from_bytes(&bytes[..end])?)
}

#[cfg(test)]
mod tests {
    use super::parse_envsys_plist;

    const ENVSYS: &str = concat!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
        "<!DOCTYPE plist PUBLIC \"-//Apple Computer//DTD PLIST 1.0//EN\" ",
        "\"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n",
        "<plist version=\"1.0\">\n<dict>\n\t<key>acpibat0</key>\n\t<array>\n",
        "\t\t<dict>\n\t\t\t<key>cur-value</key>\n\t\t\t<integer>1</integer>\n",
        "\t\t\t<key>description</key>\n\t\t\t<string>present</string>\n",
        "\t\t\t<key>state</key>\n\t\t\t<string>valid</string>\n",
        "\t\t\t<key>type</key>\n\t\t\t<string>Indicator</string>\n\t\t</dict>\n",
        "\t</array>\n</dict>\n</plist>\n"
    );

    #[test]
    fn the_kernels_trailing_nul_is_not_part_of_the_document() {
        let mut bytes = ENVSYS.as_bytes().to_vec();
        bytes.push(0);
        let dict = parse_envsys_plist(&bytes).expect("a NUL-terminated envsys plist parses");
        assert!(dict.contains_key("acpibat0"));
    }

    #[test]
    fn a_plist_without_the_nul_still_parses() {
        let dict = parse_envsys_plist(ENVSYS.as_bytes()).expect("parses");
        assert!(dict.contains_key("acpibat0"));
    }

    #[test]
    fn only_nuls_is_an_error_not_a_panic() {
        assert!(parse_envsys_plist(&[0, 0]).is_err());
        assert!(parse_envsys_plist(&[]).is_err());
    }
}

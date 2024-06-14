use std::{
    ffi::{self, CStr},
    mem,
};

use anyhow::{anyhow, Context};
use windows::{
    core::{s, PCSTR},
    Win32::{
        Foundation::FreeLibrary,
        System::{
            Diagnostics::Debug::WriteProcessMemory,
            LibraryLoader::{GetProcAddress, LoadLibraryA},
            Memory::{VirtualAllocEx, MEM_COMMIT, PAGE_READWRITE},
            Threading::{
                CreateRemoteThread, OpenProcess, WaitForSingleObject, INFINITE,
                PROCESS_CREATE_THREAD, PROCESS_VM_OPERATION, PROCESS_VM_WRITE,
            },
        },
    },
};

use crate::managed_types::ManagedHandle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QtVersion {
    Qt5,
    Qt6,
}

pub fn set_logging_rules(
    pid: u32,
    version: QtVersion,
    core_path: &CStr,
    rules: &str,
) -> anyhow::Result<()> {
    unsafe {
        let qt_core =
            LoadLibraryA(PCSTR(core_path.as_ptr() as *const u8)).context("LoadLibraryA")?;
        let addr = GetProcAddress(
            qt_core,
            s!("?setFilterRules@QLoggingCategory@@SAXAEBVQString@@@Z"),
        )
        .ok_or_else(|| anyhow!("Failed to find QLoggingCategory::setFilterRules"))?;

        let process = OpenProcess(
            PROCESS_CREATE_THREAD | PROCESS_VM_WRITE | PROCESS_VM_OPERATION,
            false,
            pid,
        )
        .context("OpenProcess")?;
        let process = ManagedHandle::new(process);

        let allocation_size = version.allocation_size(rules);
        let allocation =
            VirtualAllocEx(*process, None, allocation_size, MEM_COMMIT, PAGE_READWRITE);
        if allocation.is_null() {
            return Err(anyhow!("Failed to allocate memory in process"));
        }

        let (data, start_addr) = version.make_qstring(allocation, rules);
        if data.len() > allocation_size {
            return Err(anyhow!("QString was larger than expected"));
        }

        WriteProcessMemory(
            *process,
            allocation,
            data.as_ptr() as *mut ffi::c_void,
            data.len(),
            None,
        )
        .context("WriteProcessMemory")?;
        let thread = CreateRemoteThread(
            *process,
            None,
            0,
            Some(std::mem::transmute::<
                unsafe extern "system" fn() -> isize,
                unsafe extern "system" fn(*mut std::ffi::c_void) -> u32,
            >(addr)),
            Some(start_addr),
            0,
            None,
        )
        .context("CreateRemoteThread")?;

        WaitForSingleObject(thread, INFINITE);

        FreeLibrary(qt_core).ok();

        Ok(())
    }
}

#[derive(Clone, Copy, bytemuck::NoUninit)]
#[repr(C)]
struct Qt5String {
    d: usize,
    data: Qt5TypedArrayData,
}

#[derive(Clone, Copy, bytemuck::NoUninit)]
#[repr(C)]
struct Qt5TypedArrayData {
    r: ffi::c_int,
    size: ffi::c_int,
    flags: ffi::c_uint,
    #[cfg(target_pointer_width = "64")]
    _unused: ffi::c_uint,
    offset: usize,
}

#[derive(Clone, Copy, bytemuck::NoUninit)]
#[repr(C)]
struct Qt6ArrayData {
    header: usize,
    data: usize,
    length: usize,
}

impl QtVersion {
    pub const fn allocation_size(&self, content: &str) -> usize {
        match *self {
            QtVersion::Qt5 => {
                mem::size_of::<Qt5String>() + content.len() * 2 + mem::size_of::<ffi::c_ushort>()
            }
            QtVersion::Qt6 => content.len() * 2 + mem::size_of::<Qt6ArrayData>(),
        }
    }

    pub fn make_qstring(
        &self,
        start_addr: *mut ffi::c_void,
        content: &str,
    ) -> (Vec<u8>, *mut ffi::c_void) {
        match *self {
            QtVersion::Qt5 => Self::make_qt5string(start_addr, content),
            QtVersion::Qt6 => Self::make_qt6string(start_addr, content),
        }
    }

    fn make_qt5string(start_addr: *mut ffi::c_void, content: &str) -> (Vec<u8>, *mut ffi::c_void) {
        let mut buf = Vec::with_capacity(Self::Qt5.allocation_size(content));
        let n_codepoints = widestring::encode_utf16(content.chars()).count();

        buf.extend_from_slice(bytemuck::bytes_of(&Qt5String {
            d: unsafe { (start_addr as *mut usize).offset(1) as *mut _ as usize },
            data: Qt5TypedArrayData {
                r: -1,
                size: n_codepoints as ffi::c_int,
                flags: 0,
                #[cfg(target_pointer_width = "64")]
                _unused: 0,
                offset: mem::size_of::<Qt5TypedArrayData>(),
            },
        }));

        for codepoint in widestring::encode_utf16(content.chars()).map(u16::to_ne_bytes) {
            buf.push(codepoint[0]);
            buf.push(codepoint[1]);
        }
        // null termination isn't technically required
        buf.push(0);
        buf.push(0);

        (buf, start_addr)
    }

    fn make_qt6string(start_addr: *mut ffi::c_void, content: &str) -> (Vec<u8>, *mut ffi::c_void) {
        // QString { data: QStringPrivate = *QArrayData<char16_t> }
        let mut buf = Vec::with_capacity(Self::Qt6.allocation_size(content));
        for codepoint in widestring::encode_utf16(content.chars()).map(u16::to_ne_bytes) {
            buf.push(codepoint[0]);
            buf.push(codepoint[1]);
        }
        let offset = buf.len();

        buf.extend_from_slice(bytemuck::bytes_of(&Qt6ArrayData {
            header: 0,
            data: start_addr as usize,
            length: offset / 2,
        }));

        (buf, unsafe {
            (start_addr as *mut u8).add(offset) as *mut ffi::c_void
        })
    }
}

#[cfg(all(test, target_pointer_width = "64", target_endian = "little"))]
mod tests {
    use super::*;
    use std::ptr;

    #[test]
    fn qt6() {
        let (buf, _ptr) = QtVersion::Qt6.make_qstring(ptr::null_mut(), "Hello");
        assert_eq!(
            buf,
            b"H\0e\0l\0l\0o\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x05\0\0\0\0\0\0\0"
        )
    }

    #[test]
    fn qt5() {
        let (buf, _ptr) = QtVersion::Qt5.make_qstring(ptr::null_mut(), "Hello");
        assert_eq!(
            buf,
            b"\x08\0\0\0\0\0\0\0\xFF\xFF\xFF\xFF\x05\0\0\0\0\0\0\0\0\0\0\0\x18\0\0\0\0\0\0\0H\0e\0l\0l\0o\0\0\0"
        )
    }
}

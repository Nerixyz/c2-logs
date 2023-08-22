use std::{ffi::OsStr, os::windows::prelude::OsStrExt};

use anyhow::Context;
use windows::{
    core::{Error as WinError, Result, PCWSTR},
    Win32::{
        Foundation::{CloseHandle, HMODULE},
        System::{
            Diagnostics::Debug::{DebugActiveProcess, DebugSetProcessKillOnExit},
            ProcessStatus::{EnumProcessModules, EnumProcesses, GetModuleBaseNameW},
            Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ},
        },
    },
};

const N_PROCESSES: usize = 1024;

pub fn get_chatterino_pid(executable_name: &OsStr) -> anyhow::Result<Option<u32>> {
    let mut pids = [0u32; N_PROCESSES];
    let mut n_pids = 0;
    unsafe {
        EnumProcesses(
            pids.as_mut_ptr(),
            std::mem::size_of::<[u32; N_PROCESSES]>() as u32,
            &mut n_pids,
        )
        .context("EnumerateProcesses")?
    };

    let n_pids = (n_pids as usize) / std::mem::size_of::<u32>();

    let wide_chatterino_name: Vec<u16> = executable_name
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    for pid in &pids[..n_pids] {
        if let Ok(true) = is_chatterino(*pid, &wide_chatterino_name) {
            return Ok(Some(*pid));
        }
    }
    Ok(None)
}

pub fn attach_debugger(pid: u32) -> anyhow::Result<()> {
    unsafe {
        DebugActiveProcess(pid).with_context(|| format!("DebugActiveProcess(pid={pid})"))?;

        DebugSetProcessKillOnExit(false)
            .with_context(|| format!("DebugSetProcessKillOnExit(false) [pid={pid}]"))
    }
}

unsafe fn wstr_eq(PCWSTR(mut lhs): PCWSTR, PCWSTR(mut rhs): PCWSTR) -> bool {
    if lhs.is_null() || rhs.is_null() {
        return lhs == rhs;
    }

    loop {
        if *lhs == *rhs {
            if *lhs == 0 {
                break true;
            }
            lhs = lhs.offset(1);
            rhs = rhs.offset(1);
        } else {
            break false;
        }
    }
}

fn is_chatterino(pid: u32, chatterino_name: &[u16]) -> Result<bool> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid)?;

        let mut process_module = HMODULE::default();
        let mut _needed = 0;
        EnumProcessModules(
            handle,
            &mut process_module,
            std::mem::size_of::<HMODULE>() as u32,
            &mut _needed,
        )?;

        let mut buf = vec![0u16; chatterino_name.len()];

        if GetModuleBaseNameW(handle, process_module, &mut buf) == 0 {
            return Err(WinError::from_win32());
        }

        let _ = CloseHandle(handle);

        Ok(wstr_eq(
            PCWSTR(chatterino_name.as_ptr()),
            PCWSTR(buf.as_ptr()),
        ))
    }
}

#[cfg(test)]
mod tests {
    use windows::core::w;

    use super::*;

    #[test]
    fn eq() {
        unsafe {
            for (lhs, rhs) in [
                (PCWSTR(std::ptr::null_mut()), PCWSTR(std::ptr::null_mut())),
                (w!(""), w!("")),
                (w!("alien"), w!("alien")),
                (w!("alien\0xd"), w!("alien\0no")),
            ] {
                assert!(wstr_eq(lhs, rhs));
                assert!(wstr_eq(rhs, lhs));
            }

            for (lhs, rhs) in [
                (PCWSTR(std::ptr::null_mut()), w!("a")),
                (w!("a"), w!("")),
                (w!("alien"), w!("alienpls")),
                (w!("alien"), w!("blien")),
                (w!("alien\0xd"), w!("alienpls\0no")),
            ] {
                assert!(!wstr_eq(lhs, rhs));
                assert!(!wstr_eq(rhs, lhs));
            }
        }
    }
}

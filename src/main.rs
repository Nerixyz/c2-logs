#![deny(clippy::cargo)]

mod filter;
mod managed_types;
mod printer;
mod processes;
mod str_ext;
mod strings;

use std::{collections::HashSet, ffi::OsString};

use anyhow::{bail, Context};
use clap::Parser;
use filter::FilterMode;
use managed_types::ManagedHandle;
use printer::Printer;
use windows::{
    core::Error as WinError,
    Win32::{
        Foundation::{DBG_EXCEPTION_NOT_HANDLED, HANDLE},
        System::{
            Diagnostics::Debug::{
                ContinueDebugEvent, WaitForDebugEventEx, DEBUG_EVENT, EXIT_PROCESS_DEBUG_EVENT,
                OUTPUT_DEBUG_STRING_EVENT,
            },
            Threading::{OpenProcess, INFINITE, PROCESS_ALL_ACCESS},
        },
    },
};

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(
        short = 'm',
        value_enum,
        default_value = "exclude",
        help = "How to interpret the filters. 'include' will only show the specified categories. 'exclude' will show logs from all categories except the ones specified."
    )]
    mode: FilterMode,
    #[arg(
        long = "exe",
        default_value = "chatterino.exe",
        help = "Use this to specify the name of the chatterino executable."
    )]
    executable: OsString,
    #[arg(long, help = "Use this to specify a specific process-id to attach to.")]
    pid: Option<u32>,
    #[arg(help = "Filters for logging categories, for example 'http', 'hotkeys', or 'irc'.")]
    filters: Vec<OsString>,
}

fn print_debug_events(process_handle: HANDLE, filter: filter::Filter) -> anyhow::Result<()> {
    let mut printer = Printer::new(process_handle, filter);

    loop {
        let mut debug_event: DEBUG_EVENT = unsafe { std::mem::zeroed() };
        unsafe {
            if !WaitForDebugEventEx(&mut debug_event, INFINITE).as_bool() {
                bail!("WaitForDebugEventEx failed: {:?}", WinError::from_win32())
            }
        }

        match debug_event.dwDebugEventCode {
            OUTPUT_DEBUG_STRING_EVENT => {
                let info = unsafe { debug_event.u.DebugString };
                if info.fUnicode != 0 {
                    printer.read_string(info)?;
                }
            }
            EXIT_PROCESS_DEBUG_EVENT => {
                break;
            }
            _ => (),
        }

        unsafe {
            if !ContinueDebugEvent(
                debug_event.dwProcessId,
                debug_event.dwThreadId,
                DBG_EXCEPTION_NOT_HANDLED,
            )
            .as_bool()
            {
                bail!("ContinueDebugEvent failed: {:?}", WinError::from_win32())
            }
        }
    }
    Ok(())
}

fn debugger_thread(pid: u32, filter: filter::Filter) -> anyhow::Result<()> {
    let handle = unsafe {
        OpenProcess(PROCESS_ALL_ACCESS, false, pid).context("OpenProcess(PROCESS_ALL_ACCESS,..)")?
    };
    let handle = ManagedHandle::new(handle);
    processes::attach_debugger(pid)?;
    print_debug_events(unsafe { handle.inner() }, filter)?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let chatterino_pid = match args.pid {
        Some(pid) => pid,
        None => {
            let Some(pid) = processes::get_chatterino_pid(&args.executable)? else {
                eprintln!("Failed to find chatterino process (searched for {}).", args.executable.to_string_lossy());
                std::process::exit(1);
            };
            pid
        }
    };

    let (tx, rx) = std::sync::mpsc::channel();
    {
        let tx_ctrlc = tx.clone();
        ctrlc::set_handler(move || {
            tx_ctrlc.send(()).ok();
        })
        .unwrap();
    }

    std::thread::spawn(move || {
        if let Err(e) = debugger_thread(
            chatterino_pid,
            filter::Filter {
                mode: args.mode,
                categories: HashSet::from_iter(args.filters.into_iter().map(From::from)),
            },
        ) {
            eprintln!("Failed to debug process (pid={chatterino_pid}): {e}");
            tx.send(()).ok();
        }
    });

    rx.recv().unwrap();

    Ok(())
}

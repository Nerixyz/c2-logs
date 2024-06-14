#![warn(clippy::cargo)]
#![allow(clippy::multiple_crate_versions)] // windows-sys

mod logging;
mod managed_types;
mod printer;
mod processes;
mod qt;
mod strings;

use std::ffi::{OsStr, OsString};

use anyhow::Context;
use clap::Parser;
use managed_types::ManagedHandle;
use printer::Printer;
use windows::Win32::{
    Foundation::{DBG_EXCEPTION_NOT_HANDLED, HANDLE},
    System::{
        Diagnostics::Debug::{
            ContinueDebugEvent, WaitForDebugEventEx, DEBUG_EVENT, EXIT_PROCESS_DEBUG_EVENT,
            OUTPUT_DEBUG_STRING_EVENT,
        },
        Threading::{OpenProcess, INFINITE, PROCESS_ALL_ACCESS},
    },
};

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(
        long = "exe",
        default_value = "chatterino.exe",
        help = "Use this to specify the name of the chatterino executable."
    )]
    executable: OsString,

    #[arg(long, help = "Use this to specify a specific process-id to attach to.")]
    pid: Option<u32>,

    #[arg(short, help = "Output to a file instead")]
    output_file: Option<OsString>,

    #[arg(long, help = "Changes the output of Qt's message handler")]
    pattern: Option<String>,

    #[arg(
        help = "Qt filter rules (e.g. *.debug=true or foo.bar.debug=false) multiple rules will be joined by a newline"
    )]
    rules: Vec<String>,
}

fn print_debug_events(process_handle: HANDLE, file: Option<&OsStr>) -> anyhow::Result<()> {
    let mut printer = Printer::new(process_handle, file).context("Opening output file")?;

    loop {
        let mut debug_event: DEBUG_EVENT = unsafe { std::mem::zeroed() };
        unsafe {
            WaitForDebugEventEx(&mut debug_event, INFINITE).context("WaitForDebugEventEx")?;
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
            ContinueDebugEvent(
                debug_event.dwProcessId,
                debug_event.dwThreadId,
                DBG_EXCEPTION_NOT_HANDLED,
            )
            .context("ContiueDebugEvent")?;
        }
    }
    Ok(())
}

fn debugger_thread(pid: u32, output_file: Option<&OsStr>) -> anyhow::Result<()> {
    let handle = unsafe {
        OpenProcess(PROCESS_ALL_ACCESS, false, pid).context("OpenProcess(PROCESS_ALL_ACCESS,..)")?
    };
    let handle = ManagedHandle::new(handle);
    processes::attach_debugger(pid)?;

    log_info!("Attached to {pid}");

    print_debug_events(unsafe { handle.inner() }, output_file)?;
    Ok(())
}

fn apply_settings(pid: u32, rules: Option<&str>, pattern: Option<&str>) -> anyhow::Result<()> {
    let (v, path) = processes::qtcore_path(pid).context("finding QtCore path")?;
    log_info!("Chatterino using {v:?} QtCore loaded from {path:?}");
    qt::set_rules_and_pattern(pid, v, &path, rules, pattern).context("set_rules_and_pattern")?;
    log_info!("Applied logging rules!");
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let chatterino_pid = match args.pid {
        Some(pid) => pid,
        None => {
            let Some(pid) = processes::get_chatterino_pid(&args.executable)? else {
                eprintln!(
                    "Failed to find chatterino process (searched for {}).",
                    args.executable.to_string_lossy()
                );
                std::process::exit(1);
            };
            pid
        }
    };
    log_info!("Found chatterino PID: {chatterino_pid}");

    apply_settings(
        chatterino_pid,
        Some(args.rules)
            .filter(|r| !r.is_empty())
            .map(|r| r.join("\n"))
            .as_deref(),
        args.pattern.as_deref(),
    )?;

    let (tx, rx) = std::sync::mpsc::channel();
    {
        let tx_ctrlc = tx.clone();
        ctrlc::set_handler(move || {
            tx_ctrlc.send(()).ok();
        })
        .unwrap();
    }

    std::thread::spawn(move || {
        if let Err(e) = debugger_thread(chatterino_pid, args.output_file.as_deref()) {
            eprintln!(
                "Failed to debug process (pid={chatterino_pid}): {e} ({})",
                e.root_cause()
            );
            tx.send(()).ok();
        }
    });

    rx.recv().unwrap();

    Ok(())
}

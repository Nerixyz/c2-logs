use std::io::{StdoutLock, Write};

use anyhow::Context;
use widestring::U16Str;
use windows::Win32::{Foundation::HANDLE, System::Diagnostics::Debug::OUTPUT_DEBUG_STRING_INFO};

use crate::strings::read_string_into;

pub struct Printer<'a> {
    stdout: StdoutLock<'a>,
    buffer: Vec<u16>,
    process_handle: HANDLE,
}

impl<'a> Printer<'a> {
    pub fn new(process_handle: HANDLE) -> Self {
        Self {
            stdout: std::io::stdout().lock(),
            buffer: Vec::new(),
            process_handle,
        }
    }

    pub fn read_string(&mut self, info: OUTPUT_DEBUG_STRING_INFO) -> anyhow::Result<()> {
        unsafe {
            read_string_into(&mut self.buffer, self.process_handle, info)
                .context("read_string_into")?;
        };

        let line = U16Str::from_slice(&self.buffer);
        write!(self.stdout, "{}", line.display()).ok();

        Ok(())
    }
}

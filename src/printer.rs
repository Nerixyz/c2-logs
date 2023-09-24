use std::{
    ffi::OsStr,
    fs,
    io::{self, Write},
};

use anyhow::Context;
use widestring::U16Str;
use windows::Win32::{Foundation::HANDLE, System::Diagnostics::Debug::OUTPUT_DEBUG_STRING_INFO};

use crate::strings::read_string_into;

pub struct Printer {
    writer: Box<dyn io::Write>,
    buffer: Vec<u16>,
    process_handle: HANDLE,
}

impl Printer {
    pub fn new(process_handle: HANDLE, file: Option<&OsStr>) -> io::Result<Self> {
        Ok(Self {
            writer: match file {
                Some(f) => Box::new(
                    fs::OpenOptions::new()
                        .write(true)
                        .truncate(true)
                        .create(true)
                        .open(f)?,
                ),
                None => Box::new(std::io::stdout().lock()),
            },
            buffer: Vec::new(),
            process_handle,
        })
    }

    pub fn read_string(&mut self, info: OUTPUT_DEBUG_STRING_INFO) -> anyhow::Result<()> {
        unsafe {
            read_string_into(&mut self.buffer, self.process_handle, info)
                .context("read_string_into")?;
        };

        let line = U16Str::from_slice(&self.buffer);
        write!(self.writer, "{}", OutputDisplay { s: line }).ok();

        Ok(())
    }
}

struct OutputDisplay<'a> {
    s: &'a U16Str,
}

// Adapted from widestring::Display, but we're ignoring \0 here
impl<'a> std::fmt::Display for OutputDisplay<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;

        for c in widestring::decode_utf16_lossy(self.s.as_slice().iter().copied()) {
            if c != '\0' {
                f.write_char(c)?;
            }
        }
        Ok(())
    }
}

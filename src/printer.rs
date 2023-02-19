use std::io::{StdoutLock, Write};

use anyhow::Context;
use widestring::{U16Str, U16String};
use windows::Win32::{Foundation::HANDLE, System::Diagnostics::Debug::OUTPUT_DEBUG_STRING_INFO};

use crate::{filter, strings::read_string_into};

pub struct Printer<'a> {
    stdout: StdoutLock<'a>,
    buffer: Vec<u16>,
    last_category: U16String,
    filter: filter::Filter,
    process_handle: HANDLE,
}

impl<'a> Printer<'a> {
    pub fn new(process_handle: HANDLE, filter: filter::Filter) -> Self {
        Self {
            stdout: std::io::stdout().lock(),
            buffer: Vec::new(),
            last_category: U16String::new(),
            filter,
            process_handle,
        }
    }

    pub fn read_string(&mut self, info: OUTPUT_DEBUG_STRING_INFO) -> anyhow::Result<()> {
        unsafe {
            read_string_into(&mut self.buffer, self.process_handle, info)
                .context("read_string_into")?;
        };

        let line = U16Str::from_slice(&self.buffer);

        if let Some(category) = filter::get_category(line) {
            category.clone_into(&mut self.last_category);
        }

        if self.filter.should_print(&self.last_category) {
            write!(self.stdout, "{}", line.display()).ok();
        }

        Ok(())
    }
}

use windows::{
    core::{Error as WinError, Result},
    Win32::{
        Foundation::HANDLE,
        System::Diagnostics::Debug::{ReadProcessMemory, OUTPUT_DEBUG_STRING_INFO},
    },
};

pub unsafe fn read_string_into(
    buffer: &mut Vec<u16>,
    handle: HANDLE,
    info: OUTPUT_DEBUG_STRING_INFO,
) -> Result<()> {
    let length = info.nDebugStringLength as usize;
    buffer.resize(length / 2, 0);
    let mut n_read = 0;
    if !ReadProcessMemory(
        handle,
        info.lpDebugStringData.0.cast_const().cast(),
        buffer.as_mut_ptr().cast(),
        length,
        Some(&mut n_read),
    )
    .as_bool()
    {
        return Err(WinError::from_win32());
    }
    Ok(())
}

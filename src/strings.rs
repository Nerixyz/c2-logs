use windows::{
    core::{Error as WinError, Result},
    Win32::{
        Foundation::HANDLE,
        System::Diagnostics::Debug::{ReadProcessMemory, OUTPUT_DEBUG_STRING_INFO},
    },
};

pub fn is_wide_string(s: &[u8]) -> bool {
    s.iter().skip(1).step_by(2).take(10).all(|x| *x == 0)
}

pub unsafe fn read_string_into(
    buffer: &mut Vec<u8>,
    handle: HANDLE,
    info: OUTPUT_DEBUG_STRING_INFO,
) -> Result<()> {
    let length = info.nDebugStringLength as usize;
    buffer.resize(length, 0);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wide() {
        for s in [
            &[
                120u8, 0, 23, 0, 53, 0, 31, 0, 64, 0, 62, 0, 24, 0, 74, 0, 54, 0, 67, 0, 24, 0, 0,
                0,
            ][..],
            &[
                120u8, 0, 23, 0, 53, 0, 31, 0, 64, 0, 62, 0, 24, 0, 74, 0, 54, 0, 63, 0, 24, 0, 14,
                0, 67, 0, 24, 0, 0, 0,
            ],
            &[
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 74, 0, 54, 0, 67, 0, 24, 0, 0, 0,
            ],
            &[0, 0],
        ] {
            assert!(is_wide_string(s), "{s:?}")
        }

        for s in [
            &[120u8, 23, 53, 31, 64, 62, 24, 74, 54, 67, 24, 0][..],
            &[
                120u8, 1, 23, 5, 53, 8, 31, 0, 64, 0, 62, 0, 24, 0, 74, 0, 54, 63, 0, 24, 0, 14, 0,
                0, 67, 0, 24, 0, 0, 0,
            ],
            &[1, 2, 0],
        ] {
            assert!(!is_wide_string(s), "{s:?}")
        }
    }
}

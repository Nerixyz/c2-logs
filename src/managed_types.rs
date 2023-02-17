use windows::Win32::Foundation::{CloseHandle, HANDLE};

#[repr(transparent)]
#[derive(Debug)]
pub struct ManagedHandle(HANDLE);

impl ManagedHandle {
    /// Safety: The caller must guarantee that the returned handle is shorter lived that this struct.
    pub unsafe fn inner(&self) -> HANDLE {
        self.0
    }

    pub fn new(h: HANDLE) -> Self {
        Self(h)
    }
}

impl Drop for ManagedHandle {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}

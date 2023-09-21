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

impl std::ops::Deref for ManagedHandle {
    type Target = HANDLE;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for ManagedHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

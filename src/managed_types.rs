use windows::Win32::Foundation::{CloseHandle, FreeLibrary, HANDLE, HMODULE};

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

#[repr(transparent)]
#[derive(Debug)]
pub struct ManagedModule(HMODULE);

impl ManagedModule {
    /// Safety: The caller must guarantee that the returned module is shorter lived that this struct.
    pub unsafe fn inner(&self) -> HMODULE {
        self.0
    }

    pub fn new(h: HMODULE) -> Self {
        Self(h)
    }
}

impl std::ops::Deref for ManagedModule {
    type Target = HMODULE;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for ManagedModule {
    fn drop(&mut self) {
        unsafe {
            let _ = FreeLibrary(self.0);
        }
    }
}

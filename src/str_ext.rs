use widestring::{U16CStr, U16Str};

pub trait WStrExt<T: ?Sized> {
    fn starts_with(&self, needle: &T) -> bool;
}

impl WStrExt<U16CStr> for U16CStr {
    fn starts_with(&self, needle: &U16CStr) -> bool {
        self.as_slice().starts_with(needle.as_slice())
    }
}

impl WStrExt<U16Str> for U16CStr {
    fn starts_with(&self, needle: &U16Str) -> bool {
        self.as_slice().starts_with(needle.as_slice())
    }
}

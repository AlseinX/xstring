/// Represents str-like types that could be cast to valid byte patterns and back.
///
/// # Safety
///
/// It is the implementor's responsibility to ensure that the bytes returned by `as_bytes` could be safely passed to `from_bytes_unchecked`.
pub unsafe trait Str {
    /// Casts the given bytes to a reference to `Self`.
    ///
    /// # Safety
    ///
    /// The bytes must be valid byte patterns for `Self`.
    unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Self;

    /// Casts the given reference to `Self` to a slice of bytes.
    fn as_bytes(&self) -> &[u8];

    /// Indicates whether the bytes returned by `as_bytes` contain a NUL byte.
    const CONTAINS_NUL: bool;
}

unsafe impl Str for str {
    #[inline(always)]
    unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Self {
        unsafe { core::str::from_utf8_unchecked(bytes) }
    }

    #[inline(always)]
    fn as_bytes(&self) -> &[u8] {
        self.as_bytes()
    }

    const CONTAINS_NUL: bool = false;
}

unsafe impl Str for [u8] {
    #[inline(always)]
    unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Self {
        bytes
    }

    #[inline(always)]
    fn as_bytes(&self) -> &[u8] {
        self
    }

    const CONTAINS_NUL: bool = false;
}

unsafe impl Str for core::ffi::CStr {
    #[inline(always)]
    unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Self {
        unsafe { core::ffi::CStr::from_bytes_with_nul_unchecked(bytes) }
    }

    #[inline(always)]
    fn as_bytes(&self) -> &[u8] {
        self.to_bytes_with_nul()
    }

    const CONTAINS_NUL: bool = true;
}

#[cfg(feature = "std")]
unsafe impl Str for std::ffi::OsStr {
    #[inline(always)]
    unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Self {
        unsafe { std::ffi::OsStr::from_encoded_bytes_unchecked(bytes) }
    }

    #[inline(always)]
    fn as_bytes(&self) -> &[u8] {
        self.as_encoded_bytes()
    }

    const CONTAINS_NUL: bool = false;
}

#[cfg(feature = "std")]
unsafe impl Str for std::path::Path {
    #[inline(always)]
    unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Self {
        unsafe { Self::new(std::ffi::OsStr::from_bytes_unchecked(bytes)) }
    }

    #[inline(always)]
    fn as_bytes(&self) -> &[u8] {
        self.as_os_str().as_encoded_bytes()
    }

    const CONTAINS_NUL: bool = false;
}

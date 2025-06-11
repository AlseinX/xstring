use alloc::{borrow::Cow, ffi::CString, sync::Arc};
use core::{convert::Infallible, ffi::CStr, str::FromStr};

#[cfg(feature = "std")]
use crate::Allocator;
use crate::{Str, XString};

impl FromStr for XString {
    type Err = Infallible;
    #[inline(always)]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s))
    }
}

impl<S: Str + ?Sized> From<&S> for XString<S> {
    #[inline(always)]
    fn from(value: &S) -> Self {
        Self::new(value)
    }
}

impl<S: Str + ?Sized> From<&mut S> for XString<S> {
    #[inline(always)]
    fn from(value: &mut S) -> Self {
        Self::new(value)
    }
}

macro_rules! define_ref {
    ($s:ty, ($($t:ty),*$(,)?)) => {$(
        impl From<$t> for XString<$s> {
            #[inline(always)]
            fn from(value: $t) -> Self {
                Self::new(&value)
            }
        }
    )*};
}

define_ref!(
    str,
    (
        alloc::string::String,
        alloc::boxed::Box<str>,
        Arc<str>,
        Cow<'_, str>
    )
);

define_ref!(
    CStr,
    (CString, alloc::boxed::Box<CStr>, Arc<CStr>, Cow<'_, CStr>)
);

#[cfg(feature = "std")]
define_ref!(
    std::ffi::OsStr,
    (
        std::ffi::OsString,
        alloc::boxed::Box<std::ffi::OsStr>,
        Arc<std::ffi::OsStr>,
        Cow<'_, std::ffi::OsStr>
    )
);

#[cfg(feature = "std")]
define_ref!(
    std::path::Path,
    (
        std::path::PathBuf,
        alloc::boxed::Box<std::path::Path>,
        Arc<std::path::Path>,
        Cow<'_, std::path::Path>
    )
);

#[cfg(feature = "std")]
impl<S: Str + ?Sized + std::net::ToSocketAddrs, A: Allocator> std::net::ToSocketAddrs
    for XString<S, A>
{
    type Iter = <S as std::net::ToSocketAddrs>::Iter;

    #[inline(always)]
    fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
        <S as std::net::ToSocketAddrs>::to_socket_addrs(self)
    }
}

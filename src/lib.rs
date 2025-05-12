#![cfg_attr(not(any(test, feature = "std")), no_std)]
#![cfg_attr(feature = "allocator_api", feature(allocator_api))]

extern crate alloc;

mod allocator;
#[cfg(feature = "serde")]
mod serde;
mod str;
mod xstring;
pub use {allocator::*, str::*, xstring::*};

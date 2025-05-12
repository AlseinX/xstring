use core::{
    alloc::Layout,
    borrow::Borrow,
    cell::UnsafeCell,
    ffi::CStr,
    fmt::{Debug, Display},
    hash::Hash,
    marker::PhantomData,
    mem::MaybeUninit,
    ops::{Deref, Index},
    slice,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::{
    allocator::{Allocator, Global},
    str::Str,
};

#[repr(transparent)]
pub struct XString<S: Str + ?Sized = str, A: Allocator = Global> {
    repr: Repr<A>,
    _marker: PhantomData<S>,
}

unsafe impl<S: Str + ?Sized + Send, A: Allocator> Send for XString<S, A> {}
unsafe impl<S: Str + ?Sized + Sync, A: Allocator> Sync for XString<S, A> {}
impl<S: Str + ?Sized + Unpin, A: Allocator> Unpin for XString<S, A> {}

impl<S: Str + ?Sized> XString<S> {
    #[inline(always)]
    pub fn new(source: &S) -> Self {
        Self::new_in(source, Global)
    }

    #[inline]
    pub fn from_static(source: &'static S) -> Self {
        let source = source.as_bytes();
        let repr = unsafe {
            Repr {
                r#static: Static::new(source),
            }
        };
        Self {
            repr,
            _marker: PhantomData,
        }
    }
}

impl<S: Str + ?Sized, A: Allocator> XString<S, A> {
    #[inline]
    pub fn new_in(source: &S, allocator: A) -> Self {
        let source = source.as_bytes();
        let repr = unsafe {
            if source.len() <= INLINE_SIZE {
                Repr {
                    inline: Inlined::new(source),
                }
            } else {
                Repr {
                    alloc: Allocated::new(source, allocator),
                }
            }
        };
        Self {
            repr,
            _marker: PhantomData,
        }
    }

    /// # Safety
    ///
    /// It is the caller's responsibility to ensure that the `dst` is derived from `self`
    #[inline]
    pub unsafe fn slice_as(&self, dst: &S) -> Self {
        let src = self.deref();

        let repr = if self.repr.is_inlined() {
            Repr {
                inline: unsafe { Inlined::new(dst.as_bytes()) },
            }
        } else if self.repr.is_static() {
            Repr {
                r#static: unsafe { Static::new(dst.as_bytes()) },
            }
        } else {
            let src = src.as_bytes().as_ptr_range();
            let dst = dst.as_bytes().as_ptr_range();
            let start_offset = unsafe { dst.start.offset_from(src.start) } as i32;
            let end_offset = unsafe { dst.end.offset_from(src.end) } as i32;
            debug_assert!(start_offset >= 0);
            debug_assert!(end_offset <= 0);

            let mut target = self.clone();

            unsafe {
                target.repr.alloc.start = self.repr.alloc.start + start_offset;
                target.repr.alloc.end = self.repr.alloc.end + end_offset;
            }

            return target;
        };

        Self {
            repr,
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    pub fn slice<T>(&self, idx: T) -> Self
    where
        S: Index<T, Output = S>,
    {
        unsafe { self.slice_as(&self[idx]) }
    }
}

impl<A: Allocator> XString<str, A> {
    #[inline(always)]
    pub fn as_str(&self) -> &str {
        self
    }
}

impl<A: Allocator> XString<CStr, A> {
    #[inline(always)]
    pub fn as_c_str(&self) -> &CStr {
        self
    }
}

impl<A: Allocator> XString<[u8], A> {
    #[inline(always)]
    pub fn as_bytes(&self) -> &[u8] {
        self
    }
}

macro_rules! impl_for_refs {
    ($s:ident, $($t:ty),* $(,)?) => {$(
        impl<$s: Str + ?Sized + PartialEq, A: Allocator> PartialEq<$t> for XString<$s, A> {
            #[inline(always)]
            fn eq(&self, other: &$t) -> bool {
                $s::eq(self, other)
            }
        }

        impl<$s: Str + ?Sized + PartialOrd, A: Allocator> PartialOrd<$t> for XString<$s, A> {
            #[inline(always)]
            fn partial_cmp(&self, other: &$t) -> Option<core::cmp::Ordering> {
                $s::partial_cmp(self, other)
            }
        }
    )*};
}

impl_for_refs!(
    S,
    Self,
    S,
    &S,
    ::alloc::boxed::Box<S>,
    ::alloc::rc::Rc<S>,
    ::alloc::sync::Arc<S>,
);

impl<S: Str + ?Sized + Eq, A: Allocator> Eq for XString<S, A> {}

impl<S: Str + ?Sized + Ord, A: Allocator> Ord for XString<S, A> {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        S::cmp(self, other)
    }
}

impl<S: Str + ?Sized + Hash, A: Allocator> Hash for XString<S, A> {
    #[inline(always)]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        S::hash(self, state)
    }
}

impl<S: Str + ?Sized + Debug, A: Allocator> Debug for XString<S, A> {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&**self, f)
    }
}

impl<S: Str + ?Sized + Display, A: Allocator> Display for XString<S, A> {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&**self, f)
    }
}

impl<S: Str + ?Sized, A: Allocator> Deref for XString<S, A> {
    type Target = S;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe {
            let bytes = if self.repr.is_inlined() {
                if let Ok(s) = CStr::from_bytes_until_nul(&self.repr.inline.data) {
                    if S::CONTAINS_NUL {
                        s.to_bytes_with_nul()
                    } else {
                        s.to_bytes()
                    }
                } else {
                    &self.repr.inline.data
                }
            } else if self.repr.is_static() {
                slice::from_raw_parts(self.repr.r#static.data, (-self.repr.r#static.len) as _)
            } else {
                slice::from_raw_parts(
                    (self.repr.alloc.ptr.add(1) as *const u8).add(self.repr.alloc.start as _),
                    (self.repr.alloc.end - self.repr.alloc.start) as _,
                )
            };
            S::from_bytes_unchecked(bytes)
        }
    }
}

impl<S: Str + ?Sized, A: Allocator> Drop for XString<S, A> {
    fn drop(&mut self) {
        if self.repr.is_inlined() || self.repr.is_static() {
            return;
        }

        unsafe {
            if (*self.repr.alloc.ptr).rc.fetch_sub(1, Ordering::Relaxed) > 1 {
                return;
            }

            let (layout, _) = Layout::new::<AllocatedRepr<A>>()
                .extend(Layout::array::<u8>((*self.repr.alloc.ptr).size).unwrap())
                .unwrap();
            let mut allocator = core::mem::replace(
                &mut *(*self.repr.alloc.ptr).allocator.get(),
                MaybeUninit::uninit(),
            )
            .assume_init();

            allocator.deallocate(self.repr.alloc.ptr as _, layout);
        }
    }
}

impl<S: Str + ?Sized, A: Allocator> Clone for XString<S, A> {
    fn clone(&self) -> Self {
        if !(self.repr.is_inlined() || self.repr.is_static()) {
            unsafe { (*self.repr.alloc.ptr).rc.fetch_add(1, Ordering::Relaxed) };
        }

        Self {
            repr: self.repr,
            _marker: PhantomData,
        }
    }
}

impl<S: Str + ?Sized, A: Allocator> AsRef<S> for XString<S, A> {
    #[inline(always)]
    fn as_ref(&self) -> &S {
        self
    }
}

impl<S: Str + ?Sized, A: Allocator> Borrow<S> for XString<S, A> {
    #[inline(always)]
    fn borrow(&self) -> &S {
        self
    }
}

impl<A: Allocator> AsRef<[u8]> for XString<str, A> {
    #[inline(always)]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<A: Allocator> AsRef<[u8]> for XString<CStr, A> {
    #[inline(always)]
    fn as_ref(&self) -> &[u8] {
        self.to_bytes()
    }
}

#[cfg(feature = "std")]
impl<A: Allocator> AsRef<[u8]> for XString<std::ffi::OsStr, A> {
    #[inline(always)]
    fn as_ref(&self) -> &[u8] {
        self.as_encoded_bytes()
    }
}

#[cfg(feature = "std")]
impl<A: Allocator> AsRef<[u8]> for XString<std::path::Path, A> {
    #[inline(always)]
    fn as_ref(&self) -> &[u8] {
        self.as_os_str().as_encoded_bytes()
    }
}

#[cfg(target_pointer_width = "64")]
const POINTER_SIZE: usize = 8;

#[cfg(target_pointer_width = "32")]
const POINTER_SIZE: usize = 4;

#[cfg(target_pointer_width = "16")]
compile_error!("16 bit platforms are not supported");

const INLINE_SIZE: usize = POINTER_SIZE + 7;

#[repr(C)]
union Repr<A: Allocator> {
    inline: Inlined,
    r#static: Static,
    alloc: Allocated<A>,
}

impl<A: Allocator> Clone for Repr<A> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<A: Allocator> Copy for Repr<A> {}

#[cfg(target_endian = "little")]
#[repr(C)]
struct Allocated<A: Allocator> {
    ptr: *mut AllocatedRepr<A>,
    start: i32,
    end: i32,
}

#[cfg(target_endian = "big")]
#[repr(C)]
struct Allocated<A: Allocator> {
    end: i32,
    start: i32,
    ptr: *const AllocatedRepr<A>,
}

impl<A: Allocator> Copy for Allocated<A> {}
impl<A: Allocator> Clone for Allocated<A> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

#[cfg(target_endian = "little")]
#[derive(Clone, Copy)]
#[repr(C)]
struct Inlined {
    data: [u8; INLINE_SIZE], // null-terminated if not full filled
    indicator: i8,           // inlined while negative
}

#[cfg(target_endian = "big")]
#[derive(Clone, Copy)]
#[repr(C)]
struct Inlined {
    indicator: i8,           // inlined while negative
    data: [u8; INLINE_SIZE], // null-terminated if not full filled
}

#[cfg(target_endian = "little")]
#[derive(Clone, Copy)]
#[repr(C)]
struct Static {
    data: *const u8,
    len: i32, // static while negative
    _padding: i32,
}

#[cfg(target_endian = "big")]
#[derive(Clone, Copy)]
#[repr(C)]
struct Static {
    _padding: i32,
    len: i32, // static while negative
    data: *const u8,
}

struct AllocatedRepr<A: Allocator> {
    size: usize,
    rc: AtomicUsize,
    allocator: UnsafeCell<MaybeUninit<A>>,
}

impl<A: Allocator> Allocated<A> {
    unsafe fn new(bytes: &[u8], mut allocator: A) -> Self {
        let (layout, offset) = Layout::new::<AllocatedRepr<A>>()
            .extend(Layout::array::<u8>(bytes.len()).unwrap())
            .unwrap();
        let ptr = unsafe {
            let ptr = allocator.allocate(layout) as *mut AllocatedRepr<A>;
            ptr.write(AllocatedRepr {
                size: bytes.len(),
                rc: AtomicUsize::new(1),
                allocator: UnsafeCell::new(MaybeUninit::new(allocator)),
            });
            bytes
                .as_ptr()
                .copy_to_nonoverlapping((ptr as *mut u8).add(offset), bytes.len());
            ptr as _
        };
        Self {
            ptr,
            start: 0,
            end: bytes.len() as _,
        }
    }
}

impl Static {
    unsafe fn new(bytes: &[u8]) -> Self {
        let len = -(bytes.len() as i32);
        let data = bytes.as_ptr();
        Self {
            data,
            len,
            _padding: 0,
        }
    }
}

impl Inlined {
    unsafe fn new(bytes: &[u8]) -> Self {
        let mut this = Self {
            data: [0; INLINE_SIZE],
            indicator: -1,
        };
        unsafe {
            this.data
                .as_mut_ptr()
                .copy_from_nonoverlapping(bytes.as_ptr(), bytes.len())
        };
        this
    }
}

impl<A: Allocator> Repr<A> {
    fn is_inlined(&self) -> bool {
        unsafe { self.inline.indicator < 0 }
    }

    fn is_static(&self) -> bool {
        unsafe { self.r#static.len < 0 }
    }
}

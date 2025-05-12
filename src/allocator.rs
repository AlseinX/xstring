use core::alloc::Layout;

pub trait Allocator {
    /// # Safety
    /// See [`::alloc::global::GlobalAlloc::alloc`].
    unsafe fn allocate(&mut self, layout: Layout) -> *mut u8;

    /// # Safety
    /// See [`::alloc::global::GlobalAlloc::dealloc`].
    unsafe fn deallocate(&mut self, ptr: *mut u8, layout: Layout);
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Global;

impl Allocator for Global {
    unsafe fn allocate(&mut self, layout: Layout) -> *mut u8 {
        unsafe { ::alloc::alloc::alloc(layout) }
    }

    unsafe fn deallocate(&mut self, ptr: *mut u8, layout: Layout) {
        unsafe { ::alloc::alloc::dealloc(ptr, layout) }
    }
}

#[cfg(feature = "allocator_api")]
impl<T> Allocator for T
where
    T: core::alloc::Allocator,
{
    unsafe fn allocate(&mut self, layout: Layout) -> *mut u8 {
        if let Ok(ptr) = core::alloc::Allocator::allocate(self, layout) {
            unsafe { ptr.cast().as_mut() }
        } else {
            core::ptr::null_mut()
        }
    }

    unsafe fn deallocate(&mut self, ptr: *mut u8, layout: Layout) {
        unsafe {
            core::alloc::Allocator::deallocate(
                self,
                core::ptr::NonNull::new(ptr).expect("deallocate null pointer"),
                layout,
            )
        };
    }
}

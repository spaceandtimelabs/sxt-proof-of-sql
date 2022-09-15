/**
 * Adopted from arrow
 *
 * See third_party/license/arrow.LICENSE
 */
use std::ptr::NonNull;

/// This struct is highly `unsafe` and offers the possibility to
/// self-reference a [crate::buffer::Buffer] from
/// [crate::array::ArrayData], as a pointer to the beginning of its
/// contents.
pub(super) struct RawPtrBox<T> {
    ptr: NonNull<T>,
}

impl<T> RawPtrBox<T> {
    /// # Safety
    /// The user must guarantee that:
    /// * the contents where `ptr` points to are never `moved`. This is guaranteed when they are Pinned.
    /// * the lifetime of this struct does not outlive the lifetime of `ptr`.
    /// Failure to fulfill any the above conditions results in undefined behavior.
    /// # Panic
    /// This function panics if:
    /// * `ptr` is null
    /// * `ptr` is not aligned to a slice of type `T`. This is guaranteed if it was built from a slice of type `T`.
    pub(super) unsafe fn new(ptr: *const u8) -> Self {
        let ptr = NonNull::new(ptr as *mut u8).expect("Pointer cannot be null");
        assert_eq!(
            ptr.as_ptr().align_offset(std::mem::align_of::<T>()),
            0,
            "memory is not aligned"
        );
        Self { ptr: ptr.cast() }
    }

    pub(super) fn as_ptr(&self) -> *const T {
        self.ptr.as_ptr()
    }
}

unsafe impl<T> Send for RawPtrBox<T> {}
unsafe impl<T> Sync for RawPtrBox<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "memory is not aligned")]
    #[cfg_attr(miri, ignore)] // sometimes does not panic as expected
    fn test_primitive_array_alignment() {
        let bytes = vec![0u8, 1u8];
        unsafe { RawPtrBox::<u64>::new(bytes.as_ptr().offset(1)) };
    }
}

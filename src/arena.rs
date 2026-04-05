use std::alloc::Layout;
use std::ptr;

use crate::ffi;

/// A memory arena allocator that provides fast bump allocation.
///
/// All allocations from the arena are freed at once when the arena is destroyed or reset.
/// Individual allocations cannot be freed.
pub struct Arena {
    inner: *mut ffi::Arena,
    owned: bool,
}

impl Arena {
    /// Creates a new arena with the given initial reservation size.
    pub fn new(reserve_size: usize) -> Option<Self> {
        let ptr = unsafe { ffi::arena_create(reserve_size) };
        if ptr.is_null() {
            None
        } else {
            Some(Arena {
                inner: ptr,
                owned: true,
            })
        }
    }

    /// Allocates `size` bytes from the arena with the given alignment.
    fn alloc_aligned_inner(&self, size: usize, alignment: usize) -> *mut u8 {
        if size == 0 {
            return std::ptr::null_mut();
        }
        unsafe {
            let arena = &mut *self.inner;
            let aligned = ((arena.curr as usize) + alignment - 1) & !(alignment - 1);
            let next = aligned + size;

            if next <= arena.end as usize {
                arena.curr = next as *mut i8;
                return aligned as *mut u8;
            }

            ffi::_arena_alloc_slow(self.inner, size, alignment) as *mut u8
        }
    }

    /// Allocates `size` bytes from the arena with default alignment (16).
    pub fn alloc(&self, size: usize) -> Option<*mut u8> {
        let ptr = self.alloc_aligned_inner(size, 16);
        if ptr.is_null() { None } else { Some(ptr) }
    }

    /// Allocates memory for a value of type T and returns a mutable reference.
    pub fn alloc_value<T>(&self, value: T) -> Option<&mut T> {
        let layout = Layout::new::<T>();
        let ptr = self.alloc_aligned_inner(layout.size(), layout.align());
        if ptr.is_null() {
            return None;
        }
        let typed_ptr = ptr as *mut T;
        unsafe {
            typed_ptr.write(value);
            Some(&mut *typed_ptr)
        }
    }

    /// Allocates a slice of `count` elements of type T, initialized to zero.
    pub fn alloc_slice<T: Default + Copy>(&self, count: usize) -> Option<&mut [T]> {
        let layout = Layout::new::<T>();
        let total_size = layout.size() * count;
        let ptr = self.alloc_aligned_inner(total_size, layout.align());
        if ptr.is_null() {
            return None;
        }
        unsafe {
            ptr::write_bytes(ptr, 0, total_size);
            Some(std::slice::from_raw_parts_mut(ptr as *mut T, count))
        }
    }

    /// Allocates memory with specific alignment.
    pub fn alloc_aligned(&self, size: usize, alignment: usize) -> Option<*mut u8> {
        let ptr = self.alloc_aligned_inner(size, alignment);
        if ptr.is_null() { None } else { Some(ptr) }
    }

    /// Resets the arena, invalidating all previous allocations.
    pub fn reset(&mut self) {
        unsafe {
            let arena = &mut *self.inner;
            let block = &*arena.head;
            arena.curr = block.base;
            arena.end = block.end;
            arena.current_block = arena.head;
        }
    }

    /// Returns the total committed memory size.
    pub fn committed_size(&self) -> usize {
        unsafe { (*self.inner).total_committed }
    }

    /// Returns the amount of memory currently in use.
    pub fn used_size(&self) -> usize {
        unsafe {
            let arena = &*self.inner;
            let base = (*arena.head).base;
            arena.curr as usize - base as usize
        }
    }

    /// Returns a raw pointer to the inner Arena.
    /// Used for interop with other solidc functions that need an Arena*.
    pub(crate) fn as_ptr(&self) -> *mut ffi::Arena {
        self.inner
    }
}

impl Drop for Arena {
    fn drop(&mut self) {
        if self.owned && !self.inner.is_null() {
            unsafe { ffi::arena_destroy(self.inner) };
        }
    }
}

unsafe impl Send for Arena {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_create() {
        let arena = Arena::new(4096).unwrap();
        assert_eq!(arena.used_size(), 0);
        assert!(arena.committed_size() > 0);
    }

    #[test]
    fn test_arena_alloc() {
        let arena = Arena::new(4096).unwrap();
        let ptr = arena.alloc(64).unwrap();
        assert!(!ptr.is_null());
        assert!(arena.used_size() >= 64);
    }

    #[test]
    fn test_arena_alloc_value() {
        let arena = Arena::new(4096).unwrap();
        let val = arena.alloc_value(42i32).unwrap();
        assert_eq!(*val, 42);
        *val = 100;
        assert_eq!(*val, 100);
    }

    #[test]
    fn test_arena_alloc_slice() {
        let arena = Arena::new(4096).unwrap();
        let slice = arena.alloc_slice::<i32>(10).unwrap();
        assert_eq!(slice.len(), 10);
        for (i, item) in slice.iter_mut().enumerate() {
            *item = i as i32;
        }
        assert_eq!(slice[5], 5);
    }

    #[test]
    fn test_arena_reset() {
        let mut arena = Arena::new(4096).unwrap();
        arena.alloc(1024).unwrap();
        let used_before = arena.used_size();
        assert!(used_before >= 1024);

        arena.reset();
        assert_eq!(arena.used_size(), 0);
    }

    #[test]
    fn test_arena_multiple_allocs() {
        let arena = Arena::new(4096).unwrap();
        for _ in 0..100 {
            arena.alloc(32).unwrap();
        }
        assert!(arena.used_size() >= 3200);
    }
}

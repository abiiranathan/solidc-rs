use std::marker::PhantomData;
use std::mem;

use crate::ffi;

/// A type-safe dynamic array backed by solidc's dynarray.
pub struct DynArray<T: Copy> {
    inner: ffi::dynarray_t,
    _phantom: PhantomData<T>,
}

impl<T: Copy> DynArray<T> {
    /// Creates a new dynamic array.
    pub fn new() -> Option<Self> {
        let mut arr: ffi::dynarray_t = unsafe { mem::zeroed() };
        let ok = unsafe { ffi::dynarray_init(&mut arr, mem::size_of::<T>(), 0) };
        if ok {
            Some(DynArray { inner: arr, _phantom: PhantomData })
        } else {
            None
        }
    }

    /// Creates a new dynamic array with the given initial capacity.
    pub fn with_capacity(capacity: usize) -> Option<Self> {
        let mut arr: ffi::dynarray_t = unsafe { mem::zeroed() };
        let ok = unsafe { ffi::dynarray_init(&mut arr, mem::size_of::<T>(), capacity) };
        if ok {
            Some(DynArray { inner: arr, _phantom: PhantomData })
        } else {
            None
        }
    }

    /// Appends an element to the end.
    pub fn push(&mut self, value: T) -> bool {
        unsafe { ffi::dynarray_push(&mut self.inner, &value as *const T as *const _) }
    }

    /// Removes and returns the last element.
    pub fn pop(&mut self) -> Option<T> {
        let mut out: T = unsafe { mem::zeroed() };
        let ok = unsafe { ffi::dynarray_pop(&mut self.inner, &mut out as *mut T as *mut _) };
        if ok { Some(out) } else { None }
    }

    /// Gets the element at the given index.
    pub fn get(&self, index: usize) -> Option<T> {
        let ptr = unsafe { ffi::dynarray_get(&self.inner, index) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { *(ptr as *const T) })
        }
    }

    /// Sets the element at the given index.
    pub fn set(&mut self, index: usize, value: T) -> bool {
        unsafe { ffi::dynarray_set(&mut self.inner, index, &value as *const T as *const _) }
    }

    /// Returns the number of elements.
    pub fn len(&self) -> usize {
        self.inner.size
    }

    /// Returns true if the array is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.size == 0
    }

    /// Returns the current capacity.
    pub fn capacity(&self) -> usize {
        self.inner.capacity
    }

    /// Reserves capacity for at least `additional` more elements.
    pub fn reserve(&mut self, new_capacity: usize) -> bool {
        unsafe { ffi::dynarray_reserve(&mut self.inner, new_capacity) }
    }

    /// Shrinks the capacity to match the current length.
    pub fn shrink_to_fit(&mut self) -> bool {
        unsafe { ffi::dynarray_shrink_to_fit(&mut self.inner) }
    }

    /// Clears all elements without freeing the buffer.
    pub fn clear(&mut self) {
        unsafe { ffi::dynarray_clear(&mut self.inner) };
    }

    /// Returns an iterator over the elements.
    pub fn iter(&self) -> DynArrayIter<'_, T> {
        DynArrayIter { arr: self, index: 0 }
    }
}

impl<T: Copy> Drop for DynArray<T> {
    fn drop(&mut self) {
        unsafe { ffi::dynarray_free(&mut self.inner) };
    }
}

impl<T: Copy> Default for DynArray<T> {
    fn default() -> Self {
        Self::new().expect("Failed to create DynArray")
    }
}

pub struct DynArrayIter<'a, T: Copy> {
    arr: &'a DynArray<T>,
    index: usize,
}

impl<'a, T: Copy> Iterator for DynArrayIter<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.arr.len() {
            let val = self.arr.get(self.index);
            self.index += 1;
            val
        } else {
            None
        }
    }
}

unsafe impl<T: Copy + Send> Send for DynArray<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_pop() {
        let mut arr = DynArray::<i32>::new().unwrap();
        arr.push(1);
        arr.push(2);
        arr.push(3);
        assert_eq!(arr.len(), 3);
        assert_eq!(arr.pop(), Some(3));
        assert_eq!(arr.pop(), Some(2));
        assert_eq!(arr.len(), 1);
    }

    #[test]
    fn test_get_set() {
        let mut arr = DynArray::<i32>::new().unwrap();
        arr.push(10);
        arr.push(20);
        assert_eq!(arr.get(0), Some(10));
        assert_eq!(arr.get(1), Some(20));
        assert_eq!(arr.get(5), None);
        assert!(arr.set(0, 99));
        assert_eq!(arr.get(0), Some(99));
    }

    #[test]
    fn test_clear() {
        let mut arr = DynArray::<i32>::new().unwrap();
        arr.push(1);
        arr.push(2);
        arr.clear();
        assert!(arr.is_empty());
        assert_eq!(arr.len(), 0);
    }

    #[test]
    fn test_iterator() {
        let mut arr = DynArray::<i32>::new().unwrap();
        for i in 0..5 {
            arr.push(i);
        }
        let collected: Vec<i32> = arr.iter().collect();
        assert_eq!(collected, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_with_capacity() {
        let arr = DynArray::<f64>::with_capacity(100).unwrap();
        assert!(arr.capacity() >= 100);
        assert!(arr.is_empty());
    }
}

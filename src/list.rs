use std::marker::PhantomData;
use std::mem;

use crate::ffi;

/// A type-safe doubly linked list backed by solidc's list_t.
pub struct List<T: Copy> {
    inner: *mut ffi::list_t,
    _phantom: PhantomData<T>,
}

impl<T: Copy> List<T> {
    /// Creates a new empty list.
    pub fn new() -> Option<Self> {
        let ptr = unsafe { ffi::list_new(mem::size_of::<T>()) };
        if ptr.is_null() {
            None
        } else {
            Some(List { inner: ptr, _phantom: PhantomData })
        }
    }

    /// Returns the number of elements.
    pub fn len(&self) -> usize {
        unsafe { ffi::list_size(self.inner) }
    }

    /// Returns true if the list is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Appends an element to the back.
    pub fn push_back(&mut self, value: T) {
        unsafe { ffi::list_push_back(self.inner, &value as *const T as *mut _) };
    }

    /// Appends an element to the front.
    pub fn push_front(&mut self, value: T) {
        unsafe { ffi::list_push_front(self.inner, &value as *const T as *mut _) };
    }

    /// Removes the last element.
    pub fn pop_back(&mut self) {
        unsafe { ffi::list_pop_back(self.inner) };
    }

    /// Removes the first element.
    pub fn pop_front(&mut self) {
        unsafe { ffi::list_pop_front(self.inner) };
    }

    /// Gets the element at the given index.
    pub fn get(&self, index: usize) -> Option<T> {
        let ptr = unsafe { ffi::list_get(self.inner, index) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { *(ptr as *const T) })
        }
    }

    /// Inserts an element at the given index.
    pub fn insert(&mut self, index: usize, value: T) {
        unsafe { ffi::list_insert(self.inner, index, &value as *const T as *mut _) };
    }

    /// Clears all elements.
    pub fn clear(&mut self) {
        unsafe { ffi::list_clear(self.inner) };
    }

    /// Collects all elements into a Vec.
    pub fn to_vec(&self) -> Vec<T> {
        let mut result = Vec::with_capacity(self.len());
        for i in 0..self.len() {
            if let Some(v) = self.get(i) {
                result.push(v);
            }
        }
        result
    }
}

impl<T: Copy> Drop for List<T> {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe { ffi::list_free(self.inner) };
        }
    }
}

impl<T: Copy> Default for List<T> {
    fn default() -> Self {
        Self::new().expect("Failed to create List")
    }
}

unsafe impl<T: Copy + Send> Send for List<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_back_get() {
        let mut list = List::<i32>::new().unwrap();
        list.push_back(10);
        list.push_back(20);
        list.push_back(30);
        assert_eq!(list.len(), 3);
        assert_eq!(list.get(0), Some(10));
        assert_eq!(list.get(1), Some(20));
        assert_eq!(list.get(2), Some(30));
    }

    #[test]
    fn test_push_front() {
        let mut list = List::<i32>::new().unwrap();
        list.push_front(1);
        list.push_front(2);
        list.push_front(3);
        assert_eq!(list.get(0), Some(3));
        assert_eq!(list.get(2), Some(1));
    }

    #[test]
    fn test_pop() {
        let mut list = List::<i32>::new().unwrap();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);
        list.pop_back();
        assert_eq!(list.len(), 2);
        list.pop_front();
        assert_eq!(list.len(), 1);
        assert_eq!(list.get(0), Some(2));
    }

    #[test]
    fn test_insert() {
        let mut list = List::<i32>::new().unwrap();
        list.push_back(1);
        list.push_back(3);
        list.insert(1, 2);
        assert_eq!(list.to_vec(), vec![1, 2, 3]);
    }

    #[test]
    fn test_clear() {
        let mut list = List::<i32>::new().unwrap();
        list.push_back(1);
        list.push_back(2);
        list.clear();
        assert!(list.is_empty());
    }
}

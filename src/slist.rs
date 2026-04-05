use std::marker::PhantomData;
use std::mem;

use crate::ffi;

/// A type-safe singly linked list backed by solidc's slist.
pub struct SList<T: Copy> {
    inner: *mut ffi::slist,
    _phantom: PhantomData<T>,
}

impl<T: Copy> SList<T> {
    /// Creates a new empty singly linked list.
    pub fn new() -> Option<Self> {
        let ptr = unsafe { ffi::slist_new(mem::size_of::<T>()) };
        if ptr.is_null() {
            None
        } else {
            Some(SList { inner: ptr, _phantom: PhantomData })
        }
    }

    /// Returns the number of elements.
    pub fn len(&self) -> usize {
        unsafe { ffi::slist_size(self.inner) }
    }

    /// Returns true if the list is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Prepends an element to the front.
    pub fn push_front(&mut self, value: T) {
        unsafe { ffi::slist_push_front(self.inner, &value as *const T as *mut _) };
    }

    /// Appends an element to the back.
    pub fn push_back(&mut self, value: T) {
        unsafe { ffi::slist_push_back(self.inner, &value as *const T as *mut _) };
    }

    /// Removes the first element.
    pub fn pop_front(&mut self) {
        unsafe { ffi::slist_pop_front(self.inner) };
    }

    /// Gets the element at the given index.
    pub fn get(&self, index: usize) -> Option<T> {
        let ptr = unsafe { ffi::slist_get(self.inner, index) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { *(ptr as *const T) })
        }
    }

    /// Inserts an element at the given index.
    pub fn insert(&mut self, index: usize, value: T) {
        unsafe { ffi::slist_insert(self.inner, index, &value as *const T as *mut _) };
    }

    /// Removes the element at the given index.
    pub fn remove(&mut self, index: usize) {
        unsafe { ffi::slist_remove(self.inner, index) };
    }

    /// Clears all elements.
    pub fn clear(&mut self) {
        unsafe { ffi::slist_clear(self.inner) };
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

impl<T: Copy> Drop for SList<T> {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe { ffi::slist_free(self.inner) };
        }
    }
}

impl<T: Copy> Default for SList<T> {
    fn default() -> Self {
        Self::new().expect("Failed to create SList")
    }
}

unsafe impl<T: Copy + Send> Send for SList<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_back_get() {
        let mut list = SList::<i32>::new().unwrap();
        list.push_back(10);
        list.push_back(20);
        list.push_back(30);
        assert_eq!(list.len(), 3);
        assert_eq!(list.get(0), Some(10));
        assert_eq!(list.get(2), Some(30));
    }

    #[test]
    fn test_push_front() {
        let mut list = SList::<i32>::new().unwrap();
        list.push_front(1);
        list.push_front(2);
        assert_eq!(list.get(0), Some(2));
        assert_eq!(list.get(1), Some(1));
    }

    #[test]
    fn test_remove() {
        let mut list = SList::<i32>::new().unwrap();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);
        list.remove(1);
        assert_eq!(list.to_vec(), vec![1, 3]);
    }

    #[test]
    fn test_insert() {
        let mut list = SList::<i32>::new().unwrap();
        list.push_back(1);
        list.push_back(3);
        list.insert(1, 2);
        assert_eq!(list.to_vec(), vec![1, 2, 3]);
    }

    #[test]
    fn test_clear() {
        let mut list = SList::<i32>::new().unwrap();
        list.push_back(1);
        list.push_back(2);
        list.clear();
        assert!(list.is_empty());
    }
}

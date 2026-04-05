use std::ffi::CString;

use crate::ffi;

unsafe extern "C" fn key_compare_char_ptr(
    key1: *const std::ffi::c_void,
    key2: *const std::ffi::c_void,
) -> bool {
    if key1 == key2 {
        return true;
    }
    if key1.is_null() || key2.is_null() {
        return false;
    }
    unsafe { libc::strcmp(key1 as *const i8, key2 as *const i8) == 0 }
}

/// Hash function that uses strlen to determine key length, so rehashing is consistent.
unsafe extern "C" fn string_hash(key: *const std::ffi::c_void, _key_len: usize) -> usize {
    if key.is_null() {
        return 0;
    }
    let s = key as *const u8;
    let mut len = 0usize;
    unsafe {
        while *s.add(len) != 0 {
            len += 1;
        }
    }
    // FNV-1a
    let mut h: usize = 0xcbf29ce484222325;
    for i in 0..len {
        h ^= unsafe { *s.add(i) } as usize;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

/// A hash map that maps byte-key to arbitrary values.
pub struct HashMap {
    inner: *mut ffi::HashMap,
}

impl HashMap {
    /// Creates a new hash map with default configuration.
    pub fn new() -> Option<Self> {
        let config = ffi::MapConfig {
            initial_capacity: 16,
            key_compare: Some(key_compare_char_ptr),
            key_free: None,
            value_free: None,
            max_load_factor: 0.75,
            hash_func: Some(string_hash),
        };
        let ptr = unsafe { ffi::map_create(&config) };
        if ptr.is_null() {
            None
        } else {
            Some(HashMap { inner: ptr })
        }
    }

    /// Creates a new hash map with the specified initial capacity.
    pub fn with_capacity(capacity: usize) -> Option<Self> {
        let config = ffi::MapConfig {
            initial_capacity: capacity,
            key_compare: Some(key_compare_char_ptr),
            key_free: None,
            value_free: None,
            max_load_factor: 0.75,
            hash_func: Some(string_hash),
        };
        let ptr = unsafe { ffi::map_create(&config) };
        if ptr.is_null() {
            None
        } else {
            Some(HashMap { inner: ptr })
        }
    }

    /// Inserts a key-value pair. The key and value are copied.
    ///
    /// Returns true if successful.
    pub fn insert(&mut self, key: &str, value: &[u8]) -> bool {
        let key_c = CString::new(key).unwrap();
        let key_ptr = key_c.into_raw();
        let value_copy = Box::new(value.to_vec());
        let value_ptr = Box::into_raw(value_copy);
        unsafe {
            ffi::map_set(
                self.inner,
                key_ptr as *mut libc::c_void,
                key.len(),
                value_ptr as *mut libc::c_void,
            )
        }
    }

    /// Inserts a string key-value pair.
    pub fn insert_str(&mut self, key: &str, value: &str) -> bool {
        self.insert(key, value.as_bytes())
    }

    /// Gets the raw pointer for a key. Returns None if not found.
    pub fn get_raw(&self, key: &str) -> Option<*mut libc::c_void> {
        let key_c = CString::new(key).unwrap();
        let ptr =
            unsafe { ffi::map_get(self.inner, key_c.as_ptr() as *mut libc::c_void, key.len()) };
        if ptr.is_null() { None } else { Some(ptr) }
    }

    /// Gets a value as bytes. Returns None if not found.
    pub fn get(&self, key: &str) -> Option<&Vec<u8>> {
        self.get_raw(key)
            .map(|ptr| unsafe { &*(ptr as *const Vec<u8>) })
    }

    /// Gets a value as a string. Returns None if not found or not valid UTF-8.
    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.get(key)
            .and_then(|bytes| std::str::from_utf8(bytes).ok())
    }

    /// Removes a key-value pair. Returns true if the key was found and removed.
    pub fn remove(&mut self, key: &str) -> bool {
        let key_c = CString::new(key).unwrap();
        unsafe { ffi::map_remove(self.inner, key_c.as_ptr() as *mut libc::c_void, key.len()) }
    }

    /// Returns the number of entries in the map.
    pub fn len(&self) -> usize {
        unsafe { ffi::map_length(self.inner) }
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the current capacity.
    pub fn capacity(&self) -> usize {
        unsafe { ffi::map_capacity(self.inner) }
    }

    /// Returns true if the map contains the given key.
    pub fn contains_key(&self, key: &str) -> bool {
        self.get_raw(key).is_some()
    }

    // --- Thread-safe variants ---

    /// Thread-safe insert. Acquires the internal lock.
    pub fn insert_safe(&mut self, key: &str, value: &[u8]) -> bool {
        let key_c = CString::new(key).unwrap();
        let key_ptr = key_c.into_raw();
        let value_copy = Box::new(value.to_vec());
        let value_ptr = Box::into_raw(value_copy);
        unsafe {
            ffi::map_set_safe(
                self.inner,
                key_ptr as *mut libc::c_void,
                key.len(),
                value_ptr as *mut libc::c_void,
            )
        }
    }

    /// Thread-safe string insert.
    pub fn insert_str_safe(&mut self, key: &str, value: &str) -> bool {
        self.insert_safe(key, value.as_bytes())
    }

    /// Thread-safe get returning a raw pointer. Acquires the internal lock.
    pub fn get_raw_safe(&self, key: &str) -> Option<*mut libc::c_void> {
        let key_c = CString::new(key).unwrap();
        let ptr = unsafe {
            ffi::map_get_safe(self.inner, key_c.as_ptr() as *mut libc::c_void, key.len())
        };
        if ptr.is_null() { None } else { Some(ptr) }
    }

    /// Thread-safe get as bytes.
    pub fn get_safe(&self, key: &str) -> Option<&Vec<u8>> {
        self.get_raw_safe(key)
            .map(|ptr| unsafe { &*(ptr as *const Vec<u8>) })
    }

    /// Thread-safe get as string.
    pub fn get_str_safe(&self, key: &str) -> Option<&str> {
        self.get_safe(key)
            .and_then(|bytes| std::str::from_utf8(bytes).ok())
    }

    /// Thread-safe remove. Acquires the internal lock.
    pub fn remove_safe(&mut self, key: &str) -> bool {
        let key_c = CString::new(key).unwrap();
        unsafe { ffi::map_remove_safe(self.inner, key_c.as_ptr() as *mut libc::c_void, key.len()) }
    }
}

impl Drop for HashMap {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe { ffi::map_destroy(self.inner) };
        }
    }
}

impl Default for HashMap {
    fn default() -> Self {
        HashMap::new().expect("Failed to create HashMap")
    }
}

unsafe impl Send for HashMap {}
unsafe impl Sync for HashMap {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create() {
        let map = HashMap::new().unwrap();
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn test_insert_and_get() {
        let mut map = HashMap::new().unwrap();
        assert!(map.insert_str("hello", "world"));
        assert_eq!(map.len(), 1);
        assert!(map.contains_key("hello"));

        let val = map.get_str("hello").unwrap();
        assert_eq!(val, "world");
    }

    #[test]
    fn test_remove() {
        let mut map = HashMap::new().unwrap();
        map.insert_str("key", "value");
        assert_eq!(map.len(), 1);

        assert!(map.remove("key"));
        assert_eq!(map.len(), 0);
        assert!(!map.contains_key("key"));
    }

    #[test]
    fn test_get_nonexistent() {
        let map = HashMap::new().unwrap();
        assert!(map.get_str("nope").is_none());
    }

    #[test]
    fn test_multiple_entries() {
        let mut map = HashMap::new().unwrap();
        for i in 0..50 {
            let key = format!("key{}", i);
            let val = format!("val{}", i);
            assert!(map.insert_str(&key, &val));
        }
        assert_eq!(map.len(), 50);

        for i in 0..50 {
            let key = format!("key{}", i);
            let expected = format!("val{}", i);
            assert_eq!(map.get_str(&key).unwrap(), expected);
        }
    }

    #[test]
    fn test_safe_operations() {
        let mut map = HashMap::new().unwrap();
        assert!(map.insert_str_safe("a", "alpha"));
        assert!(map.insert_str_safe("b", "beta"));
        assert_eq!(map.get_str_safe("a").unwrap(), "alpha");
        assert_eq!(map.get_str_safe("b").unwrap(), "beta");
        assert!(map.remove_safe("a"));
        assert!(map.get_str_safe("a").is_none());
        assert_eq!(map.len(), 1);
    }
}

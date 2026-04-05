use std::ffi::CString;

use crate::ffi;

/// Thread-safe LRU cache with TTL support.
pub struct Cache {
    inner: *mut ffi::cache_t,
}

impl Cache {
    /// Creates a new cache with the given capacity and default TTL (in seconds).
    pub fn new(capacity: usize, default_ttl_secs: u32) -> Option<Self> {
        let ptr = unsafe { ffi::cache_create(capacity, default_ttl_secs) };
        if ptr.is_null() {
            None
        } else {
            Some(Cache { inner: ptr })
        }
    }

    /// Sets a key-value pair with optional TTL override.
    ///
    /// Pass `0` for `ttl_override` to use the default TTL.
    pub fn set(&self, key: &str, value: &[u8], ttl_override: u32) -> bool {
        unsafe {
            ffi::cache_set(
                self.inner,
                key.as_ptr() as *const libc::c_char,
                key.len(),
                value.as_ptr() as *const libc::c_void,
                value.len(),
                ttl_override,
            )
        }
    }

    /// Sets a string value.
    pub fn set_str(&self, key: &str, value: &str) -> bool {
        self.set(key, value.as_bytes(), 0)
    }

    /// Gets a value by key. The returned data is a copy.
    ///
    /// The underlying C API uses zero-copy with `cache_release`, but for safety
    /// we copy the data and release immediately.
    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        let mut out_len: usize = 0;
        let ptr = unsafe {
            ffi::cache_get(
                self.inner,
                key.as_ptr() as *const libc::c_char,
                key.len(),
                &mut out_len,
            )
        };
        if ptr.is_null() {
            return None;
        }
        let data = unsafe { std::slice::from_raw_parts(ptr as *const u8, out_len).to_vec() };
        unsafe { ffi::cache_release(ptr) };
        Some(data)
    }

    /// Gets a string value by key.
    pub fn get_str(&self, key: &str) -> Option<String> {
        self.get(key)
            .and_then(|bytes| String::from_utf8(bytes).ok())
    }

    /// Invalidates (removes) a key from the cache.
    pub fn invalidate(&self, key: &str) {
        let Ok(cstr) = CString::new(key) else { return };
        unsafe { ffi::cache_invalidate(self.inner, cstr.as_ptr()) }
    }

    /// Clears all entries from the cache.
    pub fn clear(&self) {
        unsafe { ffi::cache_clear(self.inner) }
    }

    /// Returns the total size of the cache in bytes.
    pub fn size(&self) -> usize {
        unsafe { ffi::get_total_cache_size(self.inner) }
    }

    /// Returns the total capacity.
    pub fn capacity(&self) -> usize {
        unsafe { ffi::get_total_capacity(self.inner) }
    }

    /// Saves the cache to a file.
    pub fn save(&self, filename: &str) -> bool {
        let Ok(cstr) = CString::new(filename) else {
            return false;
        };
        unsafe { ffi::cache_save(self.inner, cstr.as_ptr()) }
    }

    /// Loads the cache from a file.
    pub fn load(&self, filename: &str) -> bool {
        let Ok(cstr) = CString::new(filename) else {
            return false;
        };
        unsafe { ffi::cache_load(self.inner, cstr.as_ptr()) }
    }
}

impl Drop for Cache {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe { ffi::cache_destroy(self.inner) };
        }
    }
}

unsafe impl Send for Cache {}
unsafe impl Sync for Cache {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create() {
        let cache = Cache::new(100, 60).unwrap();
        assert!(cache.capacity() > 0);
    }

    #[test]
    fn test_set_get() {
        let cache = Cache::new(100, 60).unwrap();
        assert!(cache.set_str("key1", "value1"));

        let val = cache.get_str("key1").unwrap();
        assert_eq!(val, "value1");
    }

    #[test]
    fn test_get_nonexistent() {
        let cache = Cache::new(100, 60).unwrap();
        assert!(cache.get_str("nope").is_none());
    }

    #[test]
    fn test_invalidate() {
        let cache = Cache::new(100, 60).unwrap();
        cache.set_str("key", "val");
        cache.invalidate("key");
        assert!(cache.get_str("key").is_none());
    }

    #[test]
    fn test_clear() {
        let cache = Cache::new(100, 60).unwrap();
        cache.set_str("a", "1");
        cache.set_str("b", "2");
        cache.clear();
        assert!(cache.get_str("a").is_none());
        assert!(cache.get_str("b").is_none());
    }

    #[test]
    fn test_binary_data() {
        let cache = Cache::new(100, 60).unwrap();
        let data: Vec<u8> = (0..255).collect();
        assert!(cache.set("bin", &data, 0));
        let retrieved = cache.get("bin").unwrap();
        assert_eq!(retrieved, data);
    }

    #[test]
    fn test_save_load() {
        let path = "/tmp/test_solidc_cache.bin";
        {
            let cache = Cache::new(100, 60).unwrap();
            cache.set_str("persist", "data");
            assert!(cache.save(path));
        }
        {
            let cache = Cache::new(100, 60).unwrap();
            assert!(cache.load(path));
            assert_eq!(cache.get_str("persist").unwrap(), "data");
        }
        std::fs::remove_file(path).ok();
    }
}

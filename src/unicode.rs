use std::ffi::{CStr, CString};
use std::fmt;

use crate::ffi;

/// A UTF-8 string with codepoint awareness.
///
/// Unlike Rust's `String`, this tracks both byte length and codepoint count.
pub struct Utf8String {
    inner: *mut ffi::utf8_string,
}

impl Utf8String {
    /// Creates a new UTF-8 string from the given data.
    pub fn new(s: &str) -> Option<Self> {
        let Ok(cstr) = CString::new(s) else { return None };
        let ptr = unsafe { ffi::utf8_new(cstr.as_ptr()) };
        if ptr.is_null() {
            None
        } else {
            Some(Utf8String { inner: ptr })
        }
    }

    /// Creates a new empty UTF-8 string with the given byte capacity.
    pub fn with_capacity(capacity: usize) -> Option<Self> {
        let ptr = unsafe { ffi::utf8_new_with_capacity(capacity) };
        if ptr.is_null() {
            None
        } else {
            Some(Utf8String { inner: ptr })
        }
    }

    /// Returns the underlying data as a `&str`.
    pub fn as_str(&self) -> &str {
        unsafe {
            let ptr = ffi::utf8_data(self.inner);
            if ptr.is_null() {
                return "";
            }
            let cstr = CStr::from_ptr(ptr);
            cstr.to_str().unwrap_or("")
        }
    }

    /// Returns the byte length.
    pub fn byte_len(&self) -> usize {
        unsafe { (*self.inner).length }
    }

    /// Returns the number of Unicode codepoints.
    pub fn codepoint_count(&self) -> usize {
        unsafe { (*self.inner).count }
    }

    /// Appends a string.
    pub fn push_str(&mut self, s: &str) -> bool {
        let Ok(cstr) = CString::new(s) else { return false };
        unsafe { ffi::utf8_append(self.inner, cstr.as_ptr()) }
    }

    /// Inserts a string at the given byte index.
    pub fn insert(&mut self, index: usize, s: &str) -> bool {
        let Ok(cstr) = CString::new(s) else { return false };
        unsafe { ffi::utf8_insert(self.inner, index, cstr.as_ptr()) }
    }

    /// Removes `count` bytes starting at `index`.
    pub fn remove(&mut self, index: usize, count: usize) -> bool {
        unsafe { ffi::utf8_remove(self.inner, index, count) }
    }

    /// Replaces the first occurrence of `old` with `new_str`.
    pub fn replace(&mut self, old: &str, new_str: &str) -> bool {
        let Ok(old_c) = CString::new(old) else { return false };
        let Ok(new_c) = CString::new(new_str) else { return false };
        unsafe { ffi::utf8_replace(self.inner, old_c.as_ptr(), new_c.as_ptr()) }
    }

    /// Replaces all occurrences of `old` with `new_str`.
    pub fn replace_all(&mut self, old: &str, new_str: &str) -> usize {
        let Ok(old_c) = CString::new(old) else { return 0 };
        let Ok(new_c) = CString::new(new_str) else { return 0 };
        unsafe { ffi::utf8_replace_all(self.inner, old_c.as_ptr(), new_c.as_ptr()) }
    }

    /// Reverses the string in place (codepoint-aware).
    pub fn reverse(&mut self) -> bool {
        unsafe { ffi::utf8_reverse(self.inner) }
    }

    /// Finds the first occurrence of a substring as a byte index.
    pub fn find(&self, substr: &str) -> Option<usize> {
        let Ok(cstr) = CString::new(substr) else { return None };
        let idx = unsafe { ffi::utf8_index_of(self.inner, cstr.as_ptr()) };
        if idx < 0 { None } else { Some(idx as usize) }
    }

    /// Finds the last occurrence of a substring.
    pub fn rfind(&self, substr: &str) -> Option<usize> {
        let Ok(cstr) = CString::new(substr) else { return None };
        let idx = unsafe { ffi::utf8_last_index_of(self.inner, cstr.as_ptr()) };
        if idx < 0 { None } else { Some(idx as usize) }
    }

    /// Clones this UTF-8 string.
    pub fn try_clone(&self) -> Option<Self> {
        let ptr = unsafe { ffi::utf8_clone(self.inner) };
        if ptr.is_null() { None } else { Some(Utf8String { inner: ptr }) }
    }

    /// Concatenates two UTF-8 strings.
    pub fn concat(&self, other: &Utf8String) -> Option<Self> {
        let ptr = unsafe { ffi::utf8_concat(self.inner, other.inner) };
        if ptr.is_null() { None } else { Some(Utf8String { inner: ptr }) }
    }

    /// Writes the string to a file.
    pub fn write_to(&self, filename: &str) -> Result<usize, ()> {
        let Ok(cstr) = CString::new(filename) else { return Err(()) };
        let result = unsafe { ffi::utf8_writeto(self.inner, cstr.as_ptr()) };
        if result < 0 { Err(()) } else { Ok(result as usize) }
    }

    /// Reads a UTF-8 string from a file.
    pub fn read_from(filename: &str) -> Option<Self> {
        let Ok(cstr) = CString::new(filename) else { return None };
        let ptr = unsafe { ffi::utf8_readfrom(cstr.as_ptr()) };
        if ptr.is_null() { None } else { Some(Utf8String { inner: ptr }) }
    }
}

impl Drop for Utf8String {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe { ffi::utf8_free(self.inner) };
        }
    }
}

impl Clone for Utf8String {
    fn clone(&self) -> Self {
        self.try_clone().expect("Failed to clone Utf8String")
    }
}

impl fmt::Display for Utf8String {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl fmt::Debug for Utf8String {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Utf8String({:?}, {} codepoints)", self.as_str(), self.codepoint_count())
    }
}

impl PartialEq for Utf8String {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for Utf8String {}

impl From<&str> for Utf8String {
    fn from(s: &str) -> Self {
        Utf8String::new(s).expect("Failed to create Utf8String")
    }
}

unsafe impl Send for Utf8String {}

/// Validates whether a string is valid UTF-8.
pub fn is_valid_utf8(s: &str) -> bool {
    let Ok(cstr) = CString::new(s) else { return false };
    unsafe { ffi::is_valid_utf8(cstr.as_ptr()) }
}

/// Counts the number of Unicode codepoints in a string.
pub fn count_codepoints(s: &str) -> usize {
    let Ok(cstr) = CString::new(s) else { return 0 };
    unsafe { ffi::utf8_count_codepoints(cstr.as_ptr()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let s = Utf8String::new("Hello, 世界!").unwrap();
        assert_eq!(s.as_str(), "Hello, 世界!");
        assert_eq!(s.codepoint_count(), 10); // H e l l o ,   世 界 !
    }

    #[test]
    fn test_append() {
        let mut s = Utf8String::new("Hello").unwrap();
        s.push_str(", World!");
        assert_eq!(s.as_str(), "Hello, World!");
    }

    #[test]
    fn test_find() {
        let s = Utf8String::new("Hello, World!").unwrap();
        assert_eq!(s.find("World"), Some(7));
        assert_eq!(s.find("xyz"), None);
    }

    #[test]
    fn test_replace() {
        let mut s = Utf8String::new("Hello, World!").unwrap();
        s.replace("World", "Rust");
        assert_eq!(s.as_str(), "Hello, Rust!");
    }

    #[test]
    fn test_replace_all() {
        let mut s = Utf8String::new("aabaa").unwrap();
        let n = s.replace_all("a", "x");
        assert_eq!(s.as_str(), "xxbxx");
        assert_eq!(n, 4);
    }

    #[test]
    fn test_reverse() {
        let mut s = Utf8String::new("abc").unwrap();
        s.reverse();
        assert_eq!(s.as_str(), "cba");
    }

    #[test]
    fn test_concat() {
        let a = Utf8String::new("Hello, ").unwrap();
        let b = Utf8String::new("World!").unwrap();
        let c = a.concat(&b).unwrap();
        assert_eq!(c.as_str(), "Hello, World!");
    }

    #[test]
    fn test_clone() {
        let s = Utf8String::new("test").unwrap();
        let s2 = s.clone();
        assert_eq!(s, s2);
    }

    #[test]
    fn test_is_valid_utf8() {
        assert!(is_valid_utf8("Hello, 世界!"));
    }

    #[test]
    fn test_count_codepoints() {
        assert_eq!(count_codepoints("Hello"), 5);
        assert_eq!(count_codepoints("世界"), 2);
    }

    #[test]
    fn test_file_io() {
        let path = "/tmp/test_solidc_utf8.txt";
        let s = Utf8String::new("Hello, 世界!").unwrap();
        s.write_to(path).unwrap();

        let loaded = Utf8String::read_from(path).unwrap();
        assert_eq!(loaded.as_str(), "Hello, 世界!");
        std::fs::remove_file(path).ok();
    }
}

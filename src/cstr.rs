use std::ffi::CString;
use std::fmt;
use std::ops::Deref;

use crate::ffi;

/// A heap-allocated string with small-string optimization (SSO), wrapping the C `cstr` type.
///
/// Strings of 15 bytes or less are stored inline without heap allocation.
pub struct CStr_ {
    inner: *mut ffi::cstr,
}

impl CStr_ {
    /// Creates a new empty string with the given initial capacity.
    pub fn with_capacity(capacity: usize) -> Option<Self> {
        let ptr = unsafe { ffi::cstr_init(capacity) };
        if ptr.is_null() {
            None
        } else {
            Some(CStr_ { inner: ptr })
        }
    }

    /// Creates a new string from the given content.
    pub fn new(s: &str) -> Option<Self> {
        let cstr = CString::new(s).ok()?;
        let ptr = unsafe { ffi::cstr_new(cstr.as_ptr()) };
        if ptr.is_null() {
            None
        } else {
            Some(CStr_ { inner: ptr })
        }
    }

    /// Returns the length of the string in bytes.
    pub fn len(&self) -> usize {
        unsafe { (*self.inner).length as usize }
    }

    /// Returns true if the string is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the capacity of the string.
    pub fn capacity(&self) -> usize {
        unsafe { (*self.inner).capacity as usize }
    }

    /// Returns a reference to the string as a `&str`.
    pub fn as_str(&self) -> &str {
        unsafe {
            let s = &*self.inner;
            let ptr = s.data;
            let len = s.length as usize;
            let bytes = std::slice::from_raw_parts(ptr as *const u8, len);
            std::str::from_utf8_unchecked(bytes)
        }
    }

    /// Returns the byte at the given index.
    pub fn at(&self, index: usize) -> Option<u8> {
        if index < self.len() {
            Some(self.as_str().as_bytes()[index])
        } else {
            None
        }
    }

    /// Returns whether the string was heap-allocated (not using SSO).
    pub fn is_heap_allocated(&self) -> bool {
        unsafe {
            let s = &*self.inner;
            // SSO: data points to the inline buf, capacity == 0
            s.capacity > 0
        }
    }

    /// Reserves capacity for at least `additional` more bytes.
    pub fn reserve(&mut self, capacity: usize) -> bool {
        unsafe { ffi::cstr_reserve(self.inner, capacity) }
    }

    /// Shrinks the capacity to match the length.
    pub fn shrink_to_fit(&mut self) {
        unsafe { ffi::cstr_shrink_to_fit(self.inner) }
    }

    /// Clears the string, setting length to 0.
    pub fn clear(&mut self) {
        unsafe {
            (*self.inner).length = 0;
            let s = &mut *self.inner;
            let ptr = if s.data.is_null() {
                s.buf.as_mut_ptr()
            } else {
                s.data
            };
            *ptr = 0;
        }
    }

    /// Appends a string slice.
    pub fn push_str(&mut self, s: &str) -> bool {
        let Ok(cstr) = CString::new(s) else {
            return false;
        };
        unsafe { ffi::cstr_append(self.inner, cstr.as_ptr()) }
    }

    /// Appends a single character.
    pub fn push(&mut self, c: char) -> bool {
        let mut buf = [0u8; 4];
        let s = c.encode_utf8(&mut buf);
        // For single ASCII chars, use the fast path
        if c.is_ascii() {
            return unsafe { ffi::cstr_append_char(self.inner, c as libc::c_char) };
        }
        self.push_str(s)
    }

    /// Prepends a string slice.
    pub fn prepend(&mut self, s: &str) -> bool {
        let Ok(cstr) = CString::new(s) else {
            return false;
        };
        unsafe { ffi::cstr_prepend(self.inner, cstr.as_ptr()) }
    }

    /// Inserts a string at the given byte index.
    pub fn insert_str(&mut self, index: usize, s: &str) -> bool {
        let Ok(cstr) = CString::new(s) else {
            return false;
        };
        unsafe { ffi::cstr_insert(self.inner, index, cstr.as_ptr()) }
    }

    /// Removes `count` bytes starting at `index`.
    pub fn remove_range(&mut self, index: usize, count: usize) -> bool {
        unsafe { ffi::cstr_remove(self.inner, index, count) }
    }

    /// Removes all occurrences of a substring.
    pub fn remove_all(&mut self, substr: &str) -> usize {
        let Ok(cstr) = CString::new(substr) else {
            return 0;
        };
        unsafe { ffi::cstr_remove_all(self.inner, cstr.as_ptr()) }
    }

    /// Removes all occurrences of a character.
    pub fn remove_char(&mut self, c: char) {
        if c.is_ascii() {
            unsafe { ffi::cstr_remove_char(self.inner, c as libc::c_char) }
        }
    }

    /// Converts the string to lowercase in place.
    pub fn to_lowercase(&mut self) {
        unsafe { ffi::cstr_lower(self.inner) }
    }

    /// Converts the string to uppercase in place.
    pub fn to_uppercase(&mut self) {
        unsafe { ffi::cstr_upper(self.inner) }
    }

    /// Converts to snake_case in place.
    pub fn to_snakecase(&mut self) -> bool {
        unsafe { ffi::cstr_snakecase(self.inner) }
    }

    /// Converts to camelCase in place.
    pub fn to_camelcase(&mut self) {
        unsafe { ffi::cstr_camelcase(self.inner) }
    }

    /// Converts to PascalCase in place.
    pub fn to_pascalcase(&mut self) {
        unsafe { ffi::cstr_pascalcase(self.inner) }
    }

    /// Converts to Title Case in place.
    pub fn to_titlecase(&mut self) {
        unsafe { ffi::cstr_titlecase(self.inner) }
    }

    /// Trims whitespace from both ends.
    pub fn trim(&mut self) {
        unsafe { ffi::cstr_trim(self.inner) }
    }

    /// Trims whitespace from the right.
    pub fn trim_end(&mut self) {
        unsafe { ffi::cstr_rtrim(self.inner) }
    }

    /// Trims whitespace from the left.
    pub fn trim_start(&mut self) {
        unsafe { ffi::cstr_ltrim(self.inner) }
    }

    /// Trims specific characters from both ends.
    pub fn trim_chars(&mut self, chars: &str) {
        let Ok(cstr) = CString::new(chars) else {
            return;
        };
        unsafe { ffi::cstr_trim_chars(self.inner, cstr.as_ptr()) }
    }

    /// Compares with another CStr_.
    pub fn cmp(&self, other: &CStr_) -> std::cmp::Ordering {
        let result = unsafe { ffi::cstr_cmp(self.inner, other.inner) };
        result.cmp(&0)
    }

    /// Returns true if the string starts with the given prefix.
    pub fn starts_with(&self, prefix: &str) -> bool {
        let Ok(cstr) = CString::new(prefix) else {
            return false;
        };
        unsafe { ffi::cstr_starts_with(self.inner, cstr.as_ptr()) }
    }

    /// Returns true if the string ends with the given suffix.
    pub fn ends_with(&self, suffix: &str) -> bool {
        let Ok(cstr) = CString::new(suffix) else {
            return false;
        };
        unsafe { ffi::cstr_ends_with(self.inner, cstr.as_ptr()) }
    }

    /// Finds the first occurrence of a substring, returning the byte index.
    pub fn find(&self, substr: &str) -> Option<usize> {
        let Ok(cstr) = CString::new(substr) else {
            return None;
        };
        let idx = unsafe { ffi::cstr_find(self.inner, cstr.as_ptr()) };
        if idx < 0 { None } else { Some(idx as usize) }
    }

    /// Finds the last occurrence of a substring, returning the byte index.
    pub fn rfind(&self, substr: &str) -> Option<usize> {
        let Ok(cstr) = CString::new(substr) else {
            return None;
        };
        let idx = unsafe { ffi::cstr_rfind(self.inner, cstr.as_ptr()) };
        if idx < 0 { None } else { Some(idx as usize) }
    }

    /// Returns true if the string contains the given substring.
    pub fn contains(&self, substr: &str) -> bool {
        self.find(substr).is_some()
    }

    /// Counts occurrences of a substring.
    pub fn count(&self, substr: &str) -> usize {
        let Ok(cstr) = CString::new(substr) else {
            return 0;
        };
        unsafe { ffi::cstr_count_substr(self.inner, cstr.as_ptr()) }
    }

    /// Returns a substring. The caller owns the returned CStr_.
    pub fn substr(&self, start: usize, length: usize) -> Option<CStr_> {
        let ptr = unsafe { ffi::cstr_substr(self.inner, start, length) };
        if ptr.is_null() {
            None
        } else {
            Some(CStr_ { inner: ptr })
        }
    }

    /// Replaces the first occurrence of `old` with `new_str`.
    /// Returns a new CStr_ with the replacement, or None on error.
    /// The original string is unchanged.
    pub fn replace(&self, old: &str, new_str: &str) -> Option<CStr_> {
        let Ok(old_c) = CString::new(old) else {
            return None;
        };
        let Ok(new_c) = CString::new(new_str) else {
            return None;
        };
        let ptr = unsafe { ffi::cstr_replace(self.inner, old_c.as_ptr(), new_c.as_ptr()) };
        if ptr.is_null() {
            None
        } else {
            Some(CStr_ { inner: ptr })
        }
    }

    /// Replaces all occurrences of `old` with `new_str`.
    pub fn replace_all(&self, old: &str, new_str: &str) -> Option<CStr_> {
        let Ok(old_c) = CString::new(old) else {
            return None;
        };
        let Ok(new_c) = CString::new(new_str) else {
            return None;
        };
        let ptr = unsafe { ffi::cstr_replace_all(self.inner, old_c.as_ptr(), new_c.as_ptr()) };
        if ptr.is_null() {
            None
        } else {
            Some(CStr_ { inner: ptr })
        }
    }

    /// Splits the string by delimiter, returning a Vec of CStr_.
    pub fn split(&self, delim: &str) -> Vec<CStr_> {
        let Ok(delim_c) = CString::new(delim) else {
            return Vec::new();
        };
        let mut count: usize = 0;
        let ptr = unsafe { ffi::cstr_split(self.inner, delim_c.as_ptr(), &mut count) };
        if ptr.is_null() {
            return Vec::new();
        }

        let mut result = Vec::with_capacity(count);
        for i in 0..count {
            let item = unsafe { *ptr.add(i) };
            if !item.is_null() {
                result.push(CStr_ { inner: item });
            }
        }
        // Free the array (but not the elements, we own those now)
        unsafe { libc::free(ptr as *mut libc::c_void) };
        result
    }

    /// Reverses the string, returning a new CStr_.
    pub fn reversed(&self) -> Option<CStr_> {
        let ptr = unsafe { ffi::cstr_reverse(self.inner) };
        if ptr.is_null() {
            None
        } else {
            Some(CStr_ { inner: ptr })
        }
    }

    /// Reverses the string in place.
    pub fn reverse(&mut self) {
        unsafe { ffi::cstr_reverse_inplace(self.inner) }
    }

    /// Joins a slice of CStr_ with a delimiter.
    pub fn join(strings: &[CStr_], delim: &str) -> Option<CStr_> {
        if strings.is_empty() {
            return CStr_::new("");
        }
        let Ok(delim_c) = CString::new(delim) else {
            return None;
        };
        let ptrs: Vec<*const ffi::cstr> = strings.iter().map(|s| s.inner as *const _).collect();
        let ptr = unsafe { ffi::cstr_join(ptrs.as_ptr() as *mut _, ptrs.len(), delim_c.as_ptr()) };
        if ptr.is_null() {
            None
        } else {
            Some(CStr_ { inner: ptr })
        }
    }
}

impl Drop for CStr_ {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe { ffi::cstr_free(self.inner) };
        }
    }
}

impl Clone for CStr_ {
    fn clone(&self) -> Self {
        CStr_::new(self.as_str()).expect("Failed to clone CStr_")
    }
}

impl fmt::Display for CStr_ {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl fmt::Debug for CStr_ {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.as_str())
    }
}

impl PartialEq for CStr_ {
    fn eq(&self, other: &Self) -> bool {
        unsafe { ffi::cstr_cmp(self.inner, other.inner) == 0 }
    }
}

impl Eq for CStr_ {}

impl PartialOrd for CStr_ {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CStr_ {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        CStr_::cmp(self, other)
    }
}

impl Deref for CStr_ {
    type Target = str;
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl From<&str> for CStr_ {
    fn from(s: &str) -> Self {
        CStr_::new(s).expect("Failed to create CStr_ from &str")
    }
}

impl From<String> for CStr_ {
    fn from(s: String) -> Self {
        CStr_::new(&s).expect("Failed to create CStr_ from String")
    }
}

// Safety: The underlying C implementation is not thread-safe for mutation
unsafe impl Send for CStr_ {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_and_display() {
        let s = CStr_::new("Hello, World!").unwrap();
        assert_eq!(s.as_str(), "Hello, World!");
        assert_eq!(s.len(), 13);
        assert!(!s.is_empty());
    }

    #[test]
    fn test_empty_string() {
        let s = CStr_::new("").unwrap();
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn test_push_str_and_push() {
        let mut s = CStr_::new("Hello").unwrap();
        s.push_str(", World!");
        assert_eq!(s.as_str(), "Hello, World!");

        let mut s2 = CStr_::new("abc").unwrap();
        s2.push('d');
        assert_eq!(s2.as_str(), "abcd");
    }

    #[test]
    fn test_prepend() {
        let mut s = CStr_::new("World").unwrap();
        s.prepend("Hello, ");
        assert_eq!(s.as_str(), "Hello, World");
    }

    #[test]
    fn test_insert() {
        let mut s = CStr_::new("Hello World").unwrap();
        s.insert_str(5, ",");
        assert_eq!(s.as_str(), "Hello, World");
    }

    #[test]
    fn test_case_conversion() {
        let mut s = CStr_::new("Hello World").unwrap();
        s.to_lowercase();
        assert_eq!(s.as_str(), "hello world");

        s.to_uppercase();
        assert_eq!(s.as_str(), "HELLO WORLD");
    }

    #[test]
    fn test_trim() {
        let mut s = CStr_::new("  hello  ").unwrap();
        s.trim();
        assert_eq!(s.as_str(), "hello");
    }

    #[test]
    fn test_find_and_contains() {
        let s = CStr_::new("Hello, World!").unwrap();
        assert_eq!(s.find("World"), Some(7));
        assert_eq!(s.rfind("l"), Some(10));
        assert!(s.contains("World"));
        assert!(!s.contains("xyz"));
    }

    #[test]
    fn test_starts_ends_with() {
        let s = CStr_::new("Hello, World!").unwrap();
        assert!(s.starts_with("Hello"));
        assert!(s.ends_with("World!"));
        assert!(!s.starts_with("World"));
    }

    #[test]
    fn test_substr() {
        let s = CStr_::new("Hello, World!").unwrap();
        let sub = s.substr(7, 5).unwrap();
        assert_eq!(sub.as_str(), "World");
    }

    #[test]
    fn test_replace() {
        let s = CStr_::new("Hello, World!").unwrap();
        let r = s.replace("World", "Rust").unwrap();
        assert_eq!(r.as_str(), "Hello, Rust!");
    }

    #[test]
    fn test_replace_all() {
        let s = CStr_::new("aabaa").unwrap();
        let r = s.replace_all("a", "x").unwrap();
        assert_eq!(r.as_str(), "xxbxx");
    }

    #[test]
    fn test_split() {
        let s = CStr_::new("a,b,c").unwrap();
        let parts = s.split(",");
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0].as_str(), "a");
        assert_eq!(parts[1].as_str(), "b");
        assert_eq!(parts[2].as_str(), "c");
    }

    #[test]
    fn test_join() {
        let parts: Vec<CStr_> = vec!["a".into(), "b".into(), "c".into()];
        let joined = CStr_::join(&parts, ", ").unwrap();
        assert_eq!(joined.as_str(), "a, b, c");
    }

    #[test]
    fn test_reverse() {
        let s = CStr_::new("hello").unwrap();
        let r = s.reversed().unwrap();
        assert_eq!(r.as_str(), "olleh");
    }

    #[test]
    fn test_clone_and_eq() {
        let s = CStr_::new("test").unwrap();
        let s2 = s.clone();
        assert_eq!(s, s2);
    }

    #[test]
    fn test_ordering() {
        let a = CStr_::new("abc").unwrap();
        let b = CStr_::new("abd").unwrap();
        assert!(a < b);
    }

    #[test]
    fn test_count() {
        let s = CStr_::new("banana").unwrap();
        assert_eq!(s.count("an"), 2);
    }

    #[test]
    fn test_remove_all() {
        let mut s = CStr_::new("hello world hello").unwrap();
        let removed = s.remove_all("hello");
        assert_eq!(removed, 2);
    }

    #[test]
    fn test_from_str() {
        let s: CStr_ = "hello".into();
        assert_eq!(s.as_str(), "hello");
    }

    #[test]
    fn test_deref_to_str() {
        let s = CStr_::new("hello").unwrap();
        // Can use &str methods directly via Deref
        assert!(s.starts_with("hel"));
        assert_eq!(s.len(), 5);
    }

    #[test]
    fn test_sso() {
        // Short string should use SSO (inline buffer)
        let s = CStr_::new("hi").unwrap();
        assert!(!s.is_heap_allocated());

        // Long string should be heap-allocated
        let long_s = CStr_::new("this is a very long string that exceeds the SSO buffer").unwrap();
        assert!(long_s.is_heap_allocated());
    }
}

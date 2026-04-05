use std::ffi::CString;

use crate::ffi;

/// A prefix tree (trie) for efficient string storage and lookup.
///
/// Supports insertion, search, prefix matching, and autocomplete.
pub struct Trie {
    inner: *mut ffi::trie_t,
}

impl Trie {
    /// Creates a new empty trie.
    pub fn new() -> Option<Self> {
        let ptr = unsafe { ffi::trie_create() };
        if ptr.is_null() {
            None
        } else {
            Some(Trie { inner: ptr })
        }
    }

    /// Inserts a word into the trie.
    pub fn insert(&mut self, word: &str) -> bool {
        let Ok(cstr) = CString::new(word) else {
            return false;
        };
        unsafe { ffi::trie_insert(self.inner, cstr.as_ptr()) }
    }

    /// Returns true if the exact word exists in the trie.
    pub fn search(&self, word: &str) -> bool {
        let Ok(cstr) = CString::new(word) else {
            return false;
        };
        unsafe { ffi::trie_search(self.inner, cstr.as_ptr()) }
    }

    /// Returns true if any word in the trie starts with the given prefix.
    pub fn starts_with(&self, prefix: &str) -> bool {
        let Ok(cstr) = CString::new(prefix) else {
            return false;
        };
        unsafe { ffi::trie_starts_with(self.inner, cstr.as_ptr()) }
    }

    /// Deletes a word from the trie.
    pub fn delete(&mut self, word: &str) -> bool {
        let Ok(cstr) = CString::new(word) else {
            return false;
        };
        unsafe { ffi::trie_delete(self.inner, cstr.as_ptr()) }
    }

    /// Returns the frequency (insertion count) of a word.
    pub fn frequency(&self, word: &str) -> u32 {
        let Ok(cstr) = CString::new(word) else {
            return 0;
        };
        unsafe { ffi::trie_get_frequency(self.inner, cstr.as_ptr()) }
    }

    /// Returns the total number of words in the trie.
    pub fn word_count(&self) -> usize {
        unsafe { ffi::trie_get_word_count(self.inner) }
    }

    /// Returns true if the trie contains no words.
    pub fn is_empty(&self) -> bool {
        unsafe { ffi::trie_is_empty(self.inner) }
    }

    /// Returns autocomplete suggestions for the given prefix.
    pub fn autocomplete(&self, prefix: &str, max_suggestions: usize) -> Vec<String> {
        let Ok(cstr) = CString::new(prefix) else {
            return Vec::new();
        };
        let mut count: usize = 0;

        // Create a temporary arena for the results
        let arena = crate::arena::Arena::new(4096);
        let Some(arena) = arena else {
            return Vec::new();
        };

        let ptr = unsafe {
            ffi::trie_autocomplete(
                self.inner,
                cstr.as_ptr(),
                max_suggestions,
                &mut count,
                arena.as_ptr(),
            )
        };

        if ptr.is_null() || count == 0 {
            return Vec::new();
        }

        let mut results = Vec::with_capacity(count);
        for i in 0..count {
            let s_ptr = unsafe { *ptr.add(i) };
            if !s_ptr.is_null() {
                let s = unsafe { std::ffi::CStr::from_ptr(s_ptr) };
                results.push(s.to_string_lossy().into_owned());
            }
        }
        // Arena owns the memory, it will be freed when arena drops
        results
    }
}

impl Drop for Trie {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe { ffi::trie_destroy(self.inner) };
        }
    }
}

impl Default for Trie {
    fn default() -> Self {
        Trie::new().expect("Failed to create Trie")
    }
}

unsafe impl Send for Trie {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create() {
        let trie = Trie::new().unwrap();
        assert!(trie.is_empty());
        assert_eq!(trie.word_count(), 0);
    }

    #[test]
    fn test_insert_and_search() {
        let mut trie = Trie::new().unwrap();
        assert!(trie.insert("hello"));
        assert!(trie.insert("world"));

        assert!(trie.search("hello"));
        assert!(trie.search("world"));
        assert!(!trie.search("hell"));
        assert!(!trie.search("worlds"));
    }

    #[test]
    fn test_starts_with() {
        let mut trie = Trie::new().unwrap();
        trie.insert("hello");
        trie.insert("help");

        assert!(trie.starts_with("hel"));
        assert!(trie.starts_with("hello"));
        assert!(!trie.starts_with("world"));
    }

    #[test]
    fn test_delete() {
        let mut trie = Trie::new().unwrap();
        trie.insert("hello");
        assert_eq!(trie.word_count(), 1);

        assert!(trie.delete("hello"));
        assert!(!trie.search("hello"));
        assert_eq!(trie.word_count(), 0);
    }

    #[test]
    fn test_frequency() {
        let mut trie = Trie::new().unwrap();
        trie.insert("hello");
        trie.insert("hello");
        trie.insert("hello");

        assert_eq!(trie.frequency("hello"), 3);
    }

    #[test]
    fn test_autocomplete() {
        let mut trie = Trie::new().unwrap();
        trie.insert("apple");
        trie.insert("application");
        trie.insert("apply");
        trie.insert("banana");

        let suggestions = trie.autocomplete("app", 10);
        assert!(!suggestions.is_empty());
        for s in &suggestions {
            assert!(s.starts_with("app"));
        }
    }

    #[test]
    fn test_word_count() {
        let mut trie = Trie::new().unwrap();
        trie.insert("a");
        trie.insert("b");
        trie.insert("c");
        assert_eq!(trie.word_count(), 3);
    }
}

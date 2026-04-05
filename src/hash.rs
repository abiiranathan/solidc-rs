use crate::ffi;

/// Non-cryptographic hash functions.
///
/// These are fast hash functions suitable for hash tables, checksums, etc.
/// They are NOT suitable for cryptographic purposes.
pub mod hash {
    use super::ffi;
    use std::ffi::CString;

    /// DJB2 hash function (Daniel J. Bernstein).
    pub fn djb2(key: &str) -> u32 {
        let Ok(cstr) = CString::new(key) else {
            return 0;
        };
        unsafe { ffi::solidc_djb2_hash(cstr.as_ptr() as *const libc::c_void) }
    }

    /// DJB2a hash function (XOR variant).
    pub fn djb2a(key: &str) -> u32 {
        let Ok(cstr) = CString::new(key) else {
            return 0;
        };
        unsafe { ffi::solidc_djb2a_hash(cstr.as_ptr() as *const libc::c_void) }
    }

    /// SDBM hash function.
    pub fn sdbm(key: &str) -> u32 {
        let Ok(cstr) = CString::new(key) else {
            return 0;
        };
        unsafe { ffi::solidc_sdbm_hash(cstr.as_ptr() as *const libc::c_void) }
    }

    /// FNV-1a 32-bit hash function.
    pub fn fnv1a(key: &str) -> u32 {
        let Ok(cstr) = CString::new(key) else {
            return 0;
        };
        unsafe { ffi::solidc_fnv1a_hash(cstr.as_ptr() as *const libc::c_void) }
    }

    /// FNV-1a 64-bit hash function.
    pub fn fnv1a_64(key: &str) -> u64 {
        let Ok(cstr) = CString::new(key) else {
            return 0;
        };
        unsafe { ffi::solidc_fnv1a_hash64(cstr.as_ptr() as *const libc::c_void) }
    }

    /// ELF hash function.
    pub fn elf(key: &str) -> u32 {
        let Ok(cstr) = CString::new(key) else {
            return 0;
        };
        unsafe { ffi::solidc_elf_hash(cstr.as_ptr() as *const libc::c_void) }
    }

    /// CRC32 hash function.
    pub fn crc32(data: &[u8]) -> u32 {
        unsafe { ffi::solidc_crc32_hash(data.as_ptr() as *const libc::c_void, data.len()) }
    }

    /// MurmurHash function with a seed.
    pub fn murmur(key: &str, seed: u32) -> u32 {
        let Ok(cstr) = CString::new(key) else {
            return 0;
        };
        unsafe { ffi::solidc_murmur_hash(cstr.as_ptr(), key.len() as u32, seed) }
    }
}

#[cfg(test)]
mod tests {
    use super::hash;

    #[test]
    fn test_djb2() {
        let h = hash::djb2("hello");
        assert_ne!(h, 0);
        // Same input should produce same hash
        assert_eq!(h, hash::djb2("hello"));
    }

    #[test]
    fn test_fnv1a() {
        let h32 = hash::fnv1a("test");
        let h64 = hash::fnv1a_64("test");
        assert_ne!(h32, 0);
        assert_ne!(h64, 0);
    }

    #[test]
    fn test_crc32() {
        let h = hash::crc32(b"hello world");
        assert_ne!(h, 0);
        assert_eq!(h, hash::crc32(b"hello world"));
    }

    #[test]
    fn test_murmur() {
        let h = hash::murmur("hello", 42);
        assert_ne!(h, 0);
        // Different seeds should produce different hashes
        assert_ne!(h, hash::murmur("hello", 0));
    }

    #[test]
    fn test_different_hashes() {
        // Different algorithms should (usually) produce different values
        let h1 = hash::djb2("test");
        let h2 = hash::sdbm("test");
        let h3 = hash::fnv1a("test");
        let h4 = hash::elf("test");
        // At least some should differ
        assert!(h1 != h2 || h2 != h3 || h3 != h4);
    }
}

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use crate::ffi;

/// Status codes for flag parsing results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlagStatus {
    Ok,
    ErrorAllocation,
    ErrorUnknownFlag,
    ErrorMissingValue,
    ErrorInvalidNumber,
    ErrorValidation,
    ErrorRequiredMissing,
    ErrorUnknownSubcommand,
    ErrorInvalidArgument,
}

impl From<ffi::FlagStatus> for FlagStatus {
    fn from(s: ffi::FlagStatus) -> Self {
        match s {
            ffi::FlagStatus::FLAG_OK => FlagStatus::Ok,
            ffi::FlagStatus::FLAG_ERROR_ALLOCATION => FlagStatus::ErrorAllocation,
            ffi::FlagStatus::FLAG_ERROR_UNKNOWN_FLAG => FlagStatus::ErrorUnknownFlag,
            ffi::FlagStatus::FLAG_ERROR_MISSING_VALUE => FlagStatus::ErrorMissingValue,
            ffi::FlagStatus::FLAG_ERROR_INVALID_NUMBER => FlagStatus::ErrorInvalidNumber,
            ffi::FlagStatus::FLAG_ERROR_VALIDATION => FlagStatus::ErrorValidation,
            ffi::FlagStatus::FLAG_ERROR_REQUIRED_MISSING => FlagStatus::ErrorRequiredMissing,
            ffi::FlagStatus::FLAG_ERROR_UNKNOWN_SUBCOMMAND => FlagStatus::ErrorUnknownSubcommand,
            ffi::FlagStatus::FLAG_ERROR_INVALID_ARGUMENT => FlagStatus::ErrorInvalidArgument,
        }
    }
}

/// Supported flag data types.
#[derive(Debug, Clone, Copy)]
pub enum FlagType {
    Bool,
    Char,
    String,
    Int8,
    Uint8,
    Int16,
    Uint16,
    Int32,
    Uint32,
    Int64,
    Uint64,
    SizeT,
    Float,
    Double,
}

impl FlagType {
    fn to_c(self) -> ffi::FlagDataType {
        match self {
            FlagType::Bool => ffi::FlagDataType::TYPE_BOOL,
            FlagType::Char => ffi::FlagDataType::TYPE_CHAR,
            FlagType::String => ffi::FlagDataType::TYPE_STRING,
            FlagType::Int8 => ffi::FlagDataType::TYPE_INT8,
            FlagType::Uint8 => ffi::FlagDataType::TYPE_UINT8,
            FlagType::Int16 => ffi::FlagDataType::TYPE_INT16,
            FlagType::Uint16 => ffi::FlagDataType::TYPE_UINT16,
            FlagType::Int32 => ffi::FlagDataType::TYPE_INT32,
            FlagType::Uint32 => ffi::FlagDataType::TYPE_UINT32,
            FlagType::Int64 => ffi::FlagDataType::TYPE_INT64,
            FlagType::Uint64 => ffi::FlagDataType::TYPE_UINT64,
            FlagType::SizeT => ffi::FlagDataType::TYPE_SIZE_T,
            FlagType::Float => ffi::FlagDataType::TYPE_FLOAT,
            FlagType::Double => ffi::FlagDataType::TYPE_DOUBLE,
        }
    }
}

/// A command-line flag parser.
pub struct FlagParser {
    inner: *mut ffi::FlagParser,
    // Keep CStrings alive for the lifetime of the parser
    _strings: Vec<CString>,
}

impl FlagParser {
    /// Creates a new flag parser with the given name and description.
    pub fn new(name: &str, description: &str) -> Option<Self> {
        let name_c = CString::new(name).ok()?;
        let desc_c = CString::new(description).ok()?;
        let ptr = unsafe { ffi::flag_parser_new(name_c.as_ptr(), desc_c.as_ptr()) };
        if ptr.is_null() {
            None
        } else {
            Some(FlagParser {
                inner: ptr,
                _strings: vec![name_c, desc_c],
            })
        }
    }

    /// Sets footer text displayed at the bottom of help.
    pub fn set_footer(&mut self, footer: &str) {
        if let Ok(c) = CString::new(footer) {
            unsafe { ffi::flag_parser_set_footer(self.inner, c.as_ptr()) };
            self._strings.push(c);
        }
    }

    /// Adds a boolean flag.
    pub fn add_bool(&mut self, name: &str, short_name: char, desc: &str, value: &mut bool) {
        self.add_flag(
            FlagType::Bool,
            name,
            short_name,
            desc,
            value as *mut bool as *mut _,
            false,
        );
    }

    /// Adds a string flag. The `value` must point to a `*const c_char` that will be set.
    pub fn add_string(
        &mut self,
        name: &str,
        short_name: char,
        desc: &str,
        value: &mut *const c_char,
    ) {
        self.add_flag(
            FlagType::String,
            name,
            short_name,
            desc,
            value as *mut _ as *mut _,
            false,
        );
    }

    /// Adds an i32 flag.
    pub fn add_i32(&mut self, name: &str, short_name: char, desc: &str, value: &mut i32) {
        self.add_flag(
            FlagType::Int32,
            name,
            short_name,
            desc,
            value as *mut i32 as *mut _,
            false,
        );
    }

    /// Adds a u32 flag.
    pub fn add_u32(&mut self, name: &str, short_name: char, desc: &str, value: &mut u32) {
        self.add_flag(
            FlagType::Uint32,
            name,
            short_name,
            desc,
            value as *mut u32 as *mut _,
            false,
        );
    }

    /// Adds an i64 flag.
    pub fn add_i64(&mut self, name: &str, short_name: char, desc: &str, value: &mut i64) {
        self.add_flag(
            FlagType::Int64,
            name,
            short_name,
            desc,
            value as *mut i64 as *mut _,
            false,
        );
    }

    /// Adds a f64 flag.
    pub fn add_f64(&mut self, name: &str, short_name: char, desc: &str, value: &mut f64) {
        self.add_flag(
            FlagType::Double,
            name,
            short_name,
            desc,
            value as *mut f64 as *mut _,
            false,
        );
    }

    /// Adds a required i32 flag.
    pub fn add_required_i32(&mut self, name: &str, short_name: char, desc: &str, value: &mut i32) {
        self.add_flag(
            FlagType::Int32,
            name,
            short_name,
            desc,
            value as *mut i32 as *mut _,
            true,
        );
    }

    /// Adds a required string flag.
    pub fn add_required_string(
        &mut self,
        name: &str,
        short_name: char,
        desc: &str,
        value: &mut *const c_char,
    ) {
        self.add_flag(
            FlagType::String,
            name,
            short_name,
            desc,
            value as *mut _ as *mut _,
            true,
        );
    }

    fn add_flag(
        &mut self,
        typ: FlagType,
        name: &str,
        short_name: char,
        desc: &str,
        value_ptr: *mut std::ffi::c_void,
        required: bool,
    ) {
        let name_c = CString::new(name).unwrap();
        let desc_c = CString::new(desc).unwrap();
        unsafe {
            ffi::flag_add(
                self.inner,
                typ.to_c(),
                name_c.as_ptr(),
                short_name as i8,
                desc_c.as_ptr(),
                value_ptr,
                required,
            );
        }

        // Keep CStrings alive for the lifetime of the parser
        self._strings.push(name_c);
        self._strings.push(desc_c);
    }

    /// Parses command-line arguments.
    pub fn parse(&mut self, args: &[&str]) -> Result<(), FlagStatus> {
        let c_args: Vec<CString> = args.iter().filter_map(|s| CString::new(*s).ok()).collect();
        let mut ptrs: Vec<*mut c_char> = c_args.iter().map(|s| s.as_ptr() as *mut c_char).collect();

        let status = unsafe { ffi::flag_parse(self.inner, ptrs.len() as i32, ptrs.as_mut_ptr()) };

        // Keep CStrings alive since the parser may reference them for positional args
        self._strings.extend(c_args);

        let status = FlagStatus::from(status);
        if status == FlagStatus::Ok {
            Ok(())
        } else {
            Err(status)
        }
    }

    /// Gets the error message from the last parse failure.
    pub fn error(&self) -> Option<String> {
        let ptr = unsafe { ffi::flag_get_error(self.inner) };
        if ptr.is_null() {
            None
        } else {
            Some(
                unsafe { CStr::from_ptr(ptr) }
                    .to_string_lossy()
                    .into_owned(),
            )
        }
    }

    /// Checks if a flag was explicitly provided.
    pub fn is_present(&self, name: &str) -> bool {
        let Ok(c) = CString::new(name) else {
            return false;
        };
        unsafe { ffi::flag_is_present(self.inner, c.as_ptr()) }
    }

    /// Gets the number of positional arguments.
    pub fn positional_count(&self) -> usize {
        unsafe { ffi::flag_positional_count(self.inner) as usize }
    }

    /// Gets a positional argument by index.
    pub fn positional_at(&self, index: usize) -> Option<String> {
        let ptr = unsafe { ffi::flag_positional_at(self.inner, index as i32) };
        if ptr.is_null() {
            None
        } else {
            Some(
                unsafe { CStr::from_ptr(ptr) }
                    .to_string_lossy()
                    .into_owned(),
            )
        }
    }

    /// Prints the auto-generated help message.
    pub fn print_usage(&self) {
        unsafe { ffi::flag_print_usage(self.inner) };
    }
}

impl Drop for FlagParser {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe { ffi::flag_parser_free(self.inner) };
        }
    }
}

unsafe impl Send for FlagParser {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_flags() {
        let mut parser = FlagParser::new("test", "A test app").unwrap();
        let mut verbose = false;
        let mut count: i32 = 0;
        parser.add_bool("verbose", 'v', "Enable verbose mode", &mut verbose);
        parser.add_i32("count", 'c', "Set count", &mut count);

        let result = parser.parse(&["test", "--verbose", "--count", "42"]);
        assert_eq!(result, Ok(()));
        assert!(verbose);
        assert_eq!(count, 42);
    }

    #[test]
    fn test_short_flags() {
        let mut parser = FlagParser::new("test", "A test app").unwrap();
        let mut verbose = false;
        parser.add_bool("verbose", 'v', "Verbose", &mut verbose);

        let result = parser.parse(&["test", "-v"]);
        assert_eq!(result, Ok(()));
        assert!(verbose);
    }

    #[test]
    fn test_positional_args() {
        let mut parser = FlagParser::new("test", "A test app").unwrap();
        let mut verbose = false;
        parser.add_bool("verbose", 'v', "Verbose", &mut verbose);

        let result = parser.parse(&["test", "--verbose", "file1.txt", "file2.txt"]);
        assert_eq!(result, Ok(()));
        assert_eq!(parser.positional_count(), 2);
        assert_eq!(parser.positional_at(0), Some("file1.txt".to_string()));
        assert_eq!(parser.positional_at(1), Some("file2.txt".to_string()));
    }

    #[test]
    fn test_is_present() {
        let mut parser = FlagParser::new("test", "A test app").unwrap();
        let mut count: i32 = 0;
        parser.add_i32("count", 'c', "Count", &mut count);

        parser.parse(&["test", "--count", "5"]).unwrap();
        assert!(parser.is_present("count"));
        assert!(!parser.is_present("verbose"));
    }
}

use std::ffi::CString;
use std::fmt;

use crate::ffi;

/// Errors for string-to-number conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    /// The value overflowed the target type.
    Overflow,
    /// The value underflowed the target type.
    Underflow,
    /// The string is not a valid number.
    Invalid,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Overflow => write!(f, "overflow"),
            ParseError::Underflow => write!(f, "underflow"),
            ParseError::Invalid => write!(f, "invalid number"),
        }
    }
}

impl std::error::Error for ParseError {}

fn from_sto_error(err: ffi::StoError) -> Result<(), ParseError> {
    match err {
        ffi::StoError::STO_SUCCESS => Ok(()),
        ffi::StoError::STO_OVERFLOW => Err(ParseError::Overflow),
        ffi::StoError::STO_UNDERFLOW => Err(ParseError::Underflow),
        _ => Err(ParseError::Invalid),
    }
}

macro_rules! impl_parse {
    ($name:ident, $ffi_fn:ident, $ty:ty) => {
        pub fn $name(s: &str) -> Result<$ty, ParseError> {
            let Ok(cstr) = CString::new(s) else {
                return Err(ParseError::Invalid);
            };
            let mut result: $ty = 0 as $ty;
            from_sto_error(unsafe { ffi::$ffi_fn(cstr.as_ptr(), &mut result) })?;
            Ok(result)
        }
    };
}

/// String-to-number conversion functions with overflow checking.
pub mod parse {
    use super::*;

    impl_parse!(u8, str_to_u8, u8);
    impl_parse!(i8, str_to_i8, i8);
    impl_parse!(u16, str_to_u16, u16);
    impl_parse!(i16, str_to_i16, i16);
    impl_parse!(u32, str_to_u32, u32);
    impl_parse!(i32, str_to_i32, i32);
    impl_parse!(u64, str_to_u64, u64);
    impl_parse!(i64, str_to_i64, i64);

    pub fn f32(s: &str) -> Result<f32, ParseError> {
        let Ok(cstr) = CString::new(s) else {
            return Err(ParseError::Invalid);
        };
        let mut result: f32 = 0.0;
        from_sto_error(unsafe { ffi::str_to_float(cstr.as_ptr(), &mut result) })?;
        Ok(result)
    }

    pub fn f64(s: &str) -> Result<f64, ParseError> {
        let Ok(cstr) = CString::new(s) else {
            return Err(ParseError::Invalid);
        };
        let mut result: f64 = 0.0;
        from_sto_error(unsafe { ffi::str_to_double(cstr.as_ptr(), &mut result) })?;
        Ok(result)
    }

    pub fn bool(s: &str) -> Result<bool, ParseError> {
        let Ok(cstr) = CString::new(s) else {
            return Err(ParseError::Invalid);
        };
        let mut result: bool = false;
        from_sto_error(unsafe { ffi::str_to_bool(cstr.as_ptr(), &mut result) })?;
        Ok(result)
    }

    /// Parses an integer with a specific base (2-36).
    pub fn i32_base(s: &str, base: i32) -> Result<i32, ParseError> {
        let Ok(cstr) = CString::new(s) else {
            return Err(ParseError::Invalid);
        };
        let mut result: i32 = 0;
        from_sto_error(unsafe { ffi::str_to_int_base(cstr.as_ptr(), base, &mut result) })?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn test_parse_u32() {
        assert_eq!(parse::u32("42").unwrap(), 42);
        assert_eq!(parse::u32("0").unwrap(), 0);
    }

    #[test]
    fn test_parse_i32() {
        assert_eq!(parse::i32("-42").unwrap(), -42);
        assert_eq!(parse::i32("100").unwrap(), 100);
    }

    #[test]
    fn test_parse_f64() {
        let val = parse::f64("3.14").unwrap();
        assert!((val - 3.14).abs() < 1e-10);
    }

    #[test]
    fn test_parse_bool() {
        assert!(parse::bool("true").unwrap());
        assert!(!parse::bool("false").unwrap());
    }

    #[test]
    fn test_parse_overflow() {
        let result = parse::u8("256");
        assert_eq!(result, Err(super::ParseError::Overflow));
    }

    #[test]
    fn test_parse_invalid() {
        let result = parse::i32("abc");
        assert_eq!(result, Err(super::ParseError::Invalid));
    }

    #[test]
    fn test_parse_base() {
        assert_eq!(parse::i32_base("FF", 16).unwrap(), 255);
        assert_eq!(parse::i32_base("1010", 2).unwrap(), 10);
    }

    #[test]
    fn test_parse_u64() {
        assert_eq!(parse::u64("18446744073709551615").unwrap(), u64::MAX);
    }
}

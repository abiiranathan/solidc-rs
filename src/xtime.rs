use std::ffi::{CStr, CString};
use std::fmt;

use crate::ffi;

/// Error type for time operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeError {
    InvalidArg,
    ParseFailed,
    DateOutOfRange,
    BufferTooSmall,
    InvalidTime,
    SystemError,
}

impl fmt::Display for TimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimeError::InvalidArg => write!(f, "invalid argument"),
            TimeError::ParseFailed => write!(f, "parse failed"),
            TimeError::DateOutOfRange => write!(f, "date out of range"),
            TimeError::BufferTooSmall => write!(f, "buffer too small"),
            TimeError::InvalidTime => write!(f, "invalid time"),
            TimeError::SystemError => write!(f, "system error"),
        }
    }
}

impl std::error::Error for TimeError {}

fn from_xtime_error(err: ffi::xtime_error_t) -> Result<(), TimeError> {
    match err {
        ffi::xtime_error_t::XTIME_OK => Ok(()),
        ffi::xtime_error_t::XTIME_ERR_INVALID_ARG => Err(TimeError::InvalidArg),
        ffi::xtime_error_t::XTIME_ERR_PARSE_FAILED => Err(TimeError::ParseFailed),
        ffi::xtime_error_t::XTIME_ERR_DATE_OUT_OF_RANGE => Err(TimeError::DateOutOfRange),
        ffi::xtime_error_t::XTIME_ERR_BUFFER_TOO_SMALL => Err(TimeError::BufferTooSmall),
        ffi::xtime_error_t::XTIME_ERR_INVALID_TIME => Err(TimeError::InvalidTime),
        _ => Err(TimeError::SystemError),
    }
}

/// An extended time type with nanosecond precision and timezone support.
#[derive(Clone)]
pub struct Time {
    inner: ffi::xtime_t,
}

impl Time {
    /// Creates a new zeroed time.
    pub fn new() -> Result<Self, TimeError> {
        let mut t: ffi::xtime_t = unsafe { std::mem::zeroed() };
        from_xtime_error(unsafe { ffi::xtime_init(&mut t) })?;
        Ok(Time { inner: t })
    }

    /// Returns the current local time.
    pub fn now() -> Result<Self, TimeError> {
        let mut t: ffi::xtime_t = unsafe { std::mem::zeroed() };
        from_xtime_error(unsafe { ffi::xtime_now(&mut t) })?;
        Ok(Time { inner: t })
    }

    /// Returns the current UTC time.
    pub fn utc_now() -> Result<Self, TimeError> {
        let mut t: ffi::xtime_t = unsafe { std::mem::zeroed() };
        from_xtime_error(unsafe { ffi::xtime_utc_now(&mut t) })?;
        Ok(Time { inner: t })
    }

    /// Creates a Time from a Unix timestamp.
    pub fn from_unix(timestamp: i64) -> Result<Self, TimeError> {
        let mut t: ffi::xtime_t = unsafe { std::mem::zeroed() };
        from_xtime_error(unsafe { ffi::xtime_from_unix(timestamp, &mut t) })?;
        Ok(Time { inner: t })
    }

    /// Parses a time string with the given format (strftime-style).
    pub fn parse(s: &str, format: &str) -> Result<Self, TimeError> {
        let Ok(s_c) = CString::new(s) else {
            return Err(TimeError::InvalidArg);
        };
        let Ok(fmt_c) = CString::new(format) else {
            return Err(TimeError::InvalidArg);
        };
        let mut t: ffi::xtime_t = unsafe { std::mem::zeroed() };
        from_xtime_error(unsafe { ffi::xtime_parse(s_c.as_ptr(), fmt_c.as_ptr(), &mut t) })?;
        Ok(Time { inner: t })
    }

    /// Formats the time using a strftime-style format string.
    pub fn format(&self, format: &str) -> Result<String, TimeError> {
        let Ok(fmt_c) = CString::new(format) else {
            return Err(TimeError::InvalidArg);
        };
        let mut buf = vec![0u8; 256];
        from_xtime_error(unsafe {
            ffi::xtime_format(
                &self.inner,
                fmt_c.as_ptr(),
                buf.as_mut_ptr() as *mut libc::c_char,
                buf.len(),
            )
        })?;
        let cstr = unsafe { CStr::from_ptr(buf.as_ptr() as *const libc::c_char) };
        Ok(cstr.to_string_lossy().into_owned())
    }

    /// Returns the Unix timestamp.
    pub fn to_unix(&self) -> i64 {
        unsafe { ffi::xtime_to_unix(&self.inner) }
    }

    /// Formats the time as an ISO 8601 JSON string.
    pub fn to_json(&self) -> Result<String, TimeError> {
        let mut buf = vec![0u8; 64];
        from_xtime_error(unsafe {
            ffi::xtime_to_json(
                &self.inner,
                buf.as_mut_ptr() as *mut libc::c_char,
                buf.len(),
            )
        })?;
        let cstr = unsafe { CStr::from_ptr(buf.as_ptr() as *const libc::c_char) };
        Ok(cstr.to_string_lossy().into_owned())
    }

    /// Adds seconds to the time.
    pub fn add_seconds(&mut self, seconds: i64) -> Result<(), TimeError> {
        from_xtime_error(unsafe { ffi::xtime_add_seconds(&mut self.inner, seconds) })
    }

    /// Adds milliseconds to the time.
    pub fn add_millis(&mut self, millis: i64) -> Result<(), TimeError> {
        from_xtime_error(unsafe { ffi::xtime_add_milliseconds(&mut self.inner, millis) })
    }

    /// Adds minutes to the time.
    pub fn add_minutes(&mut self, minutes: i64) -> Result<(), TimeError> {
        from_xtime_error(unsafe { ffi::xtime_add_minutes(&mut self.inner, minutes) })
    }

    /// Adds hours to the time.
    pub fn add_hours(&mut self, hours: i64) -> Result<(), TimeError> {
        from_xtime_error(unsafe { ffi::xtime_add_hours(&mut self.inner, hours) })
    }

    /// Adds days to the time.
    pub fn add_days(&mut self, days: i64) -> Result<(), TimeError> {
        from_xtime_error(unsafe { ffi::xtime_add_days(&mut self.inner, days) })
    }

    /// Adds months to the time.
    pub fn add_months(&mut self, months: i32) -> Result<(), TimeError> {
        from_xtime_error(unsafe { ffi::xtime_add_months(&mut self.inner, months) })
    }

    /// Adds years to the time.
    pub fn add_years(&mut self, years: i32) -> Result<(), TimeError> {
        from_xtime_error(unsafe { ffi::xtime_add_years(&mut self.inner, years) })
    }

    /// Compares two times.
    /// Returns Ordering::Less if self < other,
    /// Ordering::Equal if self == other, and Ordering::Greater if self > other.
    pub fn compare(&self, other: &Time) -> std::cmp::Ordering {
        let result = unsafe { ffi::xtime_compare(&self.inner, &other.inner) };
        result.cmp(&0)
    }

    /// Returns the difference in seconds between two times.
    pub fn diff_seconds(&self, other: &Time) -> Result<i64, TimeError> {
        let mut out: i64 = 0;
        from_xtime_error(unsafe { ffi::xtime_diff_seconds(&self.inner, &other.inner, &mut out) })?;
        Ok(out)
    }

    /// Returns the difference in days between two times.
    pub fn diff_days(&self, other: &Time) -> Result<i64, TimeError> {
        let mut out: i64 = 0;
        from_xtime_error(unsafe { ffi::xtime_diff_days(&self.inner, &other.inner, &mut out) })?;
        Ok(out)
    }

    /// Returns true if the time's year is a leap year.
    pub fn is_leap_year(&self) -> bool {
        unsafe { ffi::xtime_is_leap_year(&self.inner) }
    }

    /// Returns the nanoseconds component.
    pub fn nanoseconds(&self) -> u32 {
        self.inner.nanoseconds
    }

    /// Returns the timezone offset in minutes from UTC.
    pub fn tz_offset(&self) -> i16 {
        self.inner.tz_offset
    }

    /// Returns whether timezone info is present.
    pub fn has_tz(&self) -> bool {
        self.inner.has_tz
    }
}

impl PartialEq for Time {
    fn eq(&self, other: &Self) -> bool {
        self.compare(other) == std::cmp::Ordering::Equal
    }
}

impl Eq for Time {}

impl PartialOrd for Time {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.compare(other))
    }
}

impl Ord for Time {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.compare(other)
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.format("%Y-%m-%d %H:%M:%S") {
            Ok(s) => write!(f, "{}", s),
            Err(_) => write!(f, "<invalid time>"),
        }
    }
}

impl fmt::Debug for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Time")
            .field("unix", &self.to_unix())
            .field("ns", &self.nanoseconds())
            .field("tz_offset", &self.tz_offset())
            .finish()
    }
}

impl Default for Time {
    fn default() -> Self {
        Time::now().expect("Failed to get current time")
    }
}

/// Returns the error message for a time error code.
pub fn time_error_string(err: TimeError) -> &'static str {
    match err {
        TimeError::InvalidArg => "invalid argument",
        TimeError::ParseFailed => "parse failed",
        TimeError::DateOutOfRange => "date out of range",
        TimeError::BufferTooSmall => "buffer too small",
        TimeError::InvalidTime => "invalid time",
        TimeError::SystemError => "system error",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_now() {
        let t = Time::now().unwrap();
        assert!(t.to_unix() > 0);
    }

    #[test]
    fn test_utc_now() {
        let t = Time::utc_now().unwrap();
        assert!(t.to_unix() > 0);
    }

    #[test]
    fn test_from_unix() {
        let t = Time::from_unix(1700000000).unwrap();
        assert_eq!(t.to_unix(), 1700000000);
    }

    #[test]
    fn test_format() {
        let t = Time::from_unix(0).unwrap();
        let formatted = t.format("%Y").unwrap();
        assert_eq!(formatted, "1970");
    }

    #[test]
    fn test_add_seconds() {
        let mut t = Time::from_unix(1000).unwrap();
        t.add_seconds(500).unwrap();
        assert_eq!(t.to_unix(), 1500);
    }

    #[test]
    fn test_compare() {
        let t1 = Time::from_unix(1000).unwrap();
        let t2 = Time::from_unix(2000).unwrap();
        assert!(t1 < t2);
        assert!(t2 > t1);
        assert_eq!(t1, t1);
    }

    #[test]
    fn test_diff() {
        let t1 = Time::from_unix(1000).unwrap();
        let t2 = Time::from_unix(2000).unwrap();
        let diff = t1.diff_seconds(&t2).unwrap();
        assert_eq!(diff.abs(), 1000);
    }

    #[test]
    fn test_display() {
        let t = Time::now().unwrap();
        let s = format!("{}", t);
        assert!(!s.is_empty());
        assert_ne!(s, "<invalid time>");
    }

    #[test]
    fn test_to_json() {
        let t = Time::now().unwrap();
        let json = t.to_json().unwrap();
        assert!(!json.is_empty());
    }
}

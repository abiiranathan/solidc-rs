use std::ffi::CString;

use crate::ffi;

/// Loads environment variables from a `.env` file.
///
/// Variables are set in the process environment and can be accessed
/// with `std::env::var()`.
///
/// # Arguments
/// * `path` - Path to the .env file
///
/// # Returns
/// `true` if the file was loaded successfully.
pub fn load_dotenv(path: &str) -> bool {
    let Ok(cstr) = CString::new(path) else {
        return false;
    };
    unsafe { ffi::load_dotenv(cstr.as_ptr()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_load_dotenv() {
        // Create a temporary .env file
        let path = "/tmp/test_solidc_dotenv";
        {
            let mut f = std::fs::File::create(path).unwrap();
            writeln!(f, "SOLIDC_TEST_VAR=hello_from_dotenv").unwrap();
            writeln!(f, "SOLIDC_TEST_NUM=42").unwrap();
        }

        assert!(load_dotenv(path));
        assert_eq!(
            std::env::var("SOLIDC_TEST_VAR").unwrap(),
            "hello_from_dotenv"
        );
        assert_eq!(std::env::var("SOLIDC_TEST_NUM").unwrap(), "42");

        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_load_dotenv_nonexistent() {
        assert!(!load_dotenv("/tmp/nonexistent_dotenv_file"));
    }
}

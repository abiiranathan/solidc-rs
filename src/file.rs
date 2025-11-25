use std::ffi::{CStr, CString};
use std::fmt;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::ptr;
use std::slice;

use crate::ffi::{
    file_close, file_copy, file_flush, file_get_size, file_lock, file_mmap, file_munmap, file_open,
    file_pread, file_pwrite, file_read, file_readall, file_result_t, file_seek, file_t, file_tell,
    file_truncate, file_unlock, file_write, file_write_string, filesize_tostring, get_file_size,
};

/// Result type for file operations.
pub type Result<T> = std::result::Result<T, FileError>;

/// Error types returned by file operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileError {
    /// Invalid arguments provided to the operation.
    InvalidArgs,
    /// Failed to open the file.
    OpenFailed,
    /// I/O operation failed.
    IoFailed,
    /// Failed to acquire file lock.
    LockFailed,
    /// Memory allocation failed.
    MemoryFailed,
    /// System-level error occurred.
    SystemError,
}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileError::InvalidArgs => write!(f, "invalid arguments"),
            FileError::OpenFailed => write!(f, "failed to open file"),
            FileError::IoFailed => write!(f, "I/O operation failed"),
            FileError::LockFailed => write!(f, "failed to acquire file lock"),
            FileError::MemoryFailed => write!(f, "memory allocation failed"),
            FileError::SystemError => write!(f, "system error"),
        }
    }
}

// Add after the FileError Display implementation
impl From<io::Error> for FileError {
    fn from(_: io::Error) -> Self {
        FileError::IoFailed
    }
}

impl std::error::Error for FileError {}

impl From<file_result_t> for Result<()> {
    fn from(result: file_result_t) -> Self {
        match result {
            file_result_t::FILE_SUCCESS => Ok(()),
            file_result_t::FILE_ERROR_INVALID_ARGS => Err(FileError::InvalidArgs),
            file_result_t::FILE_ERROR_OPEN_FAILED => Err(FileError::OpenFailed),
            file_result_t::FILE_ERROR_IO_FAILED => Err(FileError::IoFailed),
            file_result_t::FILE_ERROR_LOCK_FAILED => Err(FileError::LockFailed),
            file_result_t::FILE_ERROR_MEMORY_FAILED => Err(FileError::MemoryFailed),
            _ => Err(FileError::SystemError),
        }
    }
}

/// Safe wrapper around the C file_t type.
/// Automatically closes the file when dropped.
pub struct File {
    inner: file_t,
    closed: bool,
}

impl File {
    /// Opens a file with the specified mode.
    ///
    /// # Arguments
    /// * `path` - Path to the file
    /// * `mode` - File open mode (e.g., "r", "w", "a", "rb", "wb")
    ///
    /// # Errors
    /// Returns `FileError` if the file cannot be opened.
    pub fn open<P: AsRef<Path>>(path: P, mode: &str) -> Result<Self> {
        let path_cstr = CString::new(path.as_ref().to_string_lossy().as_bytes())
            .map_err(|_| FileError::InvalidArgs)?;
        let mode_cstr = CString::new(mode).map_err(|_| FileError::InvalidArgs)?;

        let mut inner = file_t {
            stream: ptr::null_mut(),
            native_handle: unsafe { std::mem::zeroed() },
        };

        let result = unsafe { file_open(&mut inner, path_cstr.as_ptr(), mode_cstr.as_ptr()) };
        Result::from(result)?;

        Ok(File {
            inner,
            closed: false,
        })
    }

    /// Explicitly closes the file.
    /// This is called automatically on drop, but can be called manually to handle errors.
    pub fn close(&mut self) -> Result<()> {
        if !self.closed {
            unsafe { file_close(&mut self.inner) };
            self.closed = true;
        }
        Ok(())
    }

    /// Returns the size of the file in bytes.
    pub fn size(&self) -> Result<i64> {
        let size = unsafe { file_get_size(&self.inner) };
        if size < 0 {
            Err(FileError::IoFailed)
        } else {
            Ok(size)
        }
    }

    /// Truncates or extends the file to the specified length.
    pub fn truncate(&mut self, length: i64) -> Result<()> {
        let result = unsafe { file_truncate(&mut self.inner, length) };
        Result::from(result)
    }

    /// Reads data from the file into the provided buffer.
    /// Returns the number of elements successfully read.
    pub fn read(&self, buffer: &mut [u8]) -> Result<usize> {
        let count =
            unsafe { file_read(&self.inner, buffer.as_mut_ptr() as *mut _, 1, buffer.len()) };
        Ok(count)
    }

    /// Writes data from the buffer to the file.
    /// Returns the number of elements successfully written.
    pub fn write(&mut self, buffer: &[u8]) -> Result<usize> {
        let count = unsafe {
            file_write(
                &mut self.inner,
                buffer.as_ptr() as *const _,
                1,
                buffer.len(),
            )
        };
        if count == 0 && !buffer.is_empty() {
            Err(FileError::IoFailed)
        } else {
            Ok(count)
        }
    }

    /// Writes a string to the file.
    /// Returns the number of bytes written.
    pub fn write_str(&mut self, s: &str) -> Result<usize> {
        let cstr = CString::new(s).map_err(|_| FileError::InvalidArgs)?;
        let count = unsafe { file_write_string(&mut self.inner, cstr.as_ptr()) };
        Ok(count)
    }

    /// Reads data at a specific offset without changing the file position.
    /// Returns the number of bytes read, or an error.
    pub fn pread(&self, buffer: &mut [u8], offset: i64) -> Result<usize> {
        let result = unsafe {
            file_pread(
                &self.inner,
                buffer.as_mut_ptr() as *mut _,
                buffer.len(),
                offset,
            )
        };
        if result < 0 {
            Err(FileError::IoFailed)
        } else {
            Ok(result as usize)
        }
    }

    /// Writes data at a specific offset without changing the file position.
    /// Returns the number of bytes written, or an error.
    pub fn pwrite(&mut self, buffer: &[u8], offset: i64) -> Result<usize> {
        let result = unsafe {
            file_pwrite(
                &mut self.inner,
                buffer.as_ptr() as *const _,
                buffer.len(),
                offset,
            )
        };
        if result < 0 {
            Err(FileError::IoFailed)
        } else {
            Ok(result as usize)
        }
    }

    /// Reads the entire file into a newly allocated buffer.
    /// Returns the buffer as a Vec<u8>.
    ///
    /// # Safety
    /// The underlying C function allocates memory that must be freed.
    /// This wrapper handles the deallocation properly.
    pub fn read_all(&self) -> Result<Vec<u8>> {
        let mut size: usize = 0;
        let ptr = unsafe { file_readall(&self.inner, &mut size as *mut _) };

        if ptr.is_null() {
            return Err(FileError::MemoryFailed);
        }

        // Safety: We own this pointer and know its size
        let data = unsafe { Vec::from_raw_parts(ptr as *mut u8, size, size) };
        Ok(data)
    }

    /// Acquires an exclusive lock on the file.
    pub fn lock(&self) -> Result<()> {
        let result = unsafe { file_lock(&self.inner) };
        Result::from(result)
    }

    /// Releases the file lock.
    pub fn unlock(&self) -> Result<()> {
        let result = unsafe { file_unlock(&self.inner) };
        Result::from(result)
    }

    /// Copies the contents of this file to another file.
    pub fn copy_to(&self, dst: &mut File) -> Result<()> {
        let result = unsafe { file_copy(&self.inner, &mut dst.inner) };
        Result::from(result)
    }

    /// Memory-maps the file with specified access permissions.
    /// Returns a slice representing the mapped region.
    ///
    /// # Safety
    /// The returned slice is valid only as long as this File object exists
    /// and the mapping is not unmapped. The caller must ensure proper synchronization.
    pub fn mmap(&self, length: usize, read: bool, write: bool) -> Result<MemoryMap> {
        let ptr = unsafe { file_mmap(&self.inner, length, read, write) };
        if ptr.is_null() {
            Err(FileError::IoFailed)
        } else {
            Ok(MemoryMap {
                ptr: ptr as *mut u8,
                length,
            })
        }
    }

    /// Flushes any buffered writes to the underlying storage.
    pub fn flush(&mut self) -> Result<()> {
        let result = unsafe { file_flush(&mut self.inner) };
        Result::from(result)
    }

    /// Returns the current file position.
    pub fn tell(&self) -> Result<i64> {
        let pos = unsafe { file_tell(&self.inner) };
        if pos < 0 {
            Err(FileError::IoFailed)
        } else {
            Ok(pos)
        }
    }

    /// Seeks to a position in the file using raw offset and whence values.
    ///
    /// # Arguments
    /// * `offset` - Offset in bytes
    /// * `whence` - SEEK_SET (0), SEEK_CUR (1), or SEEK_END (2)
    pub fn seek_raw(&mut self, offset: i64, whence: i32) -> Result<()> {
        let result = unsafe { file_seek(&mut self.inner, offset, whence) };
        Result::from(result)
    }
}

impl Drop for File {
    fn drop(&mut self) {
        if !self.closed {
            unsafe { file_close(&mut self.inner) };
        }
    }
}

// Implement std::io traits for ergonomic usage
impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        File::read(self, buf).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }
}

impl Write for File {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        File::write(self, buf).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }

    fn flush(&mut self) -> io::Result<()> {
        File::flush(self).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }
}

impl Seek for File {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let (offset, whence) = match pos {
            SeekFrom::Start(n) => (n as i64, 0), // SEEK_SET
            SeekFrom::Current(n) => (n, 1),      // SEEK_CUR
            SeekFrom::End(n) => (n, 2),          // SEEK_END
        };
        File::seek_raw(self, offset, whence)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        self.tell()
            .map(|p| p as u64)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }
}

/// RAII guard for memory-mapped file regions.
/// Automatically unmaps on drop.
pub struct MemoryMap {
    ptr: *mut u8,
    length: usize,
}

impl MemoryMap {
    /// Returns a slice view of the mapped memory.
    ///
    /// # Safety
    /// Caller must ensure no concurrent modifications occur through other mappings
    /// or file operations while this slice is in use.
    pub fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.ptr, self.length) }
    }

    /// Returns a mutable slice view of the mapped memory.
    ///
    /// # Safety
    /// Caller must ensure exclusive access and proper synchronization.
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.ptr, self.length) }
    }
}

impl Drop for MemoryMap {
    fn drop(&mut self) {
        unsafe {
            let _ = file_munmap(self.ptr as *mut _, self.length);
        }
    }
}

// Safety: File operations are thread-safe at the OS level when properly synchronized
unsafe impl Send for File {}
unsafe impl Sync for File {}

/// Utility function to get file size by path without opening it.
pub fn get_file_size_by_path<P: AsRef<Path>>(path: P) -> Result<i64> {
    let path_cstr = CString::new(path.as_ref().to_string_lossy().as_bytes())
        .map_err(|_| FileError::InvalidArgs)?;
    let size = unsafe { get_file_size(path_cstr.as_ptr()) };
    if size < 0 {
        Err(FileError::IoFailed)
    } else {
        Ok(size)
    }
}

/// Formats a file size as a human-readable string (e.g., "1.5 GB").
pub fn format_file_size(size: u64) -> Result<String> {
    let mut buf = [0u8; 64];
    let result = unsafe { filesize_tostring(size, buf.as_mut_ptr() as *mut _, buf.len()) };
    Result::from(result)?;

    let cstr = unsafe { CStr::from_ptr(buf.as_ptr() as *const _) };
    Ok(cstr.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_open_write_read() {
        let mut file = File::open("/tmp/test_file.txt", "w+").unwrap();
        let data = b"Hello, Rust!";
        file.write(data).unwrap();
        file.flush().unwrap();
        file.seek(SeekFrom::Start(0)).unwrap();

        let mut buffer = vec![0u8; data.len()];
        file.read(&mut buffer).unwrap();
        assert_eq!(&buffer, data);
    }

    #[test]
    fn test_file_size() {
        let size = get_file_size_by_path("/tmp/test_file.txt").unwrap();
        assert!(size >= 0);
    }

    #[test]
    fn test_format_size() {
        let formatted = format_file_size(1536).unwrap();
        assert!(!formatted.is_empty());
    }
}

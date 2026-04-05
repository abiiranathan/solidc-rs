use std::ffi::{CStr, CString};
use std::fmt;
use std::path::{Path, PathBuf};

use crate::ffi::{
    Directory, WalkDirOption, dir_chdir, dir_close, dir_create, dir_list, dir_next, dir_open,
    dir_remove, dir_rename, dir_size, dir_walk, dir_walk_depth_first, filepath_absolute,
    filepath_basename, filepath_dirname, filepath_expanduser, filepath_extension, filepath_join,
    filepath_makedirs, filepath_nameonly, filepath_remove, filepath_rename, filepath_split,
    get_cwd, get_tempdir, is_dir as is_directory, is_file as is_a_file,
    is_symlink as is_symbolic_link, make_tempdir, make_tempfile, path_exists, user_home_dir,
};

/// Result type for directory and filepath operations.
pub type Result<T> = std::result::Result<T, PathError>;

/// Error types returned by directory and filepath operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathError {
    /// Invalid path or arguments provided.
    InvalidPath,
    /// Path does not exist.
    NotFound,
    /// Operation failed (e.g., create, remove, rename).
    OperationFailed,
    /// Memory allocation failed.
    MemoryFailed,
    /// Path encoding error (invalid UTF-8).
    EncodingError,
}

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathError::InvalidPath => write!(f, "invalid path"),
            PathError::NotFound => write!(f, "path not found"),
            PathError::OperationFailed => write!(f, "operation failed"),
            PathError::MemoryFailed => write!(f, "memory allocation failed"),
            PathError::EncodingError => write!(f, "path encoding error"),
        }
    }
}

impl std::error::Error for PathError {}

/// Safe wrapper around the C Directory type.
/// Automatically closes the directory when dropped.
pub struct Dir {
    inner: *mut Directory,
}

impl Dir {
    /// Opens a directory for reading.
    ///
    /// # Errors
    /// Returns `PathError` if the directory cannot be opened.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_cstr = path_to_cstring(path.as_ref())?;
        let inner = unsafe { dir_open(path_cstr.as_ptr()) };

        if inner.is_null() {
            Err(PathError::NotFound)
        } else {
            Ok(Dir { inner })
        }
    }

    /// Reads the next entry in the directory.
    /// Returns `None` when no more entries are available.
    ///
    /// # Note
    /// Skips "." and ".." entries automatically (handled by C implementation).
    pub fn next(&mut self) -> Option<String> {
        let entry_ptr = unsafe { dir_next(self.inner) };
        if entry_ptr.is_null() {
            None
        } else {
            let cstr = unsafe { CStr::from_ptr(entry_ptr) };
            let result = cstr.to_string_lossy().into_owned();
            // The C function returns a pointer that we own, so we need to free it
            unsafe { libc::free(entry_ptr as *mut libc::c_void) };
            Some(result)
        }
    }

    /// Collects all entries in the directory into a Vec.
    pub fn entries(&mut self) -> Vec<String> {
        let mut entries = Vec::new();
        while let Some(entry) = self.next() {
            entries.push(entry);
        }
        entries
    }
}

impl Drop for Dir {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe { dir_close(self.inner) };
        }
    }
}

impl Iterator for Dir {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        Dir::next(self)
    }
}

// Safety: Directory operations are thread-safe at the OS level
unsafe impl Send for Dir {}
unsafe impl Sync for Dir {}

/// Directory operations module.
pub mod directory {
    use super::*;

    /// Creates a directory.
    ///
    /// # Errors
    /// Returns `PathError` if the directory cannot be created.
    pub fn create<P: AsRef<Path>>(path: P) -> Result<()> {
        let path_cstr = path_to_cstring(path.as_ref())?;
        let result = unsafe { dir_create(path_cstr.as_ptr()) };
        if result == 0 {
            Ok(())
        } else {
            Err(PathError::OperationFailed)
        }
    }

    /// Creates a directory and all parent directories (like `mkdir -p`).
    ///
    /// # Errors
    /// Returns `PathError` if any directory in the path cannot be created.
    pub fn create_all<P: AsRef<Path>>(path: P) -> Result<()> {
        let path_cstr = path_to_cstring(path.as_ref())?;
        let success = unsafe { filepath_makedirs(path_cstr.as_ptr()) };
        if success {
            Ok(())
        } else {
            Err(PathError::OperationFailed)
        }
    }

    /// Removes a directory.
    ///
    /// # Arguments
    /// * `path` - Path to the directory
    /// * `recursive` - If true, removes directory and all its contents
    ///
    /// # Errors
    /// Returns `PathError` if the directory cannot be removed.
    pub fn remove<P: AsRef<Path>>(path: P, recursive: bool) -> Result<()> {
        let path_cstr = path_to_cstring(path.as_ref())?;
        let result = unsafe { dir_remove(path_cstr.as_ptr(), recursive) };
        if result == 0 {
            Ok(())
        } else {
            Err(PathError::OperationFailed)
        }
    }

    /// Renames a directory.
    ///
    /// # Errors
    /// Returns `PathError` if the directory cannot be renamed.
    pub fn rename<P: AsRef<Path>>(oldpath: P, newpath: P) -> Result<()> {
        let old_cstr = path_to_cstring(oldpath.as_ref())?;
        let new_cstr = path_to_cstring(newpath.as_ref())?;
        let result = unsafe { dir_rename(old_cstr.as_ptr(), new_cstr.as_ptr()) };
        if result == 0 {
            Ok(())
        } else {
            Err(PathError::OperationFailed)
        }
    }

    /// Changes the current working directory.
    ///
    /// # Errors
    /// Returns `PathError` if the directory cannot be changed.
    pub fn chdir<P: AsRef<Path>>(path: P) -> Result<()> {
        let path_cstr = path_to_cstring(path.as_ref())?;
        let result = unsafe { dir_chdir(path_cstr.as_ptr()) };
        if result == 0 {
            Ok(())
        } else {
            Err(PathError::OperationFailed)
        }
    }

    /// Lists all files in a directory recursively.
    /// Returns a vector of file paths.
    ///
    /// # Note
    /// This walks the directory tree recursively and may be slow for large directories.
    ///
    /// # Errors
    /// Returns `PathError` if the directory cannot be read.
    pub fn list<P: AsRef<Path>>(path: P) -> Result<Vec<String>> {
        let path_cstr = path_to_cstring(path.as_ref())?;
        let mut count: usize = 0;
        let list_ptr = unsafe { dir_list(path_cstr.as_ptr(), &mut count as *mut _) };

        if list_ptr.is_null() {
            return Err(PathError::OperationFailed);
        }

        let mut entries = Vec::with_capacity(count);
        for i in 0..count {
            let entry_ptr = unsafe { *list_ptr.add(i) };
            if !entry_ptr.is_null() {
                let cstr = unsafe { CStr::from_ptr(entry_ptr) };
                entries.push(cstr.to_string_lossy().into_owned());
                unsafe { libc::free(entry_ptr as *mut libc::c_void) };
            }
        }

        unsafe { libc::free(list_ptr as *mut libc::c_void) };
        Ok(entries)
    }

    /// Lists all contents of a directory and calls the provided callback for each entry.
    /// Skips "." and ".." to avoid infinite loops.
    pub fn list_with_callback<P, F>(path: P, mut callback: F) -> Result<()>
    where
        P: AsRef<Path>,
        F: FnMut(&str),
    {
        let entries = list(path)?;
        for entry in &entries {
            callback(entry);
        }
        Ok(())
    }

    /// Calculates the total size of a directory in bytes.
    ///
    /// # Note
    /// This walks the entire directory tree and may be slow for large directories.
    ///
    /// # Errors
    /// Returns `PathError` if the directory cannot be accessed.
    pub fn size<P: AsRef<Path>>(path: P) -> Result<i64> {
        let path_cstr = path_to_cstring(path.as_ref())?;
        let size = unsafe { dir_size(path_cstr.as_ptr()) };
        if size < 0 {
            Err(PathError::OperationFailed)
        } else {
            Ok(size as i64)
        }
    }
}

/// Walk result that controls directory traversal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WalkControl {
    /// Continue walking the directory recursively.
    Continue,
    /// Stop traversal and return immediately.
    Stop,
    /// Skip the current entry and continue traversal.
    Skip,
    /// An error occurred during traversal.
    Error,
}

impl From<WalkControl> for WalkDirOption {
    fn from(control: WalkControl) -> Self {
        match control {
            WalkControl::Continue => WalkDirOption::DirContinue,
            WalkControl::Stop => WalkDirOption::DirStop,
            WalkControl::Skip => WalkDirOption::DirSkip,
            WalkControl::Error => WalkDirOption::DirError,
        }
    }
}

impl From<WalkDirOption> for WalkControl {
    fn from(option: WalkDirOption) -> Self {
        match option {
            WalkDirOption::DirContinue => WalkControl::Continue,
            WalkDirOption::DirStop => WalkControl::Stop,
            WalkDirOption::DirSkip => WalkControl::Skip,
            _ => WalkControl::Error,
        }
    }
}

/// Walks a directory tree, calling the provided callback for each entry.
///
/// # Arguments
/// * `path` - Root directory to walk
/// * `callback` - Function called for each entry with (path, name) returning WalkControl
///
/// # Errors
/// Returns `PathError` if the walk fails.
pub fn walk<P, F>(path: P, mut callback: F) -> Result<()>
where
    P: AsRef<Path>,
    F: FnMut(&str, &str) -> WalkControl,
{
    let path_cstr = path_to_cstring(path.as_ref())?;

    unsafe extern "C" fn c_callback(
        _attr: *const crate::ffi::FileAttributes,
        path: *const libc::c_char,
        name: *const libc::c_char,
        data: *mut libc::c_void,
    ) -> WalkDirOption {
        if path.is_null() || name.is_null() || data.is_null() {
            return WalkDirOption::DirError;
        }

        let path_str = unsafe { CStr::from_ptr(path).to_str().unwrap_or("") };
        let name_str = unsafe { CStr::from_ptr(name).to_str().unwrap_or("") };

        let callback_ptr = data as *mut &mut dyn FnMut(&str, &str) -> WalkControl;
        let callback = unsafe { &mut *callback_ptr };
        let control = callback(path_str, name_str);
        control.into()
    }

    let mut callback_trait: &mut dyn FnMut(&str, &str) -> WalkControl = &mut callback;
    let callback_ptr =
        &mut callback_trait as *mut &mut dyn FnMut(&str, &str) -> WalkControl as *mut libc::c_void;

    let result = unsafe { dir_walk(path_cstr.as_ptr(), Some(c_callback), callback_ptr) };

    if result == 0 {
        Ok(())
    } else {
        Err(PathError::OperationFailed)
    }
}

/// Walks a directory tree in depth-first post-order.
/// The callback is called AFTER a directory's children are processed.
/// This is suitable for operations like recursive deletion.
///
/// # Arguments
/// * `path` - Root directory to walk
/// * `callback` - Function called for each entry with (path, name) returning WalkControl
///
/// # Errors
/// Returns `PathError` if the walk fails.
pub fn walk_depth_first<P, F>(path: P, mut callback: F) -> Result<()>
where
    P: AsRef<Path>,
    F: FnMut(&str, &str) -> WalkControl,
{
    let path_cstr = path_to_cstring(path.as_ref())?;

    unsafe extern "C" fn c_callback(
        _attr: *const crate::ffi::FileAttributes,
        path: *const libc::c_char,
        name: *const libc::c_char,
        data: *mut libc::c_void,
    ) -> WalkDirOption {
        if path.is_null() || name.is_null() || data.is_null() {
            return WalkDirOption::DirError;
        }

        unsafe {
            let path_str = CStr::from_ptr(path).to_str().unwrap_or("");
            let name_str = CStr::from_ptr(name).to_str().unwrap_or("");

            let callback_ptr = data as *mut &mut dyn FnMut(&str, &str) -> WalkControl;
            let control = (*callback_ptr)(path_str, name_str);
            control.into()
        }
    }

    let mut callback_trait: &mut dyn FnMut(&str, &str) -> WalkControl = &mut callback;
    let callback_ptr =
        &mut callback_trait as *mut &mut dyn FnMut(&str, &str) -> WalkControl as *mut libc::c_void;

    let result =
        unsafe { dir_walk_depth_first(path_cstr.as_ptr(), Some(c_callback), callback_ptr) };

    if result == 0 {
        Ok(())
    } else {
        Err(PathError::OperationFailed)
    }
}

/// Path query operations.
pub mod path {
    use super::*;

    /// Returns true if the path exists.
    pub fn exists<P: AsRef<Path>>(path: P) -> bool {
        let Ok(path_cstr) = path_to_cstring(path.as_ref()) else {
            return false;
        };
        unsafe { path_exists(path_cstr.as_ptr()) }
    }

    /// Returns true if the path is a directory.
    pub fn is_dir<P: AsRef<Path>>(path: P) -> bool {
        let Ok(path_cstr) = path_to_cstring(path.as_ref()) else {
            return false;
        };
        unsafe { is_directory(path_cstr.as_ptr()) }
    }

    /// Returns true if the path is a regular file.
    pub fn is_file<P: AsRef<Path>>(path: P) -> bool {
        let Ok(path_cstr) = path_to_cstring(path.as_ref()) else {
            return false;
        };
        unsafe { is_a_file(path_cstr.as_ptr()) }
    }

    /// Returns true if the path is a symbolic link.
    /// This function does nothing on Windows and always returns false.
    pub fn is_symlink<P: AsRef<Path>>(path: P) -> bool {
        let Ok(path_cstr) = path_to_cstring(path.as_ref()) else {
            return false;
        };
        unsafe { is_symbolic_link(path_cstr.as_ptr()) }
    }

    /// Returns the current working directory.
    ///
    /// # Errors
    /// Returns `PathError` if the current directory cannot be determined.
    pub fn current_dir() -> Result<PathBuf> {
        let cwd_ptr = unsafe { get_cwd() };
        if cwd_ptr.is_null() {
            return Err(PathError::OperationFailed);
        }

        let cstr = unsafe { CStr::from_ptr(cwd_ptr) };
        let path = PathBuf::from(cstr.to_string_lossy().into_owned());
        unsafe { libc::free(cwd_ptr as *mut libc::c_void) };

        Ok(path)
    }

    /// Returns the user's home directory.
    ///
    /// # Errors
    /// Returns `PathError` if the home directory cannot be determined.
    pub fn home_dir() -> Result<PathBuf> {
        let home_ptr = unsafe { user_home_dir() };
        if home_ptr.is_null() {
            return Err(PathError::NotFound);
        }

        let cstr = unsafe { CStr::from_ptr(home_ptr) };
        Ok(PathBuf::from(cstr.to_string_lossy().into_owned()))
    }

    /// Returns the platform's temporary directory.
    ///
    /// # Errors
    /// Returns `PathError` if the temp directory cannot be determined.
    pub fn temp_dir() -> Result<PathBuf> {
        let temp_ptr = unsafe { get_tempdir() };
        if temp_ptr.is_null() {
            return Err(PathError::NotFound);
        }

        let cstr = unsafe { CStr::from_ptr(temp_ptr) };
        let path = PathBuf::from(cstr.to_string_lossy().into_owned());
        unsafe { libc::free(temp_ptr as *mut libc::c_void) };

        Ok(path)
    }

    /// Creates a temporary file and returns its path.
    ///
    /// # Errors
    /// Returns `PathError` if the temp file cannot be created.
    pub fn temp_file() -> Result<PathBuf> {
        let temp_ptr = unsafe { make_tempfile() };
        if temp_ptr.is_null() {
            return Err(PathError::OperationFailed);
        }

        let cstr = unsafe { CStr::from_ptr(temp_ptr) };
        let path = PathBuf::from(cstr.to_string_lossy().into_owned());
        unsafe { libc::free(temp_ptr as *mut libc::c_void) };

        Ok(path)
    }

    /// Creates a temporary directory and returns its path.
    ///
    /// # Errors
    /// Returns `PathError` if the temp directory cannot be created.
    pub fn temp_dir_create() -> Result<PathBuf> {
        let temp_ptr = unsafe { make_tempdir() };
        if temp_ptr.is_null() {
            return Err(PathError::OperationFailed);
        }

        let cstr = unsafe { CStr::from_ptr(temp_ptr) };
        let path = PathBuf::from(cstr.to_string_lossy().into_owned());
        unsafe { libc::free(temp_ptr as *mut libc::c_void) };

        Ok(path)
    }

    /// Removes a file or directory.
    ///
    /// # Errors
    /// Returns `PathError` if the path cannot be removed.
    pub fn remove<P: AsRef<Path>>(path: P) -> Result<()> {
        let path_cstr = path_to_cstring(path.as_ref())?;
        let result = unsafe { filepath_remove(path_cstr.as_ptr()) };
        if result == 0 {
            Ok(())
        } else {
            Err(PathError::OperationFailed)
        }
    }

    /// Renames a file or directory.
    ///
    /// # Errors
    /// Returns `PathError` if the path cannot be renamed.
    pub fn rename<P: AsRef<Path>>(oldpath: P, newpath: P) -> Result<()> {
        let old_cstr = path_to_cstring(oldpath.as_ref())?;
        let new_cstr = path_to_cstring(newpath.as_ref())?;
        let result = unsafe { filepath_rename(old_cstr.as_ptr(), new_cstr.as_ptr()) };
        if result == 0 {
            Ok(())
        } else {
            Err(PathError::OperationFailed)
        }
    }
}

/// Path manipulation operations.
pub mod filepath {
    use super::*;

    /// Returns the basename (final component) of the path.
    pub fn basename<P: AsRef<Path>>(path: P) -> Result<String> {
        let path_cstr = path_to_cstring(path.as_ref())?;
        let mut buffer = vec![0u8; 1024];

        unsafe {
            filepath_basename(
                path_cstr.as_ptr(),
                buffer.as_mut_ptr() as *mut libc::c_char,
                buffer.len(),
            )
        };

        cstring_from_buffer(&buffer)
    }

    /// Returns the directory name (all but the final component) of the path.
    pub fn dirname<P: AsRef<Path>>(path: P) -> Result<String> {
        let path_cstr = path_to_cstring(path.as_ref())?;
        let mut buffer = vec![0u8; 1024];

        unsafe {
            filepath_dirname(
                path_cstr.as_ptr(),
                buffer.as_mut_ptr() as *mut libc::c_char,
                buffer.len(),
            )
        };

        cstring_from_buffer(&buffer)
    }

    /// Returns the file extension (without the dot).
    pub fn extension<P: AsRef<Path>>(path: P) -> Result<String> {
        let path_cstr = path_to_cstring(path.as_ref())?;
        let mut buffer = vec![0u8; 256];

        unsafe {
            filepath_extension(
                path_cstr.as_ptr(),
                buffer.as_mut_ptr() as *mut libc::c_char,
                buffer.len(),
            )
        };

        cstring_from_buffer(&buffer)
    }

    /// Returns the filename without its extension.
    pub fn name_only<P: AsRef<Path>>(path: P) -> Result<String> {
        let path_cstr = path_to_cstring(path.as_ref())?;
        let mut buffer = vec![0u8; 1024];

        unsafe {
            filepath_nameonly(
                path_cstr.as_ptr(),
                buffer.as_mut_ptr() as *mut libc::c_char,
                buffer.len(),
            )
        };

        cstring_from_buffer(&buffer)
    }

    /// Returns the absolute path.
    ///
    /// # Errors
    /// Returns `PathError` if the absolute path cannot be determined.
    pub fn absolute<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
        let path_cstr = path_to_cstring(path.as_ref())?;
        let abs_ptr = unsafe { filepath_absolute(path_cstr.as_ptr()) };

        if abs_ptr.is_null() {
            return Err(PathError::OperationFailed);
        }

        let cstr = unsafe { CStr::from_ptr(abs_ptr) };
        let path = PathBuf::from(cstr.to_string_lossy().into_owned());
        unsafe { libc::free(abs_ptr as *mut libc::c_void) };

        Ok(path)
    }

    /// Expands the user home directory (~) in the path.
    ///
    /// # Errors
    /// Returns `PathError` if the expansion fails.
    pub fn expand_user<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
        let path_cstr = path_to_cstring(path.as_ref())?;
        let expanded_ptr = unsafe { filepath_expanduser(path_cstr.as_ptr()) };

        if expanded_ptr.is_null() {
            return Err(PathError::OperationFailed);
        }

        let cstr = unsafe { CStr::from_ptr(expanded_ptr) };
        let path = PathBuf::from(cstr.to_string_lossy().into_owned());
        unsafe { libc::free(expanded_ptr as *mut libc::c_void) };

        Ok(path)
    }

    /// Joins two paths using the OS-specific separator.
    ///
    /// # Errors
    /// Returns `PathError` if the join operation fails.
    pub fn join<P1, P2>(path1: P1, path2: P2) -> Result<PathBuf>
    where
        P1: AsRef<Path>,
        P2: AsRef<Path>,
    {
        let path1_cstr = path_to_cstring(path1.as_ref())?;
        let path2_cstr = path_to_cstring(path2.as_ref())?;
        let joined_ptr = unsafe { filepath_join(path1_cstr.as_ptr(), path2_cstr.as_ptr()) };

        if joined_ptr.is_null() {
            return Err(PathError::OperationFailed);
        }

        let cstr = unsafe { CStr::from_ptr(joined_ptr) };
        let path = PathBuf::from(cstr.to_string_lossy().into_owned());
        unsafe { libc::free(joined_ptr as *mut libc::c_void) };

        Ok(path)
    }

    /// Splits a path into directory and basename components.
    ///
    /// # Errors
    /// Returns `PathError` if the split operation fails.
    pub fn split<P: AsRef<Path>>(path: P) -> Result<(String, String)> {
        let path_cstr = path_to_cstring(path.as_ref())?;
        let mut dir_buffer = vec![0u8; 1024];
        let mut name_buffer = vec![0u8; 256];

        unsafe {
            filepath_split(
                path_cstr.as_ptr(),
                dir_buffer.as_mut_ptr() as *mut libc::c_char,
                name_buffer.as_mut_ptr() as *mut libc::c_char,
                dir_buffer.len(),
                name_buffer.len(),
            )
        };

        let dir = cstring_from_buffer(&dir_buffer)?;
        let name = cstring_from_buffer(&name_buffer)?;

        Ok((dir, name))
    }
}

// Helper functions

fn path_to_cstring(path: &Path) -> Result<CString> {
    let path_str = path.to_str().ok_or(PathError::EncodingError)?;
    CString::new(path_str).map_err(|_| PathError::InvalidPath)
}

fn cstring_from_buffer(buffer: &[u8]) -> Result<String> {
    let null_pos = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
    let cstr =
        CStr::from_bytes_with_nul(&buffer[..=null_pos]).map_err(|_| PathError::EncodingError)?;
    Ok(cstr.to_string_lossy().into_owned())
}

use std::ffi::{CStr, CString};
use std::fmt;

use crate::ffi;

/// Platform-specific pipe file descriptor type.
pub type PipeFd = ffi::PipeFd;

/// Error type for process operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessError {
    InvalidArgument,
    ForkFailed,
    ExecFailed,
    PipeFailed,
    Memory,
    WaitFailed,
    KillFailed,
    PermissionDenied,
    Io,
    Timeout,
    WouldBlock,
    PipeClosed,
    TerminateFailed,
    Unknown,
}

impl fmt::Display for ProcessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ProcessError {}

fn from_process_error(err: ffi::ProcessError) -> Result<(), ProcessError> {
    match err {
        ffi::ProcessError::PROCESS_SUCCESS => Ok(()),
        ffi::ProcessError::PROCESS_ERROR_INVALID_ARGUMENT => Err(ProcessError::InvalidArgument),
        ffi::ProcessError::PROCESS_ERROR_FORK_FAILED => Err(ProcessError::ForkFailed),
        ffi::ProcessError::PROCESS_ERROR_EXEC_FAILED => Err(ProcessError::ExecFailed),
        ffi::ProcessError::PROCESS_ERROR_PIPE_FAILED => Err(ProcessError::PipeFailed),
        ffi::ProcessError::PROCESS_ERROR_MEMORY => Err(ProcessError::Memory),
        ffi::ProcessError::PROCESS_ERROR_WAIT_FAILED => Err(ProcessError::WaitFailed),
        ffi::ProcessError::PROCESS_ERROR_KILL_FAILED => Err(ProcessError::KillFailed),
        ffi::ProcessError::PROCESS_ERROR_PERMISSION_DENIED => Err(ProcessError::PermissionDenied),
        ffi::ProcessError::PROCESS_ERROR_IO => Err(ProcessError::Io),
        ffi::ProcessError::PROCESS_ERROR_TIMEOUT => Err(ProcessError::Timeout),
        ffi::ProcessError::PROCESS_ERROR_WOULD_BLOCK => Err(ProcessError::WouldBlock),
        ffi::ProcessError::PROCESS_ERROR_PIPE_CLOSED => Err(ProcessError::PipeClosed),
        ffi::ProcessError::PROCESS_ERROR_TERMINATE_FAILED => Err(ProcessError::TerminateFailed),
        ffi::ProcessError::PROCESS_ERROR_UNKNOWN => Err(ProcessError::Unknown),
    }
}

/// Result of waiting for a process.
#[derive(Debug, Clone, Copy)]
pub struct ProcessResult {
    /// The exit code of the process.
    pub exit_code: i32,
    /// Whether the process exited normally.
    pub exited_normally: bool,
    /// Terminal signal if the process was killed.
    pub term_signal: i32,
}

/// IO redirection configuration for a spawned process.
pub struct ProcessIO {
    pub stdin_pipe: Option<*mut ffi::PipeHandle>,
    pub stdout_pipe: Option<*mut ffi::PipeHandle>,
    pub stderr_pipe: Option<*mut ffi::PipeHandle>,
    pub merge_stderr: bool,
}

impl Default for ProcessIO {
    fn default() -> Self {
        ProcessIO {
            stdin_pipe: None,
            stdout_pipe: None,
            stderr_pipe: None,
            merge_stderr: false,
        }
    }
}

impl ProcessIO {
    fn to_ffi(&self) -> ffi::ProcessOptions__bindgen_ty_1 {
        ffi::ProcessOptions__bindgen_ty_1 {
            stdin_pipe: self.stdin_pipe.unwrap_or(std::ptr::null_mut()),
            stdout_pipe: self.stdout_pipe.unwrap_or(std::ptr::null_mut()),
            stderr_pipe: self.stderr_pipe.unwrap_or(std::ptr::null_mut()),
            merge_stderr: self.merge_stderr,
        }
    }
}

/// Options for spawning a process.
pub struct SpawnOptions<'a> {
    pub working_directory: Option<&'a str>,
    pub inherit_environment: bool,
    pub detached: bool,
    pub io: ProcessIO,
}

impl<'a> Default for SpawnOptions<'a> {
    fn default() -> Self {
        SpawnOptions {
            working_directory: None,
            inherit_environment: true,
            detached: false,
            io: ProcessIO::default(),
        }
    }
}

/// A handle to a spawned child process.
pub struct Process {
    inner: *mut ffi::ProcessHandle,
}

impl Process {
    /// Spawns a new process with the given command and arguments.
    ///
    /// # Arguments
    /// * `command` - The command to execute
    /// * `args` - Arguments to pass (first element should be the program name)
    pub fn spawn(command: &str, args: &[&str]) -> Result<Self, ProcessError> {
        Self::spawn_with_options(command, args, &SpawnOptions::default())
    }

    /// Spawns a new process with the given command, arguments, and options.
    pub fn spawn_with_options(
        command: &str,
        args: &[&str],
        opts: &SpawnOptions,
    ) -> Result<Self, ProcessError> {
        let cmd_c = CString::new(command).map_err(|_| ProcessError::InvalidArgument)?;
        let args_c: Vec<CString> = args.iter().map(|a| CString::new(*a).unwrap()).collect();
        let mut argv_ptrs: Vec<*const libc::c_char> = args_c.iter().map(|a| a.as_ptr()).collect();
        argv_ptrs.push(std::ptr::null());

        let wd_c = opts.working_directory.map(|s| CString::new(s).unwrap());

        let options = ffi::ProcessOptions {
            working_directory: wd_c.as_ref().map_or(std::ptr::null(), |c| c.as_ptr()),
            inherit_environment: opts.inherit_environment,
            environment: std::ptr::null(),
            detached: opts.detached,
            io: opts.io.to_ffi(),
        };

        let mut handle: *mut ffi::ProcessHandle = std::ptr::null_mut();
        from_process_error(unsafe {
            ffi::process_create(&mut handle, cmd_c.as_ptr(), argv_ptrs.as_ptr(), &options)
        })?;

        Ok(Process { inner: handle })
    }

    /// Waits for the process to exit with an optional timeout in milliseconds.
    /// Pass -1 for no timeout.
    pub fn wait(&self, timeout_ms: i32) -> Result<ProcessResult, ProcessError> {
        let mut result: ffi::ProcessResult = unsafe { std::mem::zeroed() };
        from_process_error(unsafe { ffi::process_wait(self.inner, &mut result, timeout_ms) })?;
        Ok(ProcessResult {
            exit_code: result.exit_code,
            exited_normally: result.exited_normally,
            term_signal: result.term_signal,
        })
    }

    /// Terminates the process.
    /// If `force` is true, sends SIGKILL; otherwise sends SIGTERM.
    pub fn terminate(&self, force: bool) -> Result<(), ProcessError> {
        from_process_error(unsafe { ffi::process_terminate(self.inner, force) })
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe { ffi::process_free(self.inner) };
        }
    }
}

unsafe impl Send for Process {}

/// Returns a human-readable error description for a process error code.
pub fn error_string(err: ProcessError) -> &'static str {
    let ffi_err = match err {
        ProcessError::InvalidArgument => ffi::ProcessError::PROCESS_ERROR_INVALID_ARGUMENT,
        ProcessError::ForkFailed => ffi::ProcessError::PROCESS_ERROR_FORK_FAILED,
        ProcessError::ExecFailed => ffi::ProcessError::PROCESS_ERROR_EXEC_FAILED,
        ProcessError::PipeFailed => ffi::ProcessError::PROCESS_ERROR_PIPE_FAILED,
        ProcessError::Memory => ffi::ProcessError::PROCESS_ERROR_MEMORY,
        ProcessError::WaitFailed => ffi::ProcessError::PROCESS_ERROR_WAIT_FAILED,
        ProcessError::KillFailed => ffi::ProcessError::PROCESS_ERROR_KILL_FAILED,
        ProcessError::PermissionDenied => ffi::ProcessError::PROCESS_ERROR_PERMISSION_DENIED,
        ProcessError::Io => ffi::ProcessError::PROCESS_ERROR_IO,
        ProcessError::Timeout => ffi::ProcessError::PROCESS_ERROR_TIMEOUT,
        ProcessError::WouldBlock => ffi::ProcessError::PROCESS_ERROR_WOULD_BLOCK,
        ProcessError::PipeClosed => ffi::ProcessError::PROCESS_ERROR_PIPE_CLOSED,
        ProcessError::TerminateFailed => ffi::ProcessError::PROCESS_ERROR_TERMINATE_FAILED,
        ProcessError::Unknown => ffi::ProcessError::PROCESS_ERROR_UNKNOWN,
    };
    unsafe {
        let ptr = ffi::process_error_string(ffi_err);
        if ptr.is_null() {
            "Unknown error"
        } else {
            CStr::from_ptr(ptr).to_str().unwrap_or("Unknown error")
        }
    }
}

/// A pipe for inter-process communication.
pub struct Pipe {
    inner: *mut ffi::PipeHandle,
}

impl Pipe {
    /// Creates a new pipe.
    pub fn new() -> Result<Self, ProcessError> {
        let mut handle: *mut ffi::PipeHandle = std::ptr::null_mut();
        from_process_error(unsafe { ffi::pipe_create(&mut handle) })?;
        Ok(Pipe { inner: handle })
    }

    /// Returns the raw read file descriptor.
    pub fn read_fd(&self) -> PipeFd {
        unsafe { ffi::pipe_read_fd(self.inner) }
    }

    /// Returns the raw write file descriptor.
    pub fn write_fd(&self) -> PipeFd {
        unsafe { ffi::pipe_write_fd(self.inner) }
    }

    /// Returns true if the read end is closed.
    pub fn read_closed(&self) -> bool {
        unsafe { ffi::pipe_read_closed(self.inner) }
    }

    /// Returns true if the write end is closed.
    pub fn write_closed(&self) -> bool {
        unsafe { ffi::pipe_write_closed(self.inner) }
    }

    /// Sets non-blocking mode on the pipe.
    pub fn set_nonblocking(&self, nonblocking: bool) -> Result<(), ProcessError> {
        from_process_error(unsafe { ffi::pipe_set_nonblocking(self.inner, nonblocking) })
    }

    /// Reads data from the pipe.
    ///
    /// Returns the number of bytes read.
    pub fn read(&self, buf: &mut [u8], timeout_ms: i32) -> Result<usize, ProcessError> {
        let mut bytes_read: usize = 0;
        from_process_error(unsafe {
            ffi::pipe_read(
                self.inner,
                buf.as_mut_ptr() as *mut libc::c_void,
                buf.len(),
                &mut bytes_read,
                timeout_ms,
            )
        })?;
        Ok(bytes_read)
    }

    /// Writes data to the pipe.
    ///
    /// Returns the number of bytes written.
    pub fn write(&self, data: &[u8], timeout_ms: i32) -> Result<usize, ProcessError> {
        let mut bytes_written: usize = 0;
        from_process_error(unsafe {
            ffi::pipe_write(
                self.inner,
                data.as_ptr() as *const libc::c_void,
                data.len(),
                &mut bytes_written,
                timeout_ms,
            )
        })?;
        Ok(bytes_written)
    }

    /// Closes the read end of the pipe.
    pub fn close_read(&self) -> Result<(), ProcessError> {
        from_process_error(unsafe { ffi::pipe_close_read_end(self.inner) })
    }

    /// Closes the write end of the pipe.
    pub fn close_write(&self) -> Result<(), ProcessError> {
        from_process_error(unsafe { ffi::pipe_close_write_end(self.inner) })
    }

    /// Returns the raw inner pointer (for use with ProcessOptions IO).
    pub fn as_ptr(&mut self) -> *mut ffi::PipeHandle {
        self.inner
    }
}

impl Drop for Pipe {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe { ffi::pipe_close(self.inner) };
        }
    }
}

unsafe impl Send for Pipe {}

/// A file redirection handle for redirecting process IO to/from files.
pub struct FileRedirect {
    inner: *mut ffi::FileRedirection,
}

impl FileRedirect {
    /// Creates a file redirection to the given file path.
    ///
    /// `flags` are libc open flags (e.g., `libc::O_WRONLY | libc::O_CREAT`).
    /// `mode` is the file creation mode (e.g., `0o644`).
    pub fn to_file(path: &str, flags: i32, mode: u32) -> Result<Self, ProcessError> {
        let path_c = CString::new(path).map_err(|_| ProcessError::InvalidArgument)?;
        let mut handle: *mut ffi::FileRedirection = std::ptr::null_mut();
        from_process_error(unsafe {
            ffi::process_redirect_to_file(&mut handle, path_c.as_ptr(), flags, mode)
        })?;
        Ok(FileRedirect { inner: handle })
    }

    /// Creates a file redirection from an existing file descriptor.
    pub fn to_fd(fd: i32, close_on_exec: bool) -> Result<Self, ProcessError> {
        let mut handle: *mut ffi::FileRedirection = std::ptr::null_mut();
        from_process_error(unsafe { ffi::process_redirect_to_fd(&mut handle, fd, close_on_exec) })?;
        Ok(FileRedirect { inner: handle })
    }

    /// Returns the raw inner pointer (for use with ExtendedProcessIO).
    pub fn as_ptr(&mut self) -> *mut ffi::FileRedirection {
        self.inner
    }
}

impl Drop for FileRedirect {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe { ffi::process_close_redirection(self.inner) };
        }
    }
}

/// Spawns a process that redirects stdout/stderr to files.
///
/// Pass `None` for stdout_file or stderr_file to leave them unredirected.
pub fn run_with_file_redirection(
    command: &str,
    args: &[&str],
    stdout_file: Option<&str>,
    stderr_file: Option<&str>,
    append: bool,
) -> Result<Process, ProcessError> {
    let cmd_c = CString::new(command).map_err(|_| ProcessError::InvalidArgument)?;
    let args_c: Vec<CString> = args.iter().map(|a| CString::new(*a).unwrap()).collect();
    let mut argv_ptrs: Vec<*const libc::c_char> = args_c.iter().map(|a| a.as_ptr()).collect();
    argv_ptrs.push(std::ptr::null());

    let stdout_c = stdout_file.map(|s| CString::new(s).unwrap());
    let stderr_c = stderr_file.map(|s| CString::new(s).unwrap());

    let mut handle: *mut ffi::ProcessHandle = std::ptr::null_mut();
    from_process_error(unsafe {
        ffi::process_run_with_file_redirection(
            &mut handle,
            cmd_c.as_ptr(),
            argv_ptrs.as_ptr(),
            stdout_c.as_ref().map_or(std::ptr::null(), |c| c.as_ptr()),
            stderr_c.as_ref().map_or(std::ptr::null(), |c| c.as_ptr()),
            append,
        )
    })?;
    Ok(Process { inner: handle })
}

/// Runs a command and duplicates its output to multiple file descriptors.
pub fn run_with_multiwriter(
    command: &str,
    args: &[&str],
    output_fds: &mut [i32],
    error_fds: &mut [i32],
) -> Result<ProcessResult, ProcessError> {
    let cmd_c = CString::new(command).map_err(|_| ProcessError::InvalidArgument)?;
    let args_c: Vec<CString> = args.iter().map(|a| CString::new(*a).unwrap()).collect();
    let mut argv_ptrs: Vec<*const libc::c_char> = args_c.iter().map(|a| a.as_ptr()).collect();
    argv_ptrs.push(std::ptr::null());

    let mut result: ffi::ProcessResult = unsafe { std::mem::zeroed() };
    from_process_error(unsafe {
        ffi::process_run_with_multiwriter(
            &mut result,
            cmd_c.as_ptr(),
            argv_ptrs.as_mut_ptr() as *mut *const _,
            output_fds.as_mut_ptr(),
            error_fds.as_mut_ptr(),
        )
    })?;
    Ok(ProcessResult {
        exit_code: result.exit_code,
        exited_normally: result.exited_normally,
        term_signal: result.term_signal,
    })
}

/// Runs a command and captures its output, returning the exit code.
pub fn run(command: &str, args: &[&str]) -> Result<i32, ProcessError> {
    let cmd_c = CString::new(command).map_err(|_| ProcessError::InvalidArgument)?;
    let args_c: Vec<CString> = args.iter().map(|a| CString::new(*a).unwrap()).collect();
    let mut argv_ptrs: Vec<*const libc::c_char> = args_c.iter().map(|a| a.as_ptr()).collect();
    argv_ptrs.push(std::ptr::null());

    let mut options = ffi::ProcessOptions {
        working_directory: std::ptr::null(),
        inherit_environment: true,
        environment: std::ptr::null(),
        detached: false,
        io: ProcessIO::default().to_ffi(),
    };

    let mut exit_code: i32 = 0;
    from_process_error(unsafe {
        ffi::process_run_and_capture(
            cmd_c.as_ptr(),
            argv_ptrs.as_ptr(),
            &mut options,
            &mut exit_code,
        )
    })?;
    Ok(exit_code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_and_wait() {
        let proc = Process::spawn("/bin/echo", &["echo", "hello"]).unwrap();
        let result = proc.wait(-1).unwrap();
        assert!(result.exited_normally);
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_spawn_nonexistent() {
        let result = Process::spawn("/nonexistent/binary", &["binary"]);
        if let Ok(proc) = result {
            let wait_result = proc.wait(5000);
            if let Ok(r) = wait_result {
                assert_ne!(r.exit_code, 0);
            }
        }
    }

    #[test]
    fn test_run() {
        let exit_code = run("/bin/true", &["true"]).unwrap();
        assert_eq!(exit_code, 0);
    }

    #[test]
    fn test_run_false() {
        let result = run("/bin/false", &["false"]);
        match result {
            Ok(code) => assert_ne!(code, 0),
            Err(_) => {} // Also acceptable
        }
    }

    #[test]
    fn test_pipe_create_and_write_read() {
        let pipe = Pipe::new().unwrap();
        assert!(!pipe.read_closed());
        assert!(!pipe.write_closed());

        let data = b"hello pipe";
        let written = pipe.write(data, -1).unwrap();
        assert_eq!(written, data.len());

        let mut buf = [0u8; 64];
        let read = pipe.read(&mut buf, -1).unwrap();
        assert_eq!(read, data.len());
        assert_eq!(&buf[..read], data);
    }

    #[test]
    fn test_pipe_close_ends() {
        let pipe = Pipe::new().unwrap();
        pipe.close_write().unwrap();
        assert!(pipe.write_closed());
        // Read should return 0 or error since write end is closed
    }

    #[test]
    fn test_pipe_nonblocking() {
        let pipe = Pipe::new().unwrap();
        pipe.set_nonblocking(true).unwrap();
        // Non-blocking read on empty pipe should return WouldBlock
        let mut buf = [0u8; 16];
        let result = pipe.read(&mut buf, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_pipe_with_process() {
        let mut stdout_pipe = Pipe::new().unwrap();

        let opts = SpawnOptions {
            io: ProcessIO {
                stdout_pipe: Some(stdout_pipe.as_ptr()),
                ..ProcessIO::default()
            },
            ..SpawnOptions::default()
        };

        let proc =
            Process::spawn_with_options("/bin/echo", &["echo", "hello from pipe"], &opts).unwrap();
        proc.wait(-1).unwrap();

        // Close write end so read gets EOF
        stdout_pipe.close_write().unwrap();

        let mut buf = [0u8; 256];
        let n = stdout_pipe.read(&mut buf, -1).unwrap();
        let output = std::str::from_utf8(&buf[..n]).unwrap();
        assert_eq!(output.trim(), "hello from pipe");

        // Prevent Pipe::drop from double-closing (process_free closes it)
        stdout_pipe.inner = std::ptr::null_mut();
    }

    #[test]
    fn test_error_string() {
        let s = error_string(ProcessError::InvalidArgument);
        assert!(!s.is_empty());
    }

    #[test]
    fn test_run_with_file_redirection() {
        let tmp = format!("/tmp/solidc_test_redirect_{}.txt", std::process::id());
        let proc = run_with_file_redirection(
            "/bin/echo",
            &["echo", "redirected output"],
            Some(&tmp),
            None,
            false,
        )
        .unwrap();
        let result = proc.wait(-1).unwrap();
        assert!(result.exited_normally);
        assert_eq!(result.exit_code, 0);

        let contents = std::fs::read_to_string(&tmp).unwrap();
        assert_eq!(contents.trim(), "redirected output");
        std::fs::remove_file(&tmp).ok();
    }
}

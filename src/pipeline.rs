use std::ffi::CString;
use std::os::raw::c_char;

use crate::ffi;

/// A command in a pipeline.
pub struct Command {
    inner: *mut ffi::CommandNode,
    // Keep args alive
    _args: Vec<CString>,
    _arg_ptrs: Vec<*mut c_char>,
}

impl Command {
    /// Creates a new command from the given arguments.
    pub fn new(args: &[&str]) -> Option<Self> {
        let c_args: Vec<CString> = args.iter().filter_map(|s| CString::new(*s).ok()).collect();
        if c_args.len() != args.len() {
            return None;
        }

        let mut arg_ptrs: Vec<*mut c_char> =
            c_args.iter().map(|s| s.as_ptr() as *mut c_char).collect();
        arg_ptrs.push(std::ptr::null_mut()); // NULL-terminated

        let ptr = unsafe { ffi::create_command_node(arg_ptrs.as_mut_ptr()) };
        if ptr.is_null() {
            None
        } else {
            Some(Command {
                inner: ptr,
                _args: c_args,
                _arg_ptrs: arg_ptrs,
            })
        }
    }
}

/// A pipeline of commands connected by pipes.
pub struct Pipeline {
    commands: Vec<Command>,
    /// Set to true after execute/execute_to_fd so Drop doesn't double-free.
    consumed: bool,
}

impl Pipeline {
    /// Creates a new empty pipeline.
    pub fn new() -> Self {
        Pipeline {
            commands: Vec::new(),
            consumed: false,
        }
    }

    /// Adds a command to the pipeline.
    pub fn add(mut self, args: &[&str]) -> Self {
        if let Some(cmd) = Command::new(args) {
            self.commands.push(cmd);
        }
        self
    }

    /// Executes the pipeline.
    pub fn execute(mut self) {
        if self.commands.is_empty() {
            return;
        }

        // Build the linked list
        let mut ptrs: Vec<*mut ffi::CommandNode> = self.commands.iter().map(|c| c.inner).collect();
        ptrs.push(std::ptr::null_mut()); // NULL-terminated

        unsafe { ffi::build_pipeline(ptrs.as_mut_ptr()) };
        // build_pipeline links and executes; free the chain
        unsafe { ffi::free_pipeline(self.commands[0].inner) };
        self.consumed = true;
    }

    /// Executes the pipeline and captures output to a file descriptor.
    pub fn execute_to_fd(mut self, output_fd: i32) {
        if self.commands.is_empty() {
            return;
        }

        // Link commands together
        for i in 0..self.commands.len() - 1 {
            unsafe { (*self.commands[i].inner).next = self.commands[i + 1].inner };
        }

        unsafe { ffi::execute_pipeline(self.commands[0].inner, output_fd) };
        unsafe { ffi::free_pipeline(self.commands[0].inner) };
        self.consumed = true;
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}

// Pipeline manages CommandNode lifetime. If the pipeline was executed,
// free_pipeline already freed the nodes. Otherwise, Drop cleans up.
impl Drop for Pipeline {
    fn drop(&mut self) {
        if !self.consumed && !self.commands.is_empty() {
            // Link the commands into a list so free_pipeline walks them all
            for i in 0..self.commands.len() - 1 {
                unsafe { (*self.commands[i].inner).next = self.commands[i + 1].inner };
            }
            unsafe { ffi::free_pipeline(self.commands[0].inner) };
        }
        // Mark all commands so their Drop doesn't try anything
        for cmd in &mut self.commands {
            cmd.inner = std::ptr::null_mut();
        }
    }
}

impl Drop for Command {
    fn drop(&mut self) {
        // Freed by Pipeline via free_pipeline; only free if orphaned
        if !self.inner.is_null() {
            unsafe { ffi::free_pipeline(self.inner) };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_command() {
        let cmd = Command::new(&["echo", "hello"]);
        assert!(cmd.is_some());
    }

    #[test]
    fn test_pipeline_builder() {
        let pipeline = Pipeline::new()
            .add(&["echo", "hello world"])
            .add(&["wc", "-w"]);
        assert_eq!(pipeline.commands.len(), 2);
    }

    ///  Test output capture by executing a simple pipeline and reading the output.
    #[test]
    #[cfg(unix)]
    fn test_pipeline_execution() {
        use std::io::Read;
        use std::os::unix::io::FromRawFd;

        // Create a pipe: write end for the pipeline, read end to capture output
        let (read_fd, write_fd) = unsafe {
            let mut fds = [0i32; 2];
            assert_eq!(libc::pipe(fds.as_mut_ptr()), 0);
            (fds[0], fds[1])
        };

        // echo "hello world" | wc -w  →  "2\n"
        Pipeline::new()
            .add(&["echo", "hello world"])
            .add(&["wc", "-w"])
            .execute_to_fd(write_fd);

        unsafe { libc::close(write_fd) };

        let mut buf = String::new();
        let mut file = unsafe { std::fs::File::from_raw_fd(read_fd) };
        file.read_to_string(&mut buf).unwrap();

        assert_eq!(buf.trim(), "2");
    }

    // Note: Pipeline uses fork/execvp internally and is Unix-only.
    // No Windows test is provided.
}

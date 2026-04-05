use std::ffi::CStr;

use crate::ffi;

/// Thread management and system information utilities.
pub mod thread {
    use super::*;

    /// A thread handle wrapping the C thread type.
    pub struct Thread {
        inner: ffi::Thread,
    }

    // Box the closure to ensure it has a stable address on the heap
    struct ThreadData {
        func: Box<dyn FnOnce() + Send + 'static>,
    }

    unsafe extern "C" fn thread_trampoline(data: *mut libc::c_void) -> *mut libc::c_void {
        let boxed = unsafe { Box::from_raw(data as *mut ThreadData) };
        (boxed.func)();
        std::ptr::null_mut()
    }

    impl Thread {
        /// Spawns a new thread running the given closure.
        pub fn spawn<F>(f: F) -> Result<Self, i32>
        where
            F: FnOnce() + Send + 'static,
        {
            let data = Box::new(ThreadData { func: Box::new(f) });
            let data_ptr = Box::into_raw(data) as *mut libc::c_void;

            let mut thread: ffi::Thread = 0;
            let result =
                unsafe { ffi::thread_create(&mut thread, Some(thread_trampoline), data_ptr) };

            if result != 0 {
                // Reclaim the data to prevent leak
                unsafe { drop(Box::from_raw(data_ptr as *mut ThreadData)) };
                Err(result)
            } else {
                Ok(Thread { inner: thread })
            }
        }

        /// Waits for the thread to finish.
        pub fn join(self) -> Result<(), i32> {
            let result = unsafe { ffi::thread_join(self.inner, std::ptr::null_mut()) };
            std::mem::forget(self); // Don't run drop
            if result == 0 { Ok(()) } else { Err(result) }
        }

        /// Detaches the thread, allowing it to run independently.
        pub fn detach(self) -> Result<(), i32> {
            let result = unsafe { ffi::thread_detach(self.inner) };
            std::mem::forget(self);
            if result == 0 { Ok(()) } else { Err(result) }
        }
    }
}

/// System information utilities.
pub mod sysinfo {
    use super::*;

    /// Returns the number of available CPUs.
    pub fn num_cpus() -> usize {
        let n = unsafe { ffi::get_ncpus() };
        n as usize
    }

    /// Returns the current process ID.
    pub fn pid() -> i32 {
        unsafe { ffi::get_pid() }
    }

    /// Returns the parent process ID.
    pub fn ppid() -> i32 {
        unsafe { ffi::get_ppid() }
    }

    /// Returns the current thread ID.
    pub fn tid() -> u64 {
        unsafe { ffi::get_tid() as u64 }
    }

    /// Returns the current user ID.
    pub fn uid() -> u32 {
        unsafe { ffi::get_uid() }
    }

    /// Returns the current group ID.
    pub fn gid() -> u32 {
        unsafe { ffi::get_gid() }
    }

    /// Returns the current username.
    pub fn username() -> Option<String> {
        let ptr = unsafe { ffi::get_username() };
        if ptr.is_null() {
            None
        } else {
            let cstr = unsafe { CStr::from_ptr(ptr) };
            Some(cstr.to_string_lossy().into_owned())
        }
    }

    /// Returns the current group name.
    pub fn groupname() -> Option<String> {
        let ptr = unsafe { ffi::get_groupname() };
        if ptr.is_null() {
            None
        } else {
            let cstr = unsafe { CStr::from_ptr(ptr) };
            Some(cstr.to_string_lossy().into_owned())
        }
    }

    /// Sleeps for the given number of milliseconds.
    pub fn sleep_ms(ms: i32) {
        unsafe { ffi::sleep_ms(ms) }
    }
}

/// A work-stealing thread pool.
pub struct ThreadPool {
    inner: *mut ffi::Threadpool,
}

struct TaskData {
    func: Box<dyn FnOnce() + Send + 'static>,
}

unsafe extern "C" fn task_trampoline(data: *mut libc::c_void) {
    let boxed = unsafe { Box::from_raw(data as *mut TaskData) };
    (boxed.func)();
}

impl ThreadPool {
    /// Creates a new thread pool with the given number of worker threads.
    pub fn new(num_threads: usize) -> Option<Self> {
        let ptr = unsafe { ffi::threadpool_create(num_threads) };
        if ptr.is_null() {
            None
        } else {
            Some(ThreadPool { inner: ptr })
        }
    }

    /// Submits a task to the thread pool.
    pub fn submit<F>(&self, f: F) -> bool
    where
        F: FnOnce() + Send + 'static,
    {
        let data = Box::new(TaskData { func: Box::new(f) });
        let data_ptr = Box::into_raw(data) as *mut libc::c_void;
        let ok = unsafe { ffi::threadpool_submit(self.inner, Some(task_trampoline), data_ptr) };
        if !ok {
            // Reclaim to prevent leak
            unsafe { drop(Box::from_raw(data_ptr as *mut TaskData)) };
        }
        ok
    }

    /// Waits for all submitted tasks to complete.
    pub fn wait(&self) {
        unsafe { ffi::threadpool_wait(self.inner) }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe { ffi::threadpool_destroy(self.inner, 5000) };
        }
    }
}

unsafe impl Send for ThreadPool {}
unsafe impl Sync for ThreadPool {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicI32, Ordering};

    #[test]
    fn test_thread_spawn_join() {
        let counter = Arc::new(AtomicI32::new(0));
        let c = counter.clone();

        let t = thread::Thread::spawn(move || {
            c.fetch_add(1, Ordering::SeqCst);
        })
        .unwrap();

        t.join().unwrap();
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_sysinfo() {
        assert!(sysinfo::num_cpus() > 0);
        assert!(sysinfo::pid() > 0);
        assert!(sysinfo::username().is_some());
    }

    #[test]
    fn test_threadpool() {
        let pool = ThreadPool::new(4).unwrap();
        let counter = Arc::new(AtomicI32::new(0));

        for _ in 0..100 {
            let c = counter.clone();
            pool.submit(move || {
                c.fetch_add(1, Ordering::SeqCst);
            });
        }

        pool.wait();
        assert_eq!(counter.load(Ordering::SeqCst), 100);
    }

    #[test]
    fn test_sleep_ms() {
        let start = std::time::Instant::now();
        sysinfo::sleep_ms(50);
        let elapsed = start.elapsed();
        assert!(elapsed.as_millis() >= 40); // Allow some tolerance
    }
}

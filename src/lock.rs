use crate::ffi;

/// A mutual exclusion lock (mutex).
pub struct Mutex {
    inner: ffi::Lock,
}

impl Mutex {
    /// Creates a new unlocked mutex.
    pub fn new() -> Result<Self, i32> {
        let mut lock: ffi::Lock = unsafe { std::mem::zeroed() };
        let result = unsafe { ffi::lock_init(&mut lock) };
        if result == 0 {
            Ok(Mutex { inner: lock })
        } else {
            Err(result)
        }
    }

    /// Acquires the mutex, blocking until it becomes available.
    pub fn lock(&mut self) -> Result<MutexGuard<'_>, i32> {
        let result = unsafe { ffi::lock_acquire(&mut self.inner) };
        if result == 0 {
            Ok(MutexGuard { mutex: self })
        } else {
            Err(result)
        }
    }

    /// Attempts to acquire the mutex without blocking.
    pub fn try_lock(&mut self) -> Result<MutexGuard<'_>, i32> {
        let result = unsafe { ffi::lock_try_acquire(&mut self.inner) };
        if result == 0 {
            Ok(MutexGuard { mutex: self })
        } else {
            Err(result)
        }
    }
}

impl Drop for Mutex {
    fn drop(&mut self) {
        unsafe { ffi::lock_free(&mut self.inner) };
    }
}

impl Default for Mutex {
    fn default() -> Self {
        Mutex::new().expect("Failed to create Mutex")
    }
}

/// RAII guard that releases the mutex when dropped.
pub struct MutexGuard<'a> {
    mutex: &'a mut Mutex,
}

impl<'a> Drop for MutexGuard<'a> {
    fn drop(&mut self) {
        unsafe { ffi::lock_release(&mut self.mutex.inner) };
    }
}

/// A condition variable for thread synchronization.
pub struct Condvar {
    inner: ffi::Condition,
}

impl Condvar {
    /// Creates a new condition variable.
    pub fn new() -> Result<Self, i32> {
        let mut cond: ffi::Condition = unsafe { std::mem::zeroed() };
        let result = unsafe { ffi::cond_init(&mut cond) };
        if result == 0 {
            Ok(Condvar { inner: cond })
        } else {
            Err(result)
        }
    }

    /// Wakes one waiting thread.
    pub fn notify_one(&mut self) -> Result<(), i32> {
        let result = unsafe { ffi::cond_signal(&mut self.inner) };
        if result == 0 { Ok(()) } else { Err(result) }
    }

    /// Wakes all waiting threads.
    pub fn notify_all(&mut self) -> Result<(), i32> {
        let result = unsafe { ffi::cond_broadcast(&mut self.inner) };
        if result == 0 { Ok(()) } else { Err(result) }
    }

    /// Waits on the condition variable, releasing the mutex.
    pub fn wait(&mut self, guard: &mut MutexGuard<'_>) -> Result<(), i32> {
        let result = unsafe { ffi::cond_wait(&mut self.inner, &mut guard.mutex.inner) };
        if result == 0 { Ok(()) } else { Err(result) }
    }

    /// Waits on the condition variable with a timeout in milliseconds.
    pub fn wait_timeout(&mut self, guard: &mut MutexGuard<'_>, timeout_ms: i32) -> Result<(), i32> {
        let result =
            unsafe { ffi::cond_wait_timeout(&mut self.inner, &mut guard.mutex.inner, timeout_ms) };
        if result == 0 { Ok(()) } else { Err(result) }
    }
}

impl Drop for Condvar {
    fn drop(&mut self) {
        unsafe { ffi::cond_free(&mut self.inner) };
    }
}

impl Default for Condvar {
    fn default() -> Self {
        Condvar::new().expect("Failed to create Condvar")
    }
}

unsafe impl Send for Mutex {}
unsafe impl Send for Condvar {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mutex_lock_unlock() {
        let mut mutex = Mutex::new().unwrap();
        {
            let _guard = mutex.lock().unwrap();
            // Lock is held here
        }
        // Lock is released here
        let _guard = mutex.lock().unwrap();
    }

    #[test]
    fn test_mutex_try_lock() {
        let mut mutex = Mutex::new().unwrap();
        let _guard = mutex.try_lock().unwrap();
        // Lock is held, try_lock should fail on same thread
        // (behavior is implementation-defined for non-recursive mutex)
    }

    #[test]
    fn test_condvar_create() {
        let _cv = Condvar::new().unwrap();
    }
}

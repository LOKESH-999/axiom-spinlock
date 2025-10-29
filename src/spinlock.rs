//! # SpinLock
//!
//! A minimal, `no_std`-compatible axiom_spinlock implementation with optional backoff-based
//! contention handling. This lock is intended for use in low-level or embedded contexts
//! where blocking synchronization primitives (like `std::sync::Mutex`) are not available
//! or desirable.
//!
//! The [`SpinLock`] type provides mutual exclusion by continuously attempting to acquire
//! a lock using atomic operations. To reduce contention pressure, it uses an exponential
//! [`BackOff`] strategy before retrying.
//!
//! ## Features
//! - ✅ `no_std` compatible
//! - ✅ Optional backoff-based spinning via [`BackOff`]
//! - ✅ Supports `try_lock` and `try_lock_for` with custom spin limits
//! - ✅ `with_lock()` convenience method for scoped access
//! - 🧠 Simple and efficient for short critical sections
//!
//! ## Safety
//! - The `SpinLock` is **not fair** — starvation is possible under heavy contention.
//! - Always prefer using it for small, fast operations where contention is low.
//! - It should **not** be held across system calls or long-running operations.
//!
//! ## Example
//! ```rust
//! use axiom_spinlock::SpinLock;
//!
//! static COUNTER: SpinLock<u32> = SpinLock::new(0);
//!
//! fn increment() {
//!     let mut guard = COUNTER.lock();
//!     *guard += 1;
//! }
//!
//! fn read_value() -> u32 {
//!     *COUNTER.lock()
//! }
//!
//! fn main() {
//!     increment();
//!     println!("Counter = {}", read_value());
//! }
//! ```
//!
//! ## When to Use
//! - Embedded systems
//! - Custom runtimes
//! - Lock-free fallback implementations
//! - Performance-critical microbenchmarks
//!
//! ## When *Not* to Use
//! - Long critical sections
//! - Code that can yield or block the thread
//! - High-contention multi-core workloads (use a fair mutex instead)

use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{
    AtomicBool,
    Ordering::{Acquire, Release},
};

use crate::BackOff;

/// A simple spin-based mutual exclusion primitive.
///
/// This lock uses atomic spinning with an exponential [`BackOff`] to minimize
/// CPU usage under contention. It does not perform OS-level thread blocking.
///
/// See the [module-level documentation](#) for examples and caveats.
pub struct SpinLock<T> {
    data: UnsafeCell<T>,
    locked: AtomicBool,
}

/// A guard that releases the [`SpinLock`] when dropped.
///
/// This is returned from [`SpinLock::lock`] and implements [`Deref`] and [`DerefMut`]
/// to access the underlying data.
pub struct SpinGuard<'a, T> {
    guard: &'a SpinLock<T>,
}

impl<'a, T> Drop for SpinGuard<'a, T> {
    #[inline]
    fn drop(&mut self) {
        self.guard.locked.store(false, Release)
    }
}

impl<T> SpinLock<T> {
    /// Creates a new [`SpinLock`] wrapping the given data.
    ///
    /// # Example
    /// ```
    /// use axiom_spinlock::SpinLock;
    ///
    /// let lock = SpinLock::new(123);
    /// assert_eq!(*lock.lock(), 123);
    /// ```
    #[inline(always)]
    pub const fn new(data: T) -> Self {
        SpinLock {
            data: UnsafeCell::new(data),
            locked: AtomicBool::new(false),
        }
    }

    /// Acquires the lock, spinning until it becomes available.
    ///
    /// Uses an exponential [`BackOff`] to reduce contention.
    /// Returns a [`SpinGuard`] which automatically releases the lock on drop.
    #[inline]
    pub fn lock(&self) -> SpinGuard<'_, T> {
        let backoff = BackOff::new();
        while self.locked.swap(true, Acquire) {
            // Acquire is sufficient here since swap ensures visibility of writes
            backoff.wait();
        }

        SpinGuard { guard: self }
    }

    /// Unsafely releases the lock manually.
    ///
    /// # Safety
    /// - Only call this if you *own* the lock.
    /// - Misuse can cause data races or UB.
    #[inline]
    pub unsafe fn unlock(&self) {
        self.locked.store(false, Release);
    }

    /// Attempts to acquire the lock without blocking.
    ///
    /// Returns `Some(SpinGuard)` if the lock was free, or `None` otherwise.
    #[inline]
    pub fn try_lock(&self) -> Option<SpinGuard<'_, T>> {
        if !self.locked.swap(true, Acquire) {
            Some(SpinGuard { guard: self })
        } else {
            None
        }
    }

    /// Checks whether the lock is currently held.
    #[inline(always)]
    pub fn is_locked(&self) -> bool {
        self.locked.load(Acquire)
    }

    /// Tries to acquire the lock within a fixed number of spin attempts.
    ///
    /// Returns `Some(SpinGuard)` if successful, otherwise `None` after the given number of spins.
    #[inline]
    pub fn try_lock_for(&self, spins: usize) -> Option<SpinGuard<'_, T>> {
        let backoff = BackOff::new();
        for _ in 0..spins {
            if !self.locked.swap(true, Acquire) {
                return Some(SpinGuard { guard: self });
            }
            backoff.wait();
        }
        None
    }

    /// Runs a closure with exclusive access to the data.
    ///
    /// This is a convenience wrapper around [`lock()`] that automatically releases
    /// the lock when the closure returns.
    ///
    /// # Example
    /// ```
    /// use axiom_spinlock::SpinLock;
    /// let lock = SpinLock::new(0i32);
    /// lock.with_lock(|data| {
    ///     *data += 1;
    /// });
    /// ```
    #[inline]
    pub fn with_lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut guard = self.lock();
        f(&mut *guard)
    }
}

impl<T> Deref for SpinGuard<'_, T> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &T {
        unsafe { &*(self.guard.data.get()) }
    }
}

impl<T> DerefMut for SpinGuard<'_, T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.guard.data.get() }
    }
}

// Safety: SpinLock enforces mutual exclusion via atomic operations.
unsafe impl<T: Send> Send for SpinLock<T> {}
unsafe impl<T: Send> Sync for SpinLock<T> {}


#[cfg(test)]
mod test{
    #[test]
    fn test_basic_lock_unlock() {
        use crate::SpinLock;

        let lock = SpinLock::new(10);

        {
            let mut guard = lock.lock();
            *guard += 5;
            assert_eq!(*guard, 15);
        } // guard dropped here, automatically unlocks

        assert!(!lock.is_locked(), "Lock should be released after guard drop");
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_concurrent_access() {
        use crate::SpinLock;
        use std::sync::Arc;
        use std::thread;

        let lock = Arc::new(SpinLock::new(0usize));
        let mut handles = vec![];

        for _ in 0..8 {
            let lock_cloned = lock.clone();
            handles.push(thread::spawn(move || {
                for _ in 0..10_000 {
                    let mut guard = lock_cloned.lock();
                    *guard += 1;
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        let final_value = *lock.lock();
        assert_eq!(final_value, 8 * 10_000, "Counter should match total increments");
    }

    #[test]
    fn test_try_lock_for_behavior() {
        use crate::SpinLock;

        let lock = SpinLock::new(42);

        // Acquire lock once
        let _guard = lock.lock();

        // Attempt to acquire again (should fail)
        assert!(lock.try_lock_for(10).is_none(), "Lock should not be acquirable while held");

        // After drop, should succeed
        drop(_guard);
        let guard2 = lock.try_lock_for(1000);
        assert!(guard2.is_some(), "Lock should succeed after previous guard drop");
    }


}
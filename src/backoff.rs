//! # BackOff
//!
//! A lightweight, `no_std`-compatible exponential backoff utility for concurrent
//! synchronization primitives (e.g., spinlocks, mutexes, or wait-free algorithms).
//!
//! This module provides a simple and deterministic backoff mechanism that can be
//! used to reduce contention in busy loops by introducing progressively longer
//! delays between retries. It uses [`core::hint::spin_loop`] for CPU pause
//! instructions, and optionally calls [`std::thread::yield_now`] when built
//! with the `std` feature enabled.
//!
//! ## Features
//! - ✅ **`no_std` compatible**
//! - ⚙️ **Exponential spin delay** via doubling
//! - 💡 **Optional yielding** (enabled under the `std` feature)
//! - 🧩 **Configurable starting spin count**
//!
//! ## Example
//! ```rust
//! use axiom_spinlock::BackOff;
//!
//! let backoff = BackOff::new();
//!
//! // Example of use in a spin loop
//! loop {
//!     if try_acquire_lock() {
//!         break;
//!     }
//!     backoff.wait();
//! }
//! 
//! fn try_acquire_lock() -> bool {
//!     // pseudo lock acquisition
//!     true
//! }
//! ```
//!
//! ## Behavior
//! - Each call to [`BackOff::wait`] spins for a number of iterations determined
//!   by the internal counter, which doubles after every call up to a fixed limit.
//! - When compiled with the `std` feature, [`std::thread::yield_now`] is called
//!   once the internal spin count surpasses a yield threshold.
//! - You can reduce spin intensity with [`BackOff::relax`], or reset to start
//!   with [`BackOff::reset`].
//!
//! ## Feature flags
//! - **`std`** — Enables thread yielding when contention persists beyond
//!   a configurable threshold.

use core::{cell::Cell, hint::spin_loop};

/// Maximum spin iteration limit.
const MAX_SPIN: u32 = 1 << 22;

/// Default starting spin count.
const START_VALUE: u32 = 1 << 5;

/// Yield threshold used only under the `std` feature.
#[cfg(feature = "std")]
const YIELD_THRESHOLD: u32 = 1 << 10;

/// Bit shift applied during [`BackOff::relax`] to reduce spin intensity.
const RELAX_DIV_BIT_VAL: u32 = 1;

/// A simple exponential backoff manager.
///
/// This struct maintains an internal counter that controls how long to spin
/// in subsequent retries. Each call to [`wait`](BackOff::wait) increases the
/// spin duration exponentially (up to [`MAX_SPIN`]), which helps alleviate
/// contention under high concurrency.
///
/// On `std` builds, if the spin count grows beyond a threshold, it yields
/// the current thread to allow fair scheduling.
///
/// # Examples
///
/// ```rust
/// use axiom_spinlock::BackOff;
///
/// let backoff = BackOff::new();
///
/// loop {
///     if try_lock() {
///         break;
///     }
///     backoff.wait();
/// }
///
/// fn try_lock() -> bool {
///     // Simulated contention
///     true
/// }
/// ```
pub struct BackOff {
    spin: Cell<u32>,
}

impl BackOff {
    /// Creates a new [`BackOff`] with a default starting spin count.
    ///
    /// # Examples
    /// ```
    /// use axiom_spinlock::BackOff;
    /// let b = BackOff::new();
    /// ```
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            spin: Cell::new(START_VALUE),
        }
    }

    /// Creates a new [`BackOff`] with a custom starting spin value.
    ///
    /// Useful when tuning contention recovery behavior.
    ///
    /// # Examples
    /// ```
    /// use axiom_spinlock::BackOff;
    /// let b = BackOff::new_with(128);
    /// ```
    #[inline(always)]
    pub const fn new_with(start: u32) -> Self {
        Self {
            spin: Cell::new(start),
        }
    }

    /// Performs a backoff wait by spinning for a short, increasing duration.
    ///
    /// The number of spin iterations doubles each time (up to [`MAX_SPIN`]).
    /// Under the `std` feature, this method also calls [`std::thread::yield_now`]
    /// when contention persists beyond a threshold.
    ///
    /// # Examples
    /// ```ignore
    /// use axiom_spinlock::BackOff;
    /// let b = BackOff::new();
    /// b.wait(); // perform first backoff
    /// assert_eq!(b.spin.get()==1<<6);
    /// ```
    #[inline(always)]
    pub fn wait(&self) {
        let end = self.spin.get();

        for _ in 0..end {
            spin_loop();
        }

        self.spin.set((end << 1).min(MAX_SPIN));

        #[cfg(feature = "std")]
        if end > YIELD_THRESHOLD {
            std::thread::yield_now();
        }
    }

    /// Reduces the current spin intensity by a fixed shift.
    ///
    /// This can be used after successful operations or to slowly recover
    /// from aggressive backoff behavior.
    ///
    /// # Examples
    /// ```ignore
    /// use axiom_spinlock::BackOff;
    /// let b = BackOff::new();
    /// b.wait();
    /// b.relax(); // reduce next spin delay
    /// assert_eq!(b.spin.get()==1<<4);
    /// ```
    #[inline(always)]
    pub fn relax(&self) {
        let c_spin = self.spin.get();
        self.spin.set(c_spin >> RELAX_DIV_BIT_VAL);
    }

    /// Returns the current spin iteration value.
    #[inline(always)]
    pub fn current(&self) -> u32 {
        self.spin.get()
    }

    /// Resets the backoff spin count to the default starting value.
    #[inline(always)]
    pub fn reset(&self) {
        self.spin.set(START_VALUE);
    }

    /// Resets the backoff spin count to a specified value.
    #[inline(always)]
    pub fn reset_to(&self, spin: u32) {
        self.spin.set(spin);
    }

    /// Explicitly yields the current thread (only available with `std`).
    ///
    /// Equivalent to calling [`std::thread::yield_now`].
    #[cfg(feature = "std")]
    #[inline]
    pub fn yield_now(&self) {
        std::thread::yield_now();
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    /// Ensures that backoff increases exponentially up to MAX_SPIN.
    #[test]
    fn test_exponential_growth() {
        let b = BackOff::new();

        let mut prev = b.current();
        for _ in 0..23 {
            b.wait();
            let curr = b.current();
            assert!(curr >= prev, "Backoff spin did not grow");
            prev = curr;
        }

        assert!(b.current() <= MAX_SPIN, "Backoff exceeded MAX_SPIN limit");
    }

    /// Ensures that reset correctly restores the default starting value.
    #[test]
    fn test_reset_behavior() {
        let b = BackOff::new();

        // grow the spin a few times
        for _ in 0..5 {
            b.wait();
        }

        assert!(b.current() > START_VALUE);

        b.reset();
        assert_eq!(b.current(), START_VALUE, "Reset did not restore default spin");
    }

    /// Ensures that relax properly reduces the spin intensity.
    #[test]
    fn test_relax_reduces_spin() {
        let b = BackOff::new();

        for _ in 0..5 {
            b.wait();
        }

        let before = b.current();
        b.relax();
        let after = b.current();

        assert!(after < before, "Relax did not reduce spin intensity");
    }
}

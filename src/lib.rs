//! # axiom-spinlock 🌀
//!
//! A lightweight, **`no_std`-compatible** crate providing a minimal and efficient
//! **spin-based synchronization primitive** for low-level concurrent programming.
//!
//! The crate includes:
//!
//! - [`SpinLock<T>`] — a simple, fair spinlock for mutual exclusion.
//! - [`BackOff`] — an adaptive exponential backoff for reducing contention.
//!
//! Designed for environments where blocking is **not an option**—such as kernels,
//! embedded runtimes, or custom executors—this crate avoids OS-level locking
//! and context switching entirely.
//!
//! ## ✨ Features
//!
//! - ✅ `no_std` compatible (uses `core` only)
//! - ⚙️ Optional `std` feature to yield CPU (`std::thread::yield_now()`)
//! - 🧩 Adaptive backoff strategy for high-contention locks
//! - 🔒 Atomic test-and-set spinlock with RAII guard
//!
//! ## 🚀 Quick Example
//!
//! ```rust
//! use axiom_spinlock::{SpinLock, BackOff};
//!
//! // Example 1: Using SpinLock
//! let lock = SpinLock::new(0);
//! {
//!     let mut guard = lock.lock();
//!     *guard += 1;
//! } // automatically unlocked when guard is dropped
//! assert_eq!(*lock.lock(), 1);
//!
//! // Example 2: Using BackOff manually
//! let backoff = BackOff::new();
//! for _ in 0..5 {
//!     backoff.wait();
//! }
//! ```
//!
//! ## 🧠 Design
//!
//! ### SpinLock
//!
//! `SpinLock` uses an [`AtomicBool`] to implement a test-and-set mechanism,
//! ensuring mutual exclusion. It guarantees proper memory ordering using
//! **Acquire/Release** semantics and automatically releases the lock when the
//! [`SpinGuard`] is dropped.
//!
//! ### BackOff
//!
//! `BackOff` performs an adaptive exponential backoff, using
//! [`core::hint::spin_loop()`] to signal the processor that it’s in a busy-wait
//! loop. When compiled with `std`, the backoff escalates to a cooperative
//! `std::thread::yield_now()` after prolonged contention.
//!
//! ## ⚠️ Safety & Usage Notes
//!
//! - Prefer `SpinLock` for **short critical sections** only.  
//! - Never hold a spinlock during blocking or long-running operations.  
//! - `BackOff` is meant to complement spinning mechanisms for fairness and CPU efficiency.  
//! - `SpinLock` is **not reentrant**.
//!
//! ## 📦 Modules
//!
//! - [`backoff`] — Adaptive exponential backoff mechanism.  
//! - [`spinlock`] — Spin-based synchronization primitive.  
//!
//!
//! ### Crate Exports
//!
//! - [`BackOff`] — from [`backoff`]  
//! - [`SpinLock`] — from [`spinlock`]

pub mod backoff;
pub mod spinlock;

pub use backoff::BackOff;
pub use spinlock::SpinLock;

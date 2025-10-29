# axiom-spinlock 🌀

A lightweight, no_std-compatible spinlock and exponential backoff implementation for low-level concurrent systems.

This crate provides a minimal, efficient spin-based mutual exclusion primitive (`SpinLock<T>`) together with an adaptive exponential `BackOff` utility to reduce contention in busy-wait loops.

The implementation focuses on environments where blocking is not an option (kernels, embedded runtimes, custom executors), so it avoids OS-level mutexes and context switching.

---

## Highlights

- no_std compatible (uses `core` only).
- Optional `std` feature (default) to enable thread yielding.
- `SpinLock<T>`: atomic test-and-set spinlock with RAII guard (`SpinGuard`).
- `BackOff`: exponential backoff with configurable start, relax/reset and optional `std` yielding.
- Small, well-tested, and appropriate for short critical sections.

---

## Crate name

This repository builds the crate `axiom-spinlock` (see `Cargo.toml`).

---

## Quick examples

Using `SpinLock`:

```rust
use axiom_spinlock::SpinLock;

let lock = SpinLock::new(0);
{
    let mut guard = lock.lock();
    *guard += 1;
} // unlocked when `guard` is dropped
assert_eq!(*lock.lock(), 1);

// convenience API
lock.with_lock(|v| *v += 1);
```

Using `BackOff`:

```rust
use axiom_spinlock::BackOff;

let backoff = BackOff::new();
for _ in 0..5 {
    backoff.wait();
}
```

---

## API overview

### SpinLock<T>

A minimal spin-based mutual exclusion primitive:

- `const fn new(data: T) -> Self` — create a new lock.
- `fn lock(&self) -> SpinGuard<'_, T>` — acquire the lock (blocks by spinning); returns a guard that releases on drop.
- `unsafe fn unlock(&self)` — unsafely release the lock (only call if you own the lock).
- `fn try_lock(&self) -> Option<SpinGuard<'_, T>>` — try to acquire without blocking.
- `fn try_lock_for(&self, spins: usize) -> Option<SpinGuard<'_, T>>` — attempt to acquire within a fixed number of spin attempts.
- `fn is_locked(&self) -> bool` — check whether the lock is currently held.
- `fn with_lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> R` — convenience wrapper to run a closure while holding the lock.

Notes:
- The lock uses an `AtomicBool` with Acquire/Release ordering.
- The guard implements `Deref` and `DerefMut` for ergonomic access.
- `SpinLock` is marked `Send`/`Sync` when `T: Send`.
- Not reentrant and not fair — starvation is possible under heavy contention.

### BackOff

A simple exponential backoff manager used to reduce contention in spin loops.

- `const fn new() -> BackOff` — default start value.
- `const fn new_with(start: u32) -> BackOff` — create with custom start.
- `fn wait(&self)` — perform one backoff step (spins, doubles internal counter up to `MAX_SPIN`, optionally yields with `std`).
- `fn relax(&self)` — reduce current spin intensity.
- `fn current(&self) -> u32` — get current spin iteration value.
- `fn reset(&self)` — reset to default start.
- `fn reset_to(&self, spin: u32)` — reset to explicit value.
- `#[cfg(feature = "std")] fn yield_now(&self)` — explicit yield (only when compiled with `std`).

Implementation details:
- Uses `core::hint::spin_loop()` to inform the CPU of busy-wait.
- When built with the `std` feature (the crate defaults to enabling this), `std::thread::yield_now()` is called once contention exceeds a threshold.

---

## Example program (from `src/main.rs`)

The repository includes an example program that creates a static `SpinLock<i64>` and spawns 100 threads, each incrementing the shared counter 1_000_000 times. This is useful to stress-test the lock, but be aware it is CPU-intensive.

To run the example (default features include `std`):

```bash
cargo run --release
```

Expect the program to be CPU-bound and run for a while depending on your CPU.

---

## Building & testing

Build the project:

```bash
cargo build
```

Run tests (crate default features include `std` — if you want to test `no_std` behavior you must change features accordingly):

```bash
cargo test
```

---

## Features

- `std` (default): Enables `std::thread::yield_now()` during prolonged backoff and allows examples/tests that spawn threads.

The crate is implemented to be usable without `std` by disabling this feature in embedded or kernel contexts.

---

## Safety notes & recommended usage

- Use `SpinLock` only for short critical sections.
- Avoid holding a spinlock during blocking operations or long computations.
- `SpinLock` is not reentrant.
- `unlock()` is `unsafe` and should only be used by code that truly owns the lock.

---

## Contributing

- The `Cargo.toml` contains author and repository metadata.
- Contributions are welcome; open issues and pull requests on the repository: https://github.com/LOKESH-999/axiom-spinlock

---

## Acknowledgements

This crate is intentionally small and targeted for embedded/low-level use cases. It pairs a tiny spinlock with a configurable backoff to provide a lightweight synchronization primitive where OS-level primitives are unavailable or undesired.

---

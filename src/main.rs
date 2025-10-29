//! Example demonstrating usage of the `SpinLock` from the `axiom-spinlock` crate.
//!
//! This program spawns 100 threads, each incrementing a shared counter
//! a million times. The counter is protected by a `SpinLock<i64>` to ensure
//! safe concurrent mutation without data races.

use axiom_spinlock::SpinLock;
use std::thread;

// Shared static spinlock protecting a 64-bit counter.
static Q: SpinLock<i64> = SpinLock::new(0);

/// Increment the global counter one million times.
/// Each increment safely acquires the lock before modifying the value.
fn add() {
    for _ in 0..1_000_000 {
        *Q.lock() += 1;
    }
}

fn main() {
    println!("Starting spinlock test...");

    // Spawn 100 threads performing concurrent increments.
    let mut threads = Vec::with_capacity(100);
    for _ in 0..100 {
        threads.push(thread::spawn(add));
    }

    // Wait for all threads to finish.
    for t in threads {
        let _ = t.join();
    }

    // Display the final result.
    println!("Final counter value: {}", *Q.lock());
}

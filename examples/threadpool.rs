//! Demonstrates the thread pool and system info utilities.

use solidc::thread::{sysinfo, thread, ThreadPool};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

fn main() {
    // --- System info ---
    println!("=== System Info ===");
    println!("CPUs:     {}", sysinfo::num_cpus());
    println!("PID:      {}", sysinfo::pid());
    println!("PPID:     {}", sysinfo::ppid());
    println!("TID:      {}", sysinfo::tid());
    println!("UID:      {}", sysinfo::uid());
    println!("GID:      {}", sysinfo::gid());
    if let Some(user) = sysinfo::username() {
        println!("User:     {}", user);
    }

    // --- Spawning threads ---
    println!("\n=== Thread Spawn ===");
    let shared = Arc::new(AtomicU64::new(0));

    let handles: Vec<_> = (0..4)
        .map(|i| {
            let s = shared.clone();
            thread::Thread::spawn(move || {
                let val = (i + 1) * 10;
                s.fetch_add(val, Ordering::Relaxed);
            })
            .expect("spawn failed")
        })
        .collect();

    for h in handles {
        h.join().expect("join failed");
    }
    println!("Sum from 4 threads: {} (expected 100)", shared.load(Ordering::Relaxed));

    // --- Thread pool ---
    println!("\n=== Thread Pool ===");
    let ncpus = sysinfo::num_cpus();
    let pool = ThreadPool::new(ncpus).expect("Failed to create thread pool");

    let counter = Arc::new(AtomicU64::new(0));
    let n_tasks = 1000;

    for _ in 0..n_tasks {
        let c = counter.clone();
        pool.submit(move || {
            c.fetch_add(1, Ordering::Relaxed);
        });
    }

    pool.wait();
    println!(
        "Completed {} tasks across {} threads, counter = {}",
        n_tasks,
        ncpus,
        counter.load(Ordering::Relaxed)
    );
}

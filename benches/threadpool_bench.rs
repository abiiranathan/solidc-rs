use criterion::{Criterion, black_box, criterion_group, criterion_main};
use solidc::thread::ThreadPool;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

fn bench_submit_and_wait(c: &mut Criterion) {
    let mut group = c.benchmark_group("threadpool_submit_wait");
    let ncpus = solidc::thread::sysinfo::num_cpus().max(2);
    let n_tasks = 10_000;

    group.bench_function("solidc_threadpool", |b| {
        let pool = ThreadPool::new(ncpus).unwrap();
        b.iter(|| {
            let counter = Arc::new(AtomicU64::new(0));
            for _ in 0..n_tasks {
                let c = counter.clone();
                pool.submit(move || {
                    c.fetch_add(1, Ordering::Relaxed);
                });
            }
            pool.wait();
            black_box(counter.load(Ordering::Relaxed));
        });
    });

    group.bench_function("rust_std_threads", |b| {
        b.iter(|| {
            let counter = Arc::new(AtomicU64::new(0));
            // Batched: spawn ncpus threads, each doing n_tasks/ncpus work
            let tasks_per_thread = n_tasks / ncpus;
            let handles: Vec<_> = (0..ncpus)
                .map(|_| {
                    let c = counter.clone();
                    std::thread::spawn(move || {
                        for _ in 0..tasks_per_thread {
                            c.fetch_add(1, Ordering::Relaxed);
                        }
                    })
                })
                .collect();
            for h in handles {
                h.join().unwrap();
            }
            black_box(counter.load(Ordering::Relaxed));
        });
    });

    group.finish();
}

fn bench_fine_grained_tasks(c: &mut Criterion) {
    let mut group = c.benchmark_group("threadpool_fine_grained");
    let ncpus = solidc::thread::sysinfo::num_cpus().max(2);
    let n_tasks = 1_000;

    group.bench_function("solidc_threadpool", |b| {
        let pool = ThreadPool::new(ncpus).unwrap();
        b.iter(|| {
            let counter = Arc::new(AtomicU64::new(0));
            for _ in 0..n_tasks {
                let c = counter.clone();
                pool.submit(move || {
                    // Small computation
                    let mut x = 0u64;
                    for i in 0..100 {
                        x = x.wrapping_add(black_box(i));
                    }
                    c.fetch_add(x, Ordering::Relaxed);
                });
            }
            pool.wait();
            black_box(counter.load(Ordering::Relaxed));
        });
    });

    group.bench_function("rust_std_spawn_per_task", |b| {
        b.iter(|| {
            let counter = Arc::new(AtomicU64::new(0));
            let handles: Vec<_> = (0..n_tasks)
                .map(|_| {
                    let c = counter.clone();
                    std::thread::spawn(move || {
                        let mut x = 0u64;
                        for i in 0..100 {
                            x = x.wrapping_add(black_box(i));
                        }
                        c.fetch_add(x, Ordering::Relaxed);
                    })
                })
                .collect();
            for h in handles {
                h.join().unwrap();
            }
            black_box(counter.load(Ordering::Relaxed));
        });
    });

    group.finish();
}

criterion_group!(benches, bench_submit_and_wait, bench_fine_grained_tasks);
criterion_main!(benches);

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use solidc::arena::Arena;
use std::alloc::{Layout, alloc, dealloc};

const ARENA_SIZE: usize = 1024 * 1024; // 1 MB

fn bench_arena_many_small_allocs(c: &mut Criterion) {
    let mut group = c.benchmark_group("arena_small_allocs");
    let n = 10_000;

    group.bench_function("solidc_arena", |b| {
        b.iter(|| {
            let mut arena = Arena::new(ARENA_SIZE).unwrap();
            for i in 0..n {
                let val = arena.alloc_value(black_box(i as u64)).unwrap();
                black_box(val);
            }
            arena.reset();
        });
    });

    group.bench_function("rust_global_alloc", |b| {
        b.iter(|| {
            let layout = Layout::new::<u64>();
            let mut ptrs = Vec::with_capacity(n);
            for i in 0..n {
                unsafe {
                    let ptr = alloc(layout) as *mut u64;
                    ptr.write(black_box(i as u64));
                    black_box(&*ptr);
                    ptrs.push(ptr);
                }
            }
            for ptr in ptrs {
                unsafe { dealloc(ptr as *mut u8, layout) };
            }
        });
    });

    group.bench_function("rust_vec_push", |b| {
        b.iter(|| {
            let mut v: Vec<u64> = Vec::with_capacity(n);
            for i in 0..n {
                v.push(black_box(i as u64));
            }
            black_box(&v);
        });
    });

    group.finish();
}

fn bench_arena_bulk_slice(c: &mut Criterion) {
    let mut group = c.benchmark_group("arena_bulk_slice");
    let n = 100_000;

    group.bench_function("solidc_arena_slice", |b| {
        b.iter(|| {
            let mut arena = Arena::new(ARENA_SIZE * 4).unwrap();
            let slice = arena.alloc_slice::<u64>(n).unwrap();
            for (i, slot) in slice.iter_mut().enumerate() {
                *slot = black_box(i as u64);
            }
            black_box(slice);
            arena.reset();
        });
    });

    group.bench_function("rust_vec_alloc", |b| {
        b.iter(|| {
            let mut v = vec![0u64; n];
            for (i, slot) in v.iter_mut().enumerate() {
                *slot = black_box(i as u64);
            }
            black_box(&v);
        });
    });

    group.finish();
}

fn bench_arena_reset_reuse(c: &mut Criterion) {
    let mut group = c.benchmark_group("arena_reset_reuse");
    let rounds = 100;
    let allocs_per_round = 1000;

    group.bench_function("solidc_arena_reset", |b| {
        let mut arena = Arena::new(ARENA_SIZE).unwrap();
        b.iter(|| {
            for _ in 0..rounds {
                for i in 0..allocs_per_round {
                    let val = arena.alloc_value(black_box(i as u64)).unwrap();
                    black_box(val);
                }
                arena.reset();
            }
        });
    });

    group.bench_function("rust_vec_clear_reuse", |b| {
        let mut v: Vec<u64> = Vec::with_capacity(allocs_per_round);
        b.iter(|| {
            for _ in 0..rounds {
                for i in 0..allocs_per_round {
                    v.push(black_box(i as u64));
                }
                v.clear();
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_arena_many_small_allocs,
    bench_arena_bulk_slice,
    bench_arena_reset_reuse
);
criterion_main!(benches);

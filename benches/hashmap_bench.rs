use criterion::{Criterion, black_box, criterion_group, criterion_main};
use solidc::map::HashMap as SolidcMap;
use std::collections::HashMap as StdMap;

fn generate_keys(n: usize) -> Vec<String> {
    (0..n).map(|i| format!("key_{:06}", i)).collect()
}

fn bench_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("hashmap_insert");
    let n = 10_000;
    let keys = generate_keys(n);

    group.bench_function("solidc_map", |b| {
        b.iter(|| {
            let mut map = SolidcMap::new().unwrap();
            for key in &keys {
                map.insert_str(black_box(key), black_box("value"));
            }
            black_box(map.len());
        });
    });

    group.bench_function("rust_std_hashmap", |b| {
        b.iter(|| {
            let mut map = StdMap::new();
            for key in &keys {
                map.insert(black_box(key.clone()), black_box("value".to_string()));
            }
            black_box(map.len());
        });
    });

    group.finish();
}

fn bench_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("hashmap_lookup");
    let n = 10_000;
    let keys = generate_keys(n);

    // Pre-populate solidc map
    let mut sc_map = SolidcMap::with_capacity(n * 2).unwrap();
    for key in &keys {
        sc_map.insert_str(key, "value");
    }

    // Pre-populate std map
    let mut std_map = StdMap::with_capacity(n * 2);
    for key in &keys {
        std_map.insert(key.clone(), "value".to_string());
    }

    group.bench_function("solidc_map", |b| {
        b.iter(|| {
            for key in &keys {
                black_box(sc_map.get_str(black_box(key)));
            }
        });
    });

    group.bench_function("rust_std_hashmap", |b| {
        b.iter(|| {
            for key in &keys {
                black_box(std_map.get(black_box(key)));
            }
        });
    });

    group.finish();
}

fn bench_mixed_insert_remove(c: &mut Criterion) {
    let mut group = c.benchmark_group("hashmap_mixed");
    let n = 5_000;
    let keys = generate_keys(n * 2);

    group.bench_function("solidc_map", |b| {
        b.iter(|| {
            let mut map = SolidcMap::new().unwrap();
            // Insert all
            for key in &keys[..n] {
                map.insert_str(key, "val");
            }
            // Remove half, insert new half
            for i in 0..n / 2 {
                map.remove(&keys[i]);
                map.insert_str(&keys[n + i], "val2");
            }
            black_box(map.len());
        });
    });

    group.bench_function("rust_std_hashmap", |b| {
        b.iter(|| {
            let mut map = StdMap::new();
            for key in &keys[..n] {
                map.insert(key.clone(), "val".to_string());
            }
            for i in 0..n / 2 {
                map.remove(&keys[i]);
                map.insert(keys[n + i].clone(), "val2".to_string());
            }
            black_box(map.len());
        });
    });

    group.finish();
}

fn bench_contains(c: &mut Criterion) {
    let mut group = c.benchmark_group("hashmap_contains");
    let n = 10_000;
    let keys = generate_keys(n);
    let missing_keys = generate_keys(n)
        .iter()
        .map(|k| format!("miss_{}", k))
        .collect::<Vec<_>>();

    let mut sc_map = SolidcMap::with_capacity(n * 2).unwrap();
    let mut std_map = StdMap::with_capacity(n * 2);
    for key in &keys {
        sc_map.insert_str(key, "v");
        std_map.insert(key.clone(), "v".to_string());
    }

    group.bench_function("solidc_hit_miss", |b| {
        b.iter(|| {
            for key in &keys[..100] {
                black_box(sc_map.contains_key(key));
            }
            for key in &missing_keys[..100] {
                black_box(sc_map.contains_key(key));
            }
        });
    });

    group.bench_function("rust_std_hit_miss", |b| {
        b.iter(|| {
            for key in &keys[..100] {
                black_box(std_map.contains_key(key.as_str()));
            }
            for key in &missing_keys[..100] {
                black_box(std_map.contains_key(key.as_str()));
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_insert,
    bench_lookup,
    bench_mixed_insert_remove,
    bench_contains
);
criterion_main!(benches);

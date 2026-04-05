use criterion::{Criterion, black_box, criterion_group, criterion_main};
use solidc::cstr::CStr_;

fn bench_small_string_create(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_create_small");
    let s = "hello world"; // 11 bytes, fits SSO

    group.bench_function("solidc_cstr_sso", |b| {
        b.iter(|| {
            let cs = CStr_::new(black_box(s)).unwrap();
            black_box(cs.as_str());
        });
    });

    group.bench_function("rust_string", |b| {
        b.iter(|| {
            let rs = String::from(black_box(s));
            black_box(rs.as_str());
        });
    });

    group.finish();
}

fn bench_large_string_create(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_create_large");
    let s = "a]".repeat(100); // 200 bytes, heap

    group.bench_function("solidc_cstr_heap", |b| {
        let src = s.clone();
        b.iter(|| {
            let cs = CStr_::new(black_box(&src)).unwrap();
            black_box(cs.as_str());
        });
    });

    group.bench_function("rust_string", |b| {
        let src = s.clone();
        b.iter(|| {
            let rs = String::from(black_box(src.as_str()));
            black_box(rs.as_str());
        });
    });

    group.finish();
}

fn bench_string_append(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_append");
    let n = 1000;

    group.bench_function("solidc_cstr_push_str", |b| {
        b.iter(|| {
            let mut cs = CStr_::with_capacity(n * 6).unwrap();
            for _ in 0..n {
                cs.push_str(black_box("hello "));
            }
            black_box(cs.len());
        });
    });

    group.bench_function("rust_string_push_str", |b| {
        b.iter(|| {
            let mut rs = String::with_capacity(n * 6);
            for _ in 0..n {
                rs.push_str(black_box("hello "));
            }
            black_box(rs.len());
        });
    });

    group.finish();
}

fn bench_string_find(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_find");
    let haystack_str = "the quick brown fox jumps over the lazy dog ".repeat(50);

    group.bench_function("solidc_cstr_find", |b| {
        let cs = CStr_::new(&haystack_str).unwrap();
        b.iter(|| {
            black_box(cs.find(black_box("lazy")));
        });
    });

    group.bench_function("rust_str_find", |b| {
        let s = haystack_str.clone();
        b.iter(|| {
            black_box(s.find(black_box("lazy")));
        });
    });

    group.finish();
}

fn bench_string_case_conversion(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_case");

    group.bench_function("solidc_cstr_upper", |b| {
        b.iter(|| {
            let mut cs = CStr_::new(black_box("hello world from solidc")).unwrap();
            cs.to_uppercase();
            black_box(cs.as_str());
        });
    });

    group.bench_function("rust_string_to_uppercase", |b| {
        b.iter(|| {
            let rs = black_box("hello world from solidc").to_uppercase();
            black_box(rs.as_str());
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_small_string_create,
    bench_large_string_create,
    bench_string_append,
    bench_string_find,
    bench_string_case_conversion
);
criterion_main!(benches);

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use solidc::xtime::Time;
use std::time::SystemTime;

fn bench_now(c: &mut Criterion) {
    let mut group = c.benchmark_group("time_now");

    group.bench_function("solidc_xtime_now", |b| {
        b.iter(|| {
            let t = Time::now().unwrap();
            black_box(t.to_unix());
        });
    });

    group.bench_function("rust_systemtime_now", |b| {
        b.iter(|| {
            let t = SystemTime::now();
            let unix = t.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
            black_box(unix);
        });
    });

    group.finish();
}

fn bench_format(c: &mut Criterion) {
    let mut group = c.benchmark_group("time_format");

    let xt = Time::now().unwrap();
    group.bench_function("solidc_xtime_format", |b| {
        b.iter(|| {
            let s = xt.format(black_box("%Y-%m-%d %H:%M:%S")).unwrap();
            black_box(s);
        });
    });

    // Rust std has no built-in formatting — we format manually for fairness
    group.bench_function("rust_manual_format", |b| {
        let now = SystemTime::now();
        let secs = now
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        b.iter(|| {
            // Simulate formatting a unix timestamp (no strftime in std)
            let s = format!(
                "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                1970 + black_box(secs) / 31557600,
                (black_box(secs) % 31557600) / 2629800 + 1,
                (black_box(secs) % 2629800) / 86400 + 1,
                (black_box(secs) % 86400) / 3600,
                (black_box(secs) % 3600) / 60,
                black_box(secs) % 60,
            );
            black_box(s);
        });
    });

    group.finish();
}

fn bench_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("time_parse");

    group.bench_function("solidc_xtime_parse", |b| {
        b.iter(|| {
            let t = Time::parse(
                black_box("2025-12-25 14:30:00"),
                black_box("%Y-%m-%d %H:%M:%S"),
            )
            .unwrap();
            black_box(t.to_unix());
        });
    });

    // Rust std has no date parsing — manual parse for comparison
    group.bench_function("rust_manual_parse", |b| {
        b.iter(|| {
            let s = black_box("2025-12-25 14:30:00");
            let parts: Vec<&str> = s.split(|c| c == '-' || c == ' ' || c == ':').collect();
            let year: u32 = parts[0].parse().unwrap();
            let month: u32 = parts[1].parse().unwrap();
            let day: u32 = parts[2].parse().unwrap();
            let hour: u32 = parts[3].parse().unwrap();
            let min: u32 = parts[4].parse().unwrap();
            let sec: u32 = parts[5].parse().unwrap();
            black_box((year, month, day, hour, min, sec));
        });
    });

    group.finish();
}

fn bench_arithmetic(c: &mut Criterion) {
    let mut group = c.benchmark_group("time_arithmetic");

    group.bench_function("solidc_add_days_months", |b| {
        b.iter(|| {
            let mut t = Time::from_unix(black_box(1700000000)).unwrap();
            t.add_days(30).unwrap();
            t.add_months(3).unwrap();
            t.add_hours(5).unwrap();
            black_box(t.to_unix());
        });
    });

    group.bench_function("rust_duration_add", |b| {
        b.iter(|| {
            let t = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(black_box(1700000000));
            // std only supports Duration (no calendar arithmetic)
            let t2 = t + std::time::Duration::from_secs(30 * 86400); // ~30 days
            let t3 = t2 + std::time::Duration::from_secs(90 * 86400); // ~3 months
            let t4 = t3 + std::time::Duration::from_secs(5 * 3600); // 5 hours
            black_box(t4.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs());
        });
    });

    group.finish();
}

fn bench_compare(c: &mut Criterion) {
    let mut group = c.benchmark_group("time_compare");
    let n = 1000;

    group.bench_function("solidc_xtime_compare", |b| {
        let times: Vec<Time> = (0..n)
            .map(|i| Time::from_unix(1700000000 + i * 3600).unwrap())
            .collect();
        b.iter(|| {
            for w in times.windows(2) {
                black_box(w[0].compare(&w[1]));
            }
        });
    });

    group.bench_function("rust_systemtime_cmp", |b| {
        let times: Vec<SystemTime> = (0..n)
            .map(|i| {
                SystemTime::UNIX_EPOCH
                    + std::time::Duration::from_secs(1700000000 + i as u64 * 3600)
            })
            .collect();
        b.iter(|| {
            for w in times.windows(2) {
                black_box(w[0].cmp(&w[1]));
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_now,
    bench_format,
    bench_parse,
    bench_arithmetic,
    bench_compare
);
criterion_main!(benches);

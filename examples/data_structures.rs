//! Demonstrates the arena allocator, hash map, and cache.
//!
//! Shows bump allocation, a key-value store, and a TTL cache.

use solidc::arena::Arena;
use solidc::cache::Cache;
use solidc::map::HashMap;

fn main() {
    // --- Arena allocator ---
    println!("=== Arena Allocator ===");
    let mut arena = Arena::new(64 * 1024).expect("Failed to create arena");

    // Allocate individual values
    let x = arena.alloc_value(42u64).expect("alloc failed");
    let y = arena.alloc_value(3.14f64).expect("alloc failed");
    println!("Allocated u64: {}, f64: {}", *x, *y);

    // Allocate a slice
    let nums = arena.alloc_slice::<u32>(10).expect("alloc_slice failed");
    for (i, slot) in nums.iter_mut().enumerate() {
        *slot = (i * i) as u32;
    }
    println!("Squares: {:?}", nums);
    println!(
        "Arena used: {} bytes, committed: {} bytes",
        arena.used_size(),
        arena.committed_size()
    );

    // Reset reclaims all memory without deallocating
    arena.reset();
    println!("After reset: {} bytes used", arena.used_size());

    // --- HashMap ---
    println!("\n=== HashMap ===");
    let mut map = HashMap::new().expect("Failed to create map");

    let entries = [
        ("rust", "systems programming"),
        ("python", "scripting"),
        ("go", "cloud infrastructure"),
        ("c", "operating systems"),
        ("zig", "low-level programming"),
    ];

    for (lang, desc) in &entries {
        map.insert_str(lang, desc);
    }

    println!("Map has {} entries", map.len());

    for lang in &["rust", "c", "python", "java"] {
        match map.get_str(lang) {
            Some(desc) => println!("  {} -> {}", lang, desc),
            None => println!("  {} -> (not found)", lang),
        }
    }

    map.remove("python");
    println!(
        "After removing 'python': {} entries, contains python? {}",
        map.len(),
        map.contains_key("python")
    );

    // --- Cache with TTL ---
    println!("\n=== TTL Cache ===");
    let cache = Cache::new(100, 5).expect("Failed to create cache");

    cache.set_str("session:abc123", "user=alice");
    cache.set_str("session:def456", "user=bob");
    cache.set_str("config:theme", "dark");

    println!("Cache size: {} / {}", cache.size(), cache.capacity());
    println!("session:abc123 = {:?}", cache.get_str("session:abc123"));
    println!("config:theme = {:?}", cache.get_str("config:theme"));

    cache.invalidate("session:abc123");
    println!(
        "After invalidation: session:abc123 = {:?}",
        cache.get_str("session:abc123")
    );

    // Persist and reload
    let cache_file = "/tmp/solidc_example_cache.bin";
    if cache.save(cache_file) {
        println!("Cache saved to {}", cache_file);
        let cache2 = Cache::new(100, 5).unwrap();
        if cache2.load(cache_file) {
            println!(
                "Cache loaded: config:theme = {:?}",
                cache2.get_str("config:theme")
            );
        }
        std::fs::remove_file(cache_file).ok();
    }
}

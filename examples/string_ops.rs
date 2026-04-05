//! Demonstrates CStr_ string operations: SSO, case conversion, find/replace, split.

use solidc::cstr::CStr_;

fn main() {
    // --- Small String Optimization ---
    println!("=== Small String Optimization ===");
    let small = CStr_::new("hello").unwrap();
    let large = CStr_::new(&"x".repeat(64)).unwrap();
    println!(
        "'hello' heap-allocated: {} (len {})",
        small.is_heap_allocated(),
        small.len()
    );
    println!(
        "64-byte string heap-allocated: {} (len {})",
        large.is_heap_allocated(),
        large.len()
    );

    // --- Building strings ---
    println!("\n=== String Building ===");
    let mut s = CStr_::with_capacity(64).unwrap();
    s.push_str("Hello");
    s.push_str(", ");
    s.push_str("World!");
    println!("Built: '{}' (len {})", s, s.len());

    s.prepend(">> ");
    s.push_str(" <<");
    println!("Decorated: '{}'", s);

    // --- Case conversion ---
    println!("\n=== Case Conversion ===");
    let mut title = CStr_::new("the quick brown fox").unwrap();
    title.to_titlecase();
    println!("Title case: '{}'", title);

    let mut snake = CStr_::new("myVariableName").unwrap();
    snake.to_snakecase();
    println!("Snake case: '{}'", snake);

    let mut camel = CStr_::new("some_function_name").unwrap();
    camel.to_camelcase();
    println!("Camel case: '{}'", camel);

    let mut upper = CStr_::new("loud").unwrap();
    upper.to_uppercase();
    println!("Upper: '{}'", upper);

    // --- Find and replace ---
    println!("\n=== Find & Replace ===");
    let text = CStr_::new("the cat sat on the mat").unwrap();
    println!("Original: '{}'", text);

    if let Some(pos) = text.find("cat") {
        println!("Found 'cat' at position {}", pos);
    }

    let text = text.replace("the", "a").unwrap();
    println!("After replace 'the'->'a': '{}'", text);

    // --- Trim and manipulation ---
    println!("\n=== Trim & Manipulate ===");
    let mut padded = CStr_::new("   spaces everywhere   ").unwrap();
    padded.trim();
    println!("Trimmed: '{}'", padded);

    let csv_line = CStr_::new("apple,banana,cherry,date").unwrap();
    let parts = csv_line.split(",");
    println!("Split 'apple,banana,cherry,date':");
    for part in &parts {
        println!("  - {}", part);
    }

    // --- Reverse and contains ---
    println!("\n=== Misc ===");
    let mut rev = CStr_::new("desserts").unwrap();
    rev.reverse();
    println!("'desserts' reversed: '{}'", rev);

    let haystack = CStr_::new("Rust is awesome").unwrap();
    println!("Contains 'awesome': {}", haystack.contains("awesome"));
    println!("Starts with 'Rust': {}", haystack.starts_with("Rust"));
    println!("Ends with 'some': {}", haystack.ends_with("some"));
}

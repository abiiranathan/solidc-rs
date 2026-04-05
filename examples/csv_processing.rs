//! Demonstrates CSV reading and writing.

use solidc::csvparser::{CsvReader, CsvWriter, ReaderConfig};
use std::io::Write;

fn main() {
    let csv_path = "/tmp/solidc_example.csv";

    // --- Write a CSV file ---
    println!("=== CSV Writer ===");
    {
        let mut f = std::fs::File::create(csv_path).expect("create file");
        writeln!(f, "name,age,city").unwrap();
        writeln!(f, "Alice,30,Portland").unwrap();
        writeln!(f, "Bob,25,\"New York\"").unwrap();
        writeln!(f, "Charlie,35,Seattle").unwrap();
        writeln!(f, "Diana,28,Chicago").unwrap();
    }
    println!("Wrote sample CSV to {}", csv_path);

    // --- Read with CsvReader ---
    println!("\n=== CSV Reader ===");
    let config = ReaderConfig {
        delim: ',',
        has_header: true,
        skip_header: true,
        ..Default::default()
    };

    let mut reader = CsvReader::open(csv_path, 4096).expect("Failed to open CSV");
    reader.set_config(&config);

    let rows = reader.parse();
    for row in &rows {
        let name = row.get(0).unwrap_or("?");
        let age = row.get(1).unwrap_or("?");
        let city = row.get(2).unwrap_or("?");
        println!("  {} (age {}) lives in {}", name, age, city);
    }

    // --- Write programmatically with CsvWriter ---
    println!("\n=== CSV Writer (programmatic) ===");
    let out_path = "/tmp/solidc_example_out.csv";
    if let Some(mut writer) = CsvWriter::new(out_path) {
        writer.write_row(&["language", "paradigm", "year"]);
        writer.write_row(&["Rust", "systems", "2010"]);
        writer.write_row(&["C", "procedural", "1972"]);
        writer.write_row(&["Haskell", "functional", "1990"]);
        println!("Wrote TSV to {}", out_path);
    }

    // Verify the output
    let contents = std::fs::read_to_string(out_path).unwrap_or_default();
    println!("Output contents:\n{}", contents);

    // Cleanup
    std::fs::remove_file(csv_path).ok();
    std::fs::remove_file(out_path).ok();
}

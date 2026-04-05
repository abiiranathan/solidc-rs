use solidc::file::{File, FileError, Result, format_file_size, get_file_size_by_path};
use std::io::{Read, Seek, SeekFrom, Write};

fn main() {
    println!("=== File API Usage Examples ===\n");

    // Example 1: Basic file creation and writing
    if let Err(e) = example_basic_write() {
        eprintln!("Basic write example failed: {}", e);
    }

    // Example 2: Reading from a file
    if let Err(e) = example_read() {
        eprintln!("Read example failed: {}", e);
    }

    // Example 3: Using std::io traits
    if let Err(e) = example_std_io_traits() {
        eprintln!("Std IO traits example failed: {}", e);
    }

    // Example 4: Positional read/write (pread/pwrite)
    if let Err(e) = example_positional_io() {
        eprintln!("Positional I/O example failed: {}", e);
    }

    // Example 5: File locking
    if let Err(e) = example_file_locking() {
        eprintln!("File locking example failed: {}", e);
    }

    // Example 6: Read entire file
    if let Err(e) = example_read_all() {
        eprintln!("Read all example failed: {}", e);
    }

    // Example 7: Memory mapping
    if let Err(e) = example_mmap() {
        eprintln!("Memory mapping example failed: {}", e);
    }

    // Example 8: File size utilities
    if let Err(e) = example_file_size_utils() {
        eprintln!("File size utilities example failed: {}", e);
    }

    // Example 9: File truncation
    if let Err(e) = example_truncate() {
        eprintln!("Truncate example failed: {}", e);
    }

    // Example 10: File copying
    if let Err(e) = example_copy() {
        eprintln!("Copy example failed: {}", e);
    }

    println!("\n=== All examples completed ===");
}

/// Example 1: Basic file creation and writing
fn example_basic_write() -> Result<()> {
    println!("1. Basic Write Example:");

    let mut file = File::open("example1.txt", "w")?;

    // Write raw bytes
    let data = b"Hello, World!\n";
    let written = file.write(data)?;
    println!("   Wrote {} bytes", written);

    // Write a string
    let str_written = file.write_str("This is a test.\n")?;
    println!("   Wrote {} bytes from string", str_written);

    // Always flush to ensure data is written
    file.flush()?;
    println!("   File flushed successfully\n");

    Ok(())
}

/// Example 2: Reading from a file
fn example_read() -> Result<()> {
    println!("2. Read Example:");

    let file = File::open("example1.txt", "r")?;

    let mut buffer = vec![0u8; 100];
    let bytes_read = file.read(&mut buffer)?;
    buffer.truncate(bytes_read);

    println!(
        "   Read {} bytes: {:?}",
        bytes_read,
        String::from_utf8_lossy(&buffer)
    );
    println!();

    Ok(())
}

/// Example 3: Using std::io::Read, Write, Seek traits
fn example_std_io_traits() -> Result<()> {
    println!("3. Using std::io Traits:");

    let mut file = File::open("example3.txt", "w+")?;

    // Use Write trait
    file.write_all(b"First line\n")?;
    file.write_all(b"Second line\n")?;
    file.write_all(b"Third line\n")?;
    file.flush()?;

    // Use Seek trait to go back to start
    file.seek(SeekFrom::Start(0))?;

    // Use Read trait
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    println!("   File contents:\n{}", contents);

    // Seek to end and get position
    let end_pos = file.seek(SeekFrom::End(0))?;
    println!("   File size: {} bytes\n", end_pos);

    Ok(())
}

/// Example 4: Positional I/O (read/write without changing file position)
fn example_positional_io() -> Result<()> {
    println!("4. Positional I/O Example:");

    let mut file = File::open("example4.txt", "w+")?;

    // Write some initial data
    file.write(b"0123456789ABCDEFGHIJ")?;
    file.flush()?;

    // Write at specific offset without changing position
    let data = b"XXXX";
    let written = file.pwrite(data, 5)?;
    println!("   Wrote {} bytes at offset 5", written);

    // Read from specific offset without changing position
    let mut buffer = vec![0u8; 4];
    let read = file.pread(&mut buffer, 5)?;
    println!(
        "   Read {} bytes at offset 5: {:?}",
        read,
        String::from_utf8_lossy(&buffer)
    );

    // Current position should still be at end
    let pos = file.tell()?;
    println!("   Current file position: {}\n", pos);

    Ok(())
}

/// Example 5: File locking for concurrent access
fn example_file_locking() -> Result<()> {
    println!("5. File Locking Example:");

    let file = File::open("example5.txt", "w")?;

    // Acquire exclusive lock
    file.lock()?;
    println!("   Lock acquired");

    // Release lock
    file.unlock()?;
    println!("   Lock released\n");

    Ok(())
}

/// Example 6: Read entire file into memory
fn example_read_all() -> Result<()> {
    println!("6. Read All Example:");

    // Create a test file first
    let mut file = File::open("example6.txt", "w")?;
    file.write(b"This is the entire file contents.\n")?;
    file.write(b"It will be read in one operation.\n")?;
    file.flush()?;
    drop(file); // Close the write handle

    // Read entire file
    let mut file = File::open("example6.txt", "r")?;
    let contents = file.read_all()?;

    println!("   Read {} bytes total", contents.len());
    println!("   Contents: {:?}\n", String::from_utf8_lossy(&contents));

    Ok(())
}

/// Example 7: Memory mapping for efficient large file access
fn example_mmap() -> Result<()> {
    println!("7. Memory Mapping Example:");

    // Create a file with known content
    let mut file = File::open("example7.txt", "w+")?;
    let data = b"Memory mapped file contents! This is very efficient for large files.";
    file.write(data)?;
    file.flush()?;

    // Get file size
    let size = file.size()? as usize;
    println!("   File size: {} bytes", size);

    // Memory map the file with read access
    let mmap = file.mmap(size, true, false)?;
    let slice = mmap.as_slice();

    println!("   Mapped {} bytes", slice.len());
    println!(
        "   First 20 bytes: {:?}",
        String::from_utf8_lossy(&slice[..20])
    );

    // mmap is automatically unmapped when it goes out of scope
    drop(mmap);
    println!("   Memory mapping released\n");

    Ok(())
}

/// Example 8: File size utilities
fn example_file_size_utils() -> Result<()> {
    println!("8. File Size Utilities:");

    // Create a test file
    let mut file = File::open("example8.txt", "w")?;
    file.write(&vec![0u8; 2048])?; // Write 2KB
    file.flush()?;
    drop(file);

    // Get size via file handle
    let file = File::open("example8.txt", "r")?;
    let size = file.size()?;
    println!("   File size (via handle): {} bytes", size);

    // Get size via path (without opening)
    let size = get_file_size_by_path("example8.txt")?;
    println!("   File size (via path): {} bytes", size);

    // Format size as human-readable
    let formatted = format_file_size(size as u64)?;
    println!("   Human-readable size: {}\n", formatted);

    Ok(())
}

/// Example 9: File truncation
fn example_truncate() -> Result<()> {
    println!("9. Truncate Example:");

    // Create a file with content
    let mut file = File::open("example9.txt", "w+")?;
    file.write(b"This file will be truncated.")?;
    file.flush()?;

    let original_size = file.size()?;
    println!("   Original size: {} bytes", original_size);

    // Truncate to 10 bytes
    file.truncate(10)?;
    let new_size = file.size()?;
    println!("   Size after truncate: {} bytes", new_size);

    // Read the truncated content
    file.seek_raw(0, 0)?; // SEEK_SET
    let contents = file.read_all()?;
    println!(
        "   Truncated contents: {:?}\n",
        String::from_utf8_lossy(&contents)
    );

    Ok(())
}

/// Example 10: File copying
fn example_copy() -> Result<()> {
    println!("10. Copy Example:");

    // Create source file
    let mut src = File::open("example10_src.txt", "w")?;
    src.write(b"This content will be copied to another file.\n")?;
    src.flush()?;
    drop(src);

    // Open source for reading and destination for writing
    let src = File::open("example10_src.txt", "r")?;
    let mut dst = File::open("example10_dst.txt", "w")?;

    // Copy contents
    src.copy_to(&mut dst)?;
    dst.flush()?;

    println!("   File copied successfully");

    // Verify
    let dst_size = dst.size()?;
    println!("   Destination file size: {} bytes\n", dst_size);

    Ok(())
}

/// Example 11: Error handling patterns
#[allow(dead_code)]
fn example_error_handling() {
    println!("11. Error Handling Patterns:\n");

    // Pattern 1: Basic match
    match File::open("nonexistent.txt", "r") {
        Ok(file) => println!("   File opened: size = {}", file.size().unwrap_or(-1)),
        Err(FileError::OpenFailed) => println!("   Expected: file does not exist"),
        Err(e) => println!("   Unexpected error: {}", e),
    }

    // Pattern 2: Using ? operator for early return
    fn risky_operation() -> Result<()> {
        let mut file = File::open("test.txt", "w")?;
        file.write(b"data")?;
        file.flush()?;
        Ok(())
    }

    if let Err(e) = risky_operation() {
        eprintln!("   Operation failed: {}", e);
    }

    // Pattern 3: Chaining operations
    let result = File::open("chain.txt", "w+")
        .and_then(|mut f| {
            f.write(b"test")?;
            f.flush()?;
            Ok(f)
        })
        .and_then(|f| f.size());

    match result {
        Ok(size) => println!("   Chained operations succeeded, size: {}", size),
        Err(e) => println!("   Chained operations failed: {}", e),
    }

    println!();
}

/// Example 12: Working with binary data
#[allow(dead_code)]
fn example_binary_data() -> Result<()> {
    println!("12. Binary Data Example:");

    let mut file = File::open("binary.dat", "wb")?;

    // Write different binary data types
    let header: u32 = 0xDEADBEEF;
    file.write(&header.to_le_bytes())?;

    let values: Vec<f64> = vec![3.14159, 2.71828, 1.41421];
    for value in &values {
        file.write(&value.to_le_bytes())?;
    }

    file.flush()?;
    println!("   Binary data written");

    // Read it back
    file.seek_raw(0, 0)?;

    let mut header_bytes = [0u8; 4];
    file.read(&mut header_bytes)?;
    let read_header = u32::from_le_bytes(header_bytes);
    println!("   Header: 0x{:X}", read_header);

    println!();
    Ok(())
}

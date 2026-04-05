//! Demonstrates process spawning with pipe-based IPC.
//!
//! Spawns a child process, captures its stdout via a pipe, and prints the output.
//! Also shows how to run a command with file redirection.

use solidc::process::{self, Pipe, Process, ProcessIO, SpawnOptions};

fn main() {
    // --- Example 1: Simple command execution ---
    println!("=== Running /bin/echo via process::run ===");
    match process::run("/bin/echo", &["echo", "Hello from solidc!"]) {
        Ok(code) => println!("Exit code: {}", code),
        Err(e) => eprintln!("Error: {} - {}", e, process::error_string(e)),
    }

    // --- Example 2: Spawn with pipe to capture stdout ---
    println!("\n=== Capturing stdout via pipe ===");
    let mut stdout_pipe = Pipe::new().expect("Failed to create pipe");

    let opts = SpawnOptions {
        io: ProcessIO {
            stdout_pipe: Some(stdout_pipe.as_ptr()),
            ..ProcessIO::default()
        },
        ..SpawnOptions::default()
    };

    let child = Process::spawn_with_options("/bin/ls", &["ls", "-1", "/tmp"], &opts)
        .expect("Failed to spawn ls");

    let result = child.wait(-1).expect("Wait failed");
    stdout_pipe
        .close_write()
        .expect("Failed to close write end");

    let mut output = Vec::new();
    let mut buf = [0u8; 4096];
    loop {
        match stdout_pipe.read(&mut buf, 100) {
            Ok(0) => break,
            Ok(n) => output.extend_from_slice(&buf[..n]),
            Err(_) => break,
        }
    }

    let text = String::from_utf8_lossy(&output);
    let lines: Vec<&str> = text.lines().collect();
    println!(
        "ls /tmp returned {} files (exit code {})",
        lines.len(),
        result.exit_code
    );
    for line in lines.iter().take(5) {
        println!("  {}", line);
    }
    if lines.len() > 5 {
        println!("  ... and {} more", lines.len() - 5);
    }

    // --- Example 3: Redirect output to a file ---
    println!("\n=== Redirecting output to file ===");
    let outfile = "/tmp/solidc_example_output.txt";
    let proc =
        process::run_with_file_redirection("/bin/date", &["date"], Some(outfile), None, false)
            .expect("Failed to spawn date");

    proc.wait(-1).expect("Wait failed");
    let contents = std::fs::read_to_string(outfile).expect("Failed to read output file");
    println!("Date output saved to {}: {}", outfile, contents.trim());
    std::fs::remove_file(outfile).ok();

    // --- Example 4: Pipe-based IPC between parent and child ---
    println!("\n=== Pipe write/read (in-process) ===");
    let pipe = Pipe::new().expect("Failed to create pipe");
    let message = b"Hello through the pipe!";
    pipe.write(message, -1).expect("Write failed");

    let mut recv = [0u8; 64];
    let n = pipe.read(&mut recv, -1).expect("Read failed");
    println!(
        "Sent {} bytes, received: {}",
        message.len(),
        std::str::from_utf8(&recv[..n]).unwrap()
    );
}

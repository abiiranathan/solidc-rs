//! Demonstrates Unix command pipelines (e.g., ls | grep | wc).

#[cfg(unix)]
use solidc::pipeline::Pipeline;

fn main() {
    #[cfg(not(unix))]
    {
        println!("Pipeline examples require Unix.");
        return;
    }

    #[cfg(unix)]
    {
        // --- Simple pipeline: echo | tr ---
        println!("=== echo 'hello world' | tr ' ' '\\n' ===");
        Pipeline::new()
            .add(&["echo", "hello world"])
            .add(&["tr", " ", "\n"])
            .execute();
        println!();

        // --- Multi-stage: find files, filter, count ---
        println!("=== ls /usr/bin | grep -c '^g' ===");
        Pipeline::new()
            .add(&["ls", "/usr/bin"])
            .add(&["grep", "^g"])
            .add(&["wc", "-l"])
            .execute();
        println!();

        // --- Capture output to a file ---
        println!("=== Capture pipeline to file ===");
        let out_path = "/tmp/solidc_pipeline_out.txt";
        if let Ok(f) = std::fs::File::create(out_path) {
            use std::os::unix::io::AsRawFd;
            Pipeline::new()
                .add(&["echo", "captured output from pipeline"])
                .execute_to_fd(f.as_raw_fd());
            drop(f);
            let contents = std::fs::read_to_string(out_path).unwrap_or_default();
            println!("Wrote to file: {}", contents.trim());
            std::fs::remove_file(out_path).ok();
        }

        // --- Sort and unique ---
        println!("\n=== printf | sort | uniq ===");
        Pipeline::new()
            .add(&["printf", "banana\napple\ncherry\napple\nbanana\n"])
            .add(&["sort"])
            .add(&["uniq"])
            .execute();
    }
}

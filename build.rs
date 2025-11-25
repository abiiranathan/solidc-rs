// build.rs
use bindgen::Builder;
use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-search=native=/usr/local/lib");
    println!("cargo:rustc-link-search=native=/usr/lib");

    println!("cargo:rustc-link-lib=m");
    println!("cargo:rustc-link-lib=solidc");
    println!("cargo:rustc-link-arg=-pthread");
    println!("cargo:rustc-link-lib=BlocksRuntime");

    // 3. Generate bindings
    let bindings = Builder::default()
        .header("wrapper.h")
        .use_core()
        .rustified_enum("file_result_t")
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: false,
        })
        .clang_arg("-std=c23")
        .clang_arg("-fblocks")
        .clang_arg("-D_GNU_SOURCE")
        .clang_arg("-fparse-all-comments")
        .blocklist_file(".*fenv.*") // blocks anything from <fenv.h>, <bits/fenv.h>
        .blocklist_item("FE_.*") // FE_TONEAREST, FE_DOWNWARD, etc.
        .blocklist_item("FP_.*") // FP_INT_UPWARD, etc.
        .blocklist_item("fexcept_t")
        .blocklist_item("fenv_t")
        .blocklist_item("__fe_.*")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

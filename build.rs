use std::{env, fs, path::PathBuf};

fn main() {
    let target = env::var("TARGET").unwrap_or_default();
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    if target.contains("riscv64") {
        fs::copy("memory-d0.x", out_dir.join("memory.x")).expect("Failed to copy memory-d0.x");
        println!("cargo:rustc-link-search={}", out_dir.display());
    } else if target.contains("riscv32") {
        fs::copy("memory-m0.x", out_dir.join("memory.x")).expect("Failed to copy memory-m0.x");
        println!("cargo:rustc-link-search={}", out_dir.display());
    }

    println!("cargo:rerun-if-changed=memory-d0.x");
    println!("cargo:rerun-if-changed=memory-m0.x");
    println!("cargo:rerun-if-changed=build.rs");
}

//! Build script: make our custom `memory.x` visible to the linker.
//!
//! cortex-m-rt's link.x does `INCLUDE memory.x`, so memory.x must be on the
//! linker search path. We add the crate root to that path and re-run if it
//! changes. (We do NOT use embassy-stm32's "memory-x" feature — see Cargo.toml —
//! because we ship our own OTA-aware memory.x.)

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Copy memory.x into OUT_DIR and add OUT_DIR to the linker search path.
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::copy("memory.x", out.join("memory.x")).unwrap();
    println!("cargo:rustc-link-search={}", out.display());

    println!("cargo:rerun-if-changed=memory.x");
    println!("cargo:rerun-if-changed=build.rs");
}

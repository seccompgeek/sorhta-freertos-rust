use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // Tell Cargo to look for the linker script in the current directory
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    
    // Copy memory.x to the out directory
    fs::copy("memory.x", out_dir.join("memory.x")).unwrap();
    
    // Tell cargo to re-run if memory.x or link.ld changes
    println!("cargo:rerun-if-changed=memory.x");
    println!("cargo:rerun-if-changed=link.ld");
    
    // Tell the linker where to find the memory configuration
    println!("cargo:rustc-link-search={}", out_dir.display());
    
    // Detect target
    let target = env::var("TARGET").unwrap_or_else(|_| "aarch64-unknown-none-softfloat".to_string());
    println!("cargo:warning=Building for target: {}", target);
    
    // Print rustc version for debugging
    if let Ok(output) = Command::new("rustc").arg("--version").output() {
        if let Ok(version) = String::from_utf8(output.stdout) {
            println!("cargo:warning=Using rustc: {}", version.trim());
        }
    }
}
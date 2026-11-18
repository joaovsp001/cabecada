// The window library links imm32 (Windows text input). Some MinGW setups
// ship without its import library, so on windows-gnu we let the linker read
// the system's imm32.dll directly (GNU ld can link against a DLL).
fn main() {
    let target = std::env::var("TARGET").unwrap_or_default();
    if target.ends_with("windows-gnu") {
        let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
        let windir = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".into());
        let src = std::path::Path::new(&windir).join("System32").join("imm32.dll");
        if src.exists() && std::fs::copy(&src, out.join("imm32.dll")).is_ok() {
            println!("cargo:rustc-link-search=native={}", out.display());
        }
    }
    println!("cargo:rerun-if-changed=build.rs");
}

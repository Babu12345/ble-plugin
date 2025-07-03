fn main() {
    println!(". /Users/babuwanyeki/export-esp.sh");
    println!("cargo:rustc-link-arg-bins=-Tlinkall.x");
    println!("cargo::rerun-if-changed=build.rs");
}

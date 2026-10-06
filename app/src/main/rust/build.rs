fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os != "android" {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        println!("cargo:rustc-link-search=native={}/.stubs", manifest_dir);
    }
    println!("cargo:rustc-link-lib=android");
    println!("cargo:rustc-link-lib=log");
}

// Windows resource embedding for the app icon and version info.
#[cfg(windows)]
fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os != "windows" {
        return;
    }

    println!("cargo:rerun-if-changed=assets/icon.ico");
    println!("cargo:rerun-if-changed=build.rs");

    let mut res = winresource::WindowsResource::new();
    res.set_icon("assets/icon.ico");
    res.set("ProductName", "FontCraft");
    res.set("FileDescription", "FontCraft: a fast native font browser");
    res.set("LegalCopyright", "Copyright (c) 2026 mohseenjamall");
    res.set("OriginalFilename", "fontcraft.exe");

    if let Err(e) = res.compile() {
        panic!("Failed to embed Windows resources: {e}");
    }
}
// winresource is only a build-dependency on Windows hosts.
#[cfg(not(windows))]
fn main() {}

// Windows resource embedding for the app icon and version info.
use std::env;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
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
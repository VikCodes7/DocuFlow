//! Build script.
//!
//! On Windows, tauri-build embeds the app manifest only into the `.exe`
//! (`rustc-link-arg-bins`). The `cargo test` binary for this crate then loads
//! Common Controls v5, which lacks `TaskDialogIndirect`, and dies at startup
//! with STATUS_ENTRYPOINT_NOT_FOUND (0xc0000139). So we turn off Tauri's copy
//! and embed the same manifest into every linked artifact ourselves.

fn main() {
    let windows_msvc = std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc");

    let mut windows = tauri_build::WindowsAttributes::new();
    if windows_msvc {
        windows = tauri_build::WindowsAttributes::new_without_app_manifest();
        embed_windows_manifest();
    }

    tauri_build::try_build(tauri_build::Attributes::new().windows_attributes(windows))
        .expect("failed to run tauri-build");
}

fn embed_windows_manifest() {
    let manifest =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("windows-app-manifest.xml");
    println!("cargo:rerun-if-changed={}", manifest.display());
    println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
    println!("cargo:rustc-link-arg=/MANIFESTINPUT:{}", manifest.display());
}

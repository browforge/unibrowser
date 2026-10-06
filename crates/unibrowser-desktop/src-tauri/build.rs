use std::path::Path;

fn main() {
    stage_media_sidecar();
    // E2E release only: serve the same `dist/` bytes over loopback (not Vite). WebKitGTK under
    // xvfb + tauri-driver loads bundled `tauri://` unreliably; loopback matches frontendDist.
    if std::env::var("CARGO_FEATURE_E2E").is_ok() {
        std::env::set_var(
            "TAURI_CONFIG",
            r#"{"app":{"windows":[{"label":"main","title":"Unibrowser","width":1200,"height":800,"url":"http://127.0.0.1:18765/index.html"}]}}"#,
        );
    }
    tauri_build::build()
}

fn stage_media_sidecar() {
    let target = std::env::var("TARGET").unwrap_or_default();
    if target.is_empty() {
        return;
    }
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    if manifest_dir.is_empty() {
        return;
    }
    let binaries_dir = Path::new(&manifest_dir).join("binaries");
    let dest = binaries_dir.join(format!("unibrowser-media-sidecar-{}", target));
    if dest.is_file() {
        return;
    }
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".into());
    let workspace = Path::new(&manifest_dir).join("../../..");
    let sidecar_name = if target.contains("windows") {
        "unibrowser-media-sidecar.exe"
    } else {
        "unibrowser-media-sidecar"
    };
    let src = workspace.join("target").join(&profile).join(sidecar_name);
    if !src.is_file() {
        return;
    }
    std::fs::create_dir_all(&binaries_dir).ok();
    if std::fs::copy(&src, &dest).is_ok() {
        println!("cargo:rerun-if-changed={}", src.display());
    }
}

use std::path::Path;

fn main() {
    stage_media_sidecar();
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    let stamp_src = Path::new(&manifest_dir).join("../dist/.ub-frontend-stamp");
    if !stamp_src.is_file() {
        panic!(
            "missing {} — run `npm run build` in crates/unibrowser-desktop before `cargo build`",
            stamp_src.display()
        );
    }
    let out_dir = std::env::var("OUT_DIR").unwrap_or_default();
    let stamp_dst = Path::new(&out_dir).join("frontend-stamp.txt");
    if let Ok(bytes) = std::fs::read(&stamp_src) {
        std::fs::write(&stamp_dst, bytes).expect("write frontend-stamp.txt");
    }
    println!("cargo:rerun-if-changed={}", stamp_src.display());
    let index_html = Path::new(&manifest_dir).join("../dist/index.html");
    println!("cargo:rerun-if-changed={}", index_html.display());
    if std::env::var("CARGO_FEATURE_E2E").is_ok() {
        std::env::set_var(
            "TAURI_CONFIG",
            r#"{"app":{"windows":[{"label":"main","title":"Unibrowser","width":1200,"height":800,"visible":true,"url":"http://127.0.0.1:18765/index.html"}]}}"#,
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

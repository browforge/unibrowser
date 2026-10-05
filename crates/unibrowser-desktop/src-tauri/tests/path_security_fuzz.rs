//! Adversarial inputs for path guards (no panics, no escape from temp sandbox).

use std::fs;
use std::path::{Path, PathBuf};

use unibrowser_desktop_lib::path_security::{reject_traversal, resolve_open_path};

fn adversarial_strings() -> Vec<String> {
    let mut out = Vec::new();
    let seeds: &[&str] = &[
        "",
        ".",
        "..",
        "...",
        "..\\..\\etc\\passwd",
        "/etc/passwd",
        "foo/../../../bar",
        "foo\0bar",
        "%2e%2e%2f",
        "....//....//etc/passwd",
        "C:\\Windows\\System32",
        "/",
        "\\",
        " ",
        "\t",
        "valid-name.txt",
        "..\\valid",
        "foo/..",
        "foo/./bar",
    ];
    for s in seeds {
        out.push((*s).to_string());
    }
    out.push("a".repeat(512));
    for i in 0..64 {
        out.push(format!("seg{i}/../other"));
        out.push(format!("..{i}"));
    }
    out
}

#[test]
fn fuzz_reject_traversal_never_panics() {
    for raw in adversarial_strings() {
        let _ = reject_traversal(Path::new(&raw));
        let _ = reject_traversal(&PathBuf::from(raw));
    }
}

#[test]
fn fuzz_resolve_open_path_stays_in_sandbox() {
    let sandbox = std::env::temp_dir().join(format!("ub-path-fuzz-{}", std::process::id()));
    let _ = fs::remove_dir_all(&sandbox);
    fs::create_dir_all(&sandbox).expect("sandbox");
    let inside = sandbox.join("inside.txt");
    fs::write(&inside, b"x").expect("write");

    for raw in adversarial_strings() {
        if raw.starts_with('/') || raw.starts_with('\\') || raw.contains(':') {
            continue;
        }
        let result = resolve_open_path(&raw, &sandbox);
        if result.is_ok() {
            let got = result.unwrap();
            let canon_sandbox = sandbox.canonicalize().expect("canon sandbox");
            let canon_got = got.canonicalize().unwrap_or(got);
            assert!(
                canon_got.starts_with(&canon_sandbox),
                "escaped sandbox: raw={:?} got={}",
                raw,
                canon_got.display()
            );
        }
    }
    let _ = fs::remove_dir_all(&sandbox);
}

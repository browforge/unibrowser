use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use unibrowser_build_info::{version_line, GIT_HASH, VERSION};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("unibrowser {}", version_line());
        if let Some(h) = GIT_HASH {
            println!("git {}", h);
        }
        return;
    }
    if args.len() >= 2 && args[1] == "mod" {
        if let Err(e) = run_mod_subcommand(&args[2..]) {
            eprintln!("{}", e);
            std::process::exit(1);
        }
        return;
    }
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return;
    }
    eprintln!("unibrowser {} — desktop UI runs via unibrowser-desktop (Tauri).", VERSION);
    eprintln!("Try: unibrowser --version");
    eprintln!("     unibrowser mod new wasm <name> <output-dir>");
    std::process::exit(1);
}

fn print_help() {
    println!("unibrowser {}", VERSION);
    println!("Usage:");
    println!("  unibrowser --version");
    println!("  unibrowser mod new wasm <name> <output-directory>");
}

fn run_mod_subcommand(args: &[String]) -> Result<(), String> {
    if args.len() < 4 || args[0] != "new" || args[1] != "wasm" {
        return Err("Usage: unibrowser mod new wasm <name> <output-directory>".into());
    }
    let name = &args[2];
    let out = &args[3];
    scaffold_wasm_mod(name, Path::new(out))?;
    println!("Scaffolded WASM mod at {}", out);
    println!(
        "Build: cargo build --manifest-path {}/Cargo.toml --target wasm32-unknown-unknown",
        out
    );
    Ok(())
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."))
}

fn scaffold_wasm_mod(name: &str, out: &Path) -> Result<(), String> {
    let root = workspace_root();
    let template = root.join("examples/ub-mod-wasm");
    if !template.is_dir() {
        return Err(format!("template missing: {}", template.display()));
    }
    if out.exists() {
        let mut empty = true;
        if let Ok(mut rd) = fs::read_dir(out) {
            if rd.next().is_some() {
                empty = false;
            }
        }
        if !empty {
            return Err(format!("output directory not empty: {}", out.display()));
        }
    } else {
        fs::create_dir_all(out).map_err(|e| e.to_string())?;
    }
    let crate_name = name.replace('-', "_");
    let cmd_id = format!("mod.wasm.{}", crate_name);
    let files = [
        ("Cargo.toml", "Cargo.toml"),
        ("info.json", "info.json"),
        ("src/lib.rs", "src/lib.rs"),
    ];
    for (src_rel, dest_rel) in files {
        let raw = fs::read_to_string(template.join(src_rel)).map_err(|e| e.to_string())?;
        let text = raw
            .replace("ub-mod-wasm", name)
            .replace("ub_mod_wasm", &crate_name)
            .replace("mod.wasm.demo", &cmd_id)
            .replace("WASM demo mod", &format!("{} (WASM)", name))
            .replace("WASM demo command", &format!("{} command", name));
        let dest = out.join(dest_rel);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::write(&dest, text).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scaffold_wasm_mod_writes_manifest() {
        let out = std::env::temp_dir().join(format!("ub-mod-scaffold-{}", std::process::id()));
        let _ = fs::remove_dir_all(&out);
        scaffold_wasm_mod("demo-scaffold", &out).expect("scaffold");
        assert!(out.join("Cargo.toml").is_file());
        let lib = fs::read_to_string(out.join("src/lib.rs")).expect("lib");
        assert!(lib.contains("demo_scaffold"));
        let status = Command::new("cargo")
            .arg("build")
            .arg("--manifest-path")
            .arg(out.join("Cargo.toml"))
            .arg("--target")
            .arg("wasm32-unknown-unknown")
            .status();
        match status {
            Ok(s) if s.success() => {}
            _ => eprintln!("skip wasm build: wasm32 target unavailable"),
        }
        let _ = fs::remove_dir_all(&out);
    }
}

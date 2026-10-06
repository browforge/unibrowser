use std::sync::{Mutex, OnceLock};

fn launch_open_paths() -> &'static Mutex<Vec<String>> {
    static SLOT: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(Vec::new()))
}

fn demo_state_slot() -> &'static Mutex<Option<String>> {
    static SLOT: OnceLock<Mutex<Option<String>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

/// Parse argv (after program name). Supports `--open <path>`, `--help`, and positional file paths.
pub fn ingest_process_args() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut open_paths: Vec<String> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        if a == "--help" || a == "-h" {
            print_help();
            std::process::exit(0);
        }
        if a == "--open" {
            i += 1;
            if i < args.len() {
                open_paths.push(args[i].clone());
            }
            i += 1;
            continue;
        }
        if a.starts_with('-') {
            i += 1;
            continue;
        }
        open_paths.push(a.clone());
        i += 1;
    }
    if let Ok(mut slot) = launch_open_paths().lock() {
        *slot = open_paths;
    }
    if let Ok(mut demo) = demo_state_slot().lock() {
        *demo = std::env::var("UNIBROWSER_DEMO_STATE")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
    }
}

fn print_help() {
    eprintln!(
        "Unibrowser desktop\n\
\n\
Usage:\n\
  unibrowser-desktop [--open <path>] [<path> ...]\n\
\n\
  --open <path>   Open a file or folder in a new tab (repeatable).\n\
  <path>          Same as --open (for shell \"Open with\" and drag-free launch).\n\
\n\
Hidden (docs / screenshots):\n\
  UNIBROWSER_DEMO_STATE   Comma-separated UI preset (problems,ai,live,vsix,ntp,palette,settings,mods).\n"
    );
}

#[tauri::command]
pub fn take_launch_open_paths() -> Vec<String> {
    launch_open_paths()
        .lock()
        .map(|mut p| {
            let out = p.clone();
            p.clear();
            out
        })
        .unwrap_or_default()
}

#[tauri::command]
pub fn get_demo_state() -> Option<String> {
    demo_state_slot().lock().ok().and_then(|d| d.clone())
}

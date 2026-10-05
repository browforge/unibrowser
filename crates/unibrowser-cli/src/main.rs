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
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("unibrowser {}", VERSION);
        println!("Usage: unibrowser --version");
        return;
    }
    eprintln!("unibrowser {} — desktop UI runs via unibrowser-desktop (Tauri).", VERSION);
    eprintln!("Try: unibrowser --version");
    std::process::exit(1);
}

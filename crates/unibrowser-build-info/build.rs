fn main() {
    if let Ok(out) = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
    {
        if out.status.success() {
            let hash = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !hash.is_empty() {
                println!("cargo:rustc-env=UB_GIT_HASH={}", hash);
            }
        }
    }
}

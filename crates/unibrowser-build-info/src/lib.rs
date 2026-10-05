//! Version strings embedded at compile time.

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Short git revision when `build.rs` could run `git rev-parse`.
pub const GIT_HASH: Option<&'static str> = option_env!("UB_GIT_HASH");

pub fn version_line() -> String {
    match GIT_HASH {
        Some(h) => format!("{} ({})", VERSION, h),
        None => VERSION.to_string(),
    }
}

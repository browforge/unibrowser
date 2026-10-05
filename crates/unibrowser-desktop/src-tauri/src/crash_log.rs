//! Local panic logs under the config dir. Never uploaded.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::OnceLock;

use crate::config::config_dir;

static INSTALLED: OnceLock<()> = OnceLock::new();

pub fn crash_logs_dir() -> Option<PathBuf> {
    config_dir().map(|d| d.join("crash-logs"))
}

pub fn ensure_crash_logs_dir() -> Option<PathBuf> {
    let dir = crash_logs_dir()?;
    let _ = fs::create_dir_all(&dir);
    Some(dir)
}

fn write_panic_record(info: &std::panic::PanicHookInfo) -> Option<PathBuf> {
    let dir = ensure_crash_logs_dir()?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let path = dir.join(format!("panic-{}.log", now));
    let payload = info.payload();
    let message = if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "non-string panic payload".into()
    };
    let location = info
        .location()
        .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
        .unwrap_or_else(|| "unknown".into());
    let backtrace = std::backtrace::Backtrace::force_capture();
    let body = format!(
        "Unibrowser panic log (local only, not uploaded)\n\
timestamp_unix={}\n\
location={}\n\
message={}\n\
\nBacktrace:\n{}\n",
        now,
        location,
        message,
        backtrace
    );
    if let Ok(mut f) = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&path)
    {
        let _ = f.write_all(body.as_bytes());
    }
    Some(path)
}

pub fn install_panic_hook() {
    if INSTALLED.set(()).is_err() {
        return;
    }
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = write_panic_record(info);
        default(info);
    }));
}

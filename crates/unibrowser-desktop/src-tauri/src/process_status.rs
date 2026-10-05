//! Cheap process stats for diagnostics (Linux RSS when available).

#[derive(Debug, Clone, serde::Serialize)]
pub struct ProcessMemoryDto {
    pub rss_kb: Option<u64>,
    pub detail: String,
}

pub fn process_memory() -> ProcessMemoryDto {
    #[cfg(target_os = "linux")]
    {
        if let Some(kb) = linux_vm_rss_kb() {
            let mb = kb as f64 / 1024.0;
            return ProcessMemoryDto {
                rss_kb: Some(kb),
                detail: format!("RSS ~{:.1} MB (from /proc/self/status)", mb),
            };
        }
        return ProcessMemoryDto {
            rss_kb: None,
            detail: "Could not read VmRSS on this Linux host".into(),
        };
    }
    #[cfg(not(target_os = "linux"))]
    {
        ProcessMemoryDto {
            rss_kb: None,
            detail: "RSS not collected on this platform".into(),
        }
    }
}

#[cfg(target_os = "linux")]
fn linux_vm_rss_kb() -> Option<u64> {
    let text = std::fs::read_to_string("/proc/self/status").ok()?;
    for line in text.lines() {
        if line.starts_with("VmRSS:") {
            return line
                .split_whitespace()
                .nth(1)
                .and_then(|v| v.parse().ok());
        }
    }
    None
}

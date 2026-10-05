//! Ring buffer of recent shell errors for `unibrowser://diagnostics`.

use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, serde::Serialize)]
pub struct DiagnosticLogEntry {
    pub at_ms: u64,
    pub message: String,
    pub details: Option<String>,
}

const MAX_ENTRIES: usize = 24;

static LOG: Mutex<VecDeque<DiagnosticLogEntry>> = Mutex::new(VecDeque::new());

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn record_info(message: impl Into<String>, details: Option<String>) {
    record_error(message, details);
}

pub fn record_error(message: impl Into<String>, details: Option<String>) {
    let msg = message.into();
    let trimmed = msg.trim();
    if trimmed.is_empty() {
        return;
    }
    let entry = DiagnosticLogEntry {
        at_ms: now_ms(),
        message: trimmed.chars().take(400).collect(),
        details: details.and_then(|d| {
            let s: String = d.chars().take(2000).collect();
            if s.trim().is_empty() {
                None
            } else {
                Some(s)
            }
        }),
    };
    if let Ok(mut log) = LOG.lock() {
        log.push_back(entry);
        while log.len() > MAX_ENTRIES {
            log.pop_front();
        }
    }
}

pub fn recent_entries() -> Vec<DiagnosticLogEntry> {
    LOG.lock()
        .map(|log| log.iter().cloned().collect())
        .unwrap_or_default()
}

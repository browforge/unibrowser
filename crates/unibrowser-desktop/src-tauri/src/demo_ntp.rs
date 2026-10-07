//! Representative New-tab Recent and Downloads rows for docs captures and first-run polish.

use crate::download_history::{DownloadHistoryEntry, DownloadHistoryStatus};
use crate::preferences::{load_preferences, push_recent_file, save_preferences};

fn demo_root() -> String {
    std::env::var("UNIBROWSER_REPO_ROOT")
        .map(|r| format!("{}/examples/demo-pack", r.trim_end_matches('/')))
        .unwrap_or_else(|_| "/workspace/examples/demo-pack".into())
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn seed_demo_ntp_fixture() -> Result<(), String> {
    let root = demo_root();
    let now = now_ms();
    let mut prefs = load_preferences();
    let recent = [
        (format!("{}/sample.ts", root), "sample.ts", now - 12 * 60_000),
        (format!("{}/sample.pdf", root), "sample.pdf", now - 3 * 3600_000),
        (format!("{}/sample-rich.docx", root), "sample-rich.docx", now - 26 * 3600_000),
        (format!("{}/sample.svg", root), "sample.svg", now - 3 * 24 * 3600_000),
    ];
    for (path, label, opened_at_ms) in recent {
        let path = path;
        push_recent_file(&mut prefs, path.clone(), label.to_string());
        if let Some(entry) = prefs.recent_files.iter_mut().find(|e| e.path == path) {
            entry.opened_at_ms = opened_at_ms;
        }
    }
    save_preferences(&prefs)?;

    let in_progress = DownloadHistoryEntry {
        id: "demo-dl-progress".into(),
        url: "https://releases.example/unibrowser-demo-pack.zip".into(),
        label: "unibrowser-demo-pack.zip".into(),
        status: DownloadHistoryStatus::InProgress,
        error: None,
        destination: None,
        progress_percent: Some(64),
        at_ms: now - 4 * 60_000,
    };
    let completed = DownloadHistoryEntry {
        id: "demo-dl-complete".into(),
        url: "https://cdn.example/field-notes.md".into(),
        label: "field-notes.md".into(),
        status: DownloadHistoryStatus::Saved,
        error: None,
        destination: Some(format!("{}/sample.md", root)),
        progress_percent: None,
        at_ms: now - 2 * 3600_000,
    };
    crate::download_history::seed_demo_entries_if_empty(vec![in_progress, completed])?;
    Ok(())
}

#[tauri::command]
pub fn seed_demo_ntp_fixture_cmd() -> Result<(), String> {
    seed_demo_ntp_fixture()
}

//! Directory listing performance smoke (scale with UB_LIST_STRESS=1 for 10k).

use std::fs;
use std::time::Instant;

use base_files::list_directory;

fn entry_count() -> usize {
    if std::env::var("UB_LIST_STRESS").as_deref() == Ok("1") {
        10_000
    } else {
        1_500
    }
}

#[test]
fn list_large_directory_under_time_budget() {
    let n = entry_count();
    let dir = std::env::temp_dir().join(format!("ub-list-perf-{}-{}", n, std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("mkdir");
    for i in 0..n {
        fs::write(dir.join(format!("file-{i:05}.txt")), b"x").expect("write");
    }

    let start = Instant::now();
    let entries = list_directory(&dir).expect("list");
    let elapsed = start.elapsed();
    assert_eq!(entries.len(), n);
    let budget_ms = if n >= 10_000 { 8_000 } else { 3_000 };
    assert!(
        elapsed.as_millis() < budget_ms,
        "list_directory({} entries) took {}ms (budget {}ms)",
        n,
        elapsed.as_millis(),
        budget_ms
    );

    let _ = fs::remove_dir_all(&dir);
}

//! Per-request cancel tokens for slow directory listings.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use base_files::DirEntry;

pub struct ListingControl {
    next_id: AtomicU64,
    tokens: Mutex<HashMap<u64, Arc<AtomicBool>>>,
}

impl ListingControl {
    pub fn new() -> Self {
        Self {
            next_id: AtomicU64::new(1),
            tokens: Mutex::new(HashMap::new()),
        }
    }

    pub fn register(&self) -> (u64, Arc<AtomicBool>) {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let flag = Arc::new(AtomicBool::new(false));
        self.tokens
            .lock()
            .expect("listing tokens")
            .insert(id, flag.clone());
        (id, flag)
    }

    pub fn cancel(&self, id: u64) {
        if let Some(flag) = self.tokens.lock().expect("listing tokens").get(&id) {
            flag.store(true, Ordering::SeqCst);
        }
    }

    pub fn token(&self, id: u64) -> Option<Arc<AtomicBool>> {
        self.tokens
            .lock()
            .expect("listing tokens")
            .get(&id)
            .cloned()
    }

    pub fn finish(&self, id: u64) {
        self.tokens.lock().expect("listing tokens").remove(&id);
    }
}

pub fn list_dir_with_timeout(
    path: &std::path::Path,
    cancel: Arc<AtomicBool>,
    timeout: Duration,
) -> Result<Vec<DirEntry>, String> {
    let path = path.to_path_buf();
    let worker_cancel = cancel.clone();
    let handle =
        std::thread::spawn(move || crate::dir_list::list_dir_cancellable(&path, &worker_cancel));
    let started = std::time::Instant::now();
    while started.elapsed() < timeout {
        if handle.is_finished() {
            return handle
                .join()
                .map_err(|_| "listing thread panicked".to_string())?;
        }
        if cancel.load(Ordering::Relaxed) {
            cancel.store(true, Ordering::SeqCst);
        }
        std::thread::sleep(Duration::from_millis(40));
    }
    cancel.store(true, Ordering::SeqCst);
    Err("directory listing timed out".into())
}

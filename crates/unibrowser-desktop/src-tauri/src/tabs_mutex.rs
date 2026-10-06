//! Per-window tab manager mutex with debug re-entrancy detection. See `docs/concurrency.md`.

use std::ops::{Deref, DerefMut};
use std::sync::{Mutex, MutexGuard, PoisonError};

use unibrowser_engine::TabManager;

#[cfg(debug_assertions)]
mod track {
    use std::sync::atomic::{AtomicUsize, Ordering};

    static DEPTH: AtomicUsize = AtomicUsize::new(0);

    pub fn acquire() {
        let prev = DEPTH.fetch_add(1, Ordering::SeqCst);
        if prev != 0 {
            panic!(
                "re-entrant `WindowSession::tabs` lock on this thread (see docs/concurrency.md)"
            );
        }
    }

    pub fn release() {
        let prev = DEPTH.fetch_sub(1, Ordering::SeqCst);
        if prev == 0 {
            panic!("tabs lock release without acquire");
        }
    }

    pub fn assert_free() {
        if DEPTH.load(Ordering::SeqCst) != 0 {
            panic!(
                "`snapshot` or other nested command called while holding `tabs` lock (see docs/concurrency.md)"
            );
        }
    }
}

#[cfg(not(debug_assertions))]
mod track {
    pub fn acquire() {}
    pub fn release() {}
    pub fn assert_free() {}
}

pub fn assert_tabs_lock_free() {
    track::assert_free();
}

pub struct TabsMutex(Mutex<TabManager>);

pub struct TrackedTabsGuard<'a>(MutexGuard<'a, TabManager>);

impl Deref for TrackedTabsGuard<'_> {
    type Target = TabManager;
    fn deref(&self) -> &TabManager {
        &*self.0
    }
}

impl DerefMut for TrackedTabsGuard<'_> {
    fn deref_mut(&mut self) -> &mut TabManager {
        &mut *self.0
    }
}

impl Drop for TrackedTabsGuard<'_> {
    fn drop(&mut self) {
        track::release();
    }
}

impl TabsMutex {
    pub fn new(inner: TabManager) -> Self {
        Self(Mutex::new(inner))
    }

    pub fn lock(&self) -> Result<TrackedTabsGuard<'_>, PoisonError<MutexGuard<'_, TabManager>>> {
        track::acquire();
        match self.0.lock() {
            Ok(guard) => Ok(TrackedTabsGuard(guard)),
            Err(e) => {
                track::release();
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use unibrowser_mod_api::BrowserLocation;

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "re-entrant")]
    fn double_tabs_lock_panics_in_debug() {
        let home = BrowserLocation::File {
            path: std::path::PathBuf::from("/tmp"),
        };
        let mux = TabsMutex::new(TabManager::new_with_home(home));
        let _a = mux.lock().expect("lock");
        let _b = mux.lock().expect("lock");
    }
}

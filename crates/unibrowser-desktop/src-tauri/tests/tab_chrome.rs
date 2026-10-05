//! Tab pin/reorder behaviour without UI.

use std::path::PathBuf;

use unibrowser_engine::TabManager;
use unibrowser_mod_api::BrowserLocation;

#[test]
fn reorder_tabs_updates_order() {
    let home = BrowserLocation::File {
        path: PathBuf::from("/tmp"),
    };
    let mut mgr = TabManager::new_with_home(home);
    mgr.open_tab(BrowserLocation::File {
        path: PathBuf::from("/a"),
    });
    mgr.open_tab(BrowserLocation::File {
        path: PathBuf::from("/b"),
    });
    assert_eq!(mgr.tab_count(), 3);
    assert!(mgr.move_tab(0, 2));
    let labels: Vec<String> = mgr
        .tabs()
        .iter()
        .map(|t| match &t.location {
            BrowserLocation::File { path } => path.to_string_lossy().to_string(),
            _ => "?".into(),
        })
        .collect();
    assert_eq!(labels[2], "/tmp");
}

#[test]
fn pin_toggle_on_tab() {
    let home = BrowserLocation::File {
        path: PathBuf::from("/tmp"),
    };
    let mut mgr = TabManager::new_with_home(home);
    assert!(!mgr.is_tab_pinned(0));
    assert!(mgr.set_tab_pinned(0, true));
    assert!(mgr.is_tab_pinned(0));
}

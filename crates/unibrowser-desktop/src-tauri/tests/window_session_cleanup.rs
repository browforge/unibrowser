//! Closing many document tabs must not retain undo stacks for every path.

use std::fs;

use unibrowser_mod_api::BrowserLocation;

use unibrowser_desktop_lib::window_session::WindowSession;

#[test]
fn closing_many_tabs_purges_undo_stacks() {
    let dir = std::env::temp_dir().join(format!("ub-session-clean-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("dir");

    let session = std::sync::Arc::new(WindowSession::new_with_home(&dir));

    for i in 0..80 {
        let path = dir.join(format!("doc-{i}.txt"));
        fs::write(&path, format!("body {i}")).expect("write");
        let path_key = path.to_string_lossy().to_string();
        let doc = unibrowser_desktop_lib::open_file_document(&path);
        {
            let mut tabs = session.tabs.lock().expect("tabs");
            tabs.open_tab(BrowserLocation::File { path });
            if let Some(tab) = tabs.active_tab_mut() {
                tab.document = doc;
                tab.is_directory = false;
            }
        }
        {
            let mut stacks = session.undo_stacks.lock().expect("undo");
            stacks.record_change(&path_key, "a".into(), "b");
        }
    }

    {
        let stacks = session.undo_stacks.lock().expect("undo");
        assert!(stacks.tracked_path_count() > 10);
    }

    while {
        let tabs = session.tabs.lock().expect("tabs");
        tabs.tab_count() > 1
    } {
        let idx = {
            let tabs = session.tabs.lock().expect("tabs");
            tabs.tab_count() - 1
        };
        {
            let tabs = session.tabs.lock().expect("tabs");
            if let Some(tab) = tabs.tabs().get(idx) {
                session.purge_tab_resources(tab);
            }
        }
        let mut tabs = session.tabs.lock().expect("tabs");
        assert!(tabs.close_tab(idx));
    }

    {
        let stacks = session.undo_stacks.lock().expect("undo");
        assert_eq!(
            stacks.tracked_path_count(),
            0,
            "undo stacks should be empty after closing document tabs"
        );
    }

    let _ = fs::remove_dir_all(&dir);
}

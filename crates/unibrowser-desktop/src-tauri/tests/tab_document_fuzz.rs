//! Open/close cycles for common document backends (no Tauri runtime).

use std::fs;
use std::path::PathBuf;

use unibrowser_engine::TabManager;
use unibrowser_mod_api::BrowserLocation;

fn write_fixture(dir: &PathBuf, name: &str, bytes: &[u8]) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, bytes).expect("write fixture");
    path
}

#[test]
fn fuzz_open_close_document_tabs() {
    let dir = std::env::temp_dir().join("unibrowser-tab-fuzz");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("mkdir");

    let txt = write_fixture(&dir, "a.txt", b"hello");
    let md = write_fixture(&dir, "b.md", b"# Title\n\nBody");
    let svg = write_fixture(
        &dir,
        "c.svg",
        br#"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><rect width="10" height="10"/></svg>"#,
    );
    let epub = write_fixture(&dir, "d.epub", b"PK\x03\x04stub");
    let pdf = write_fixture(&dir, "e.pdf", b"%PDF-1.4\n1 0 obj<<>>endobj\ntrailer<<>>\n%%EOF\n");

    let paths = [txt, md, svg, epub, pdf];
    let home = BrowserLocation::File {
        path: dir.clone(),
    };
    let mut tabs = TabManager::new_with_home(home);

    for round in 0..6 {
        for path in &paths {
            let doc = unibrowser_desktop_lib::open_file_document(path);
            tabs.open_tab(BrowserLocation::File { path: path.clone() });
            if let Some(tab) = tabs.active_tab_mut() {
                tab.document = doc;
                tab.is_directory = false;
            }
            assert!(tabs.active_tab().unwrap().document.is_some());
            if tabs.tab_count() > 1 {
                tabs.close_tab(tabs.active_index());
            }
        }
        let _ = round;
    }

    let _ = fs::remove_dir_all(&dir);
}

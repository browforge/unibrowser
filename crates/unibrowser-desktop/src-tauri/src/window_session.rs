//! Per-window tab set and related UI session state (split, undo, compare).

use std::path::Path;
use std::sync::{Arc, Mutex};

use unibrowser_engine::TabManager;
use unibrowser_mod_api::BrowserLocation;

use crate::state::{FileCompareState, SplitViewState};
use crate::undo_stack::UndoStacks;

pub struct WindowSession {
    pub tabs: Mutex<TabManager>,
    pub split_view: Mutex<SplitViewState>,
    pub undo_stacks: Mutex<UndoStacks>,
    pub file_compare: Mutex<Option<FileCompareState>>,
    pub chrome_notice: Mutex<Option<String>>,
}

impl WindowSession {
    pub fn new_with_home(cwd: &Path) -> Self {
        let home = BrowserLocation::File {
            path: cwd.to_path_buf(),
        };
        let mut tab_mgr = TabManager::new_with_home(home);
        if let Some(tab) = tab_mgr.active_tab_mut() {
            tab.is_directory = cwd.is_dir();
        }
        Self {
            tabs: Mutex::new(tab_mgr),
            split_view: Mutex::new(SplitViewState::default()),
            undo_stacks: Mutex::new(UndoStacks::default()),
            file_compare: Mutex::new(None),
            chrome_notice: Mutex::new(None),
        }
    }
}

pub type SharedWindowSession = Arc<WindowSession>;

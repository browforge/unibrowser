//! Per-window tab set and related UI session state (split, undo, compare).

use std::path::Path;
use std::sync::{Arc, Mutex};

use unibrowser_engine::TabManager;
use unibrowser_mod_api::BrowserLocation;

use crate::close_undo::CloseUndoStack;
use crate::crash_launch::RecoveryBarState;
use crate::closed_tabs::ClosedTabStack;
use crate::state::{FileCompareState, ImageCompareState, SplitViewState};
use crate::undo_stack::UndoStacks;

pub struct WindowSession {
    pub tabs: Mutex<TabManager>,
    pub closed_tabs: Mutex<ClosedTabStack>,
    pub close_undo: Mutex<CloseUndoStack>,
    pub recovery_bar: Mutex<RecoveryBarState>,
    pub split_view: Mutex<SplitViewState>,
    pub undo_stacks: Mutex<UndoStacks>,
    pub file_compare: Mutex<Option<FileCompareState>>,
    pub image_compare: Mutex<Option<ImageCompareState>>,
    pub chrome_notice: Mutex<Option<String>>,
}

impl WindowSession {
    /// Drop per-tab undo stacks when a document tab closes (avoids unbounded retention).
    pub fn purge_tab_resources(&self, tab: &unibrowser_engine::tabs::Tab) {
        if let Some(doc) = &tab.document {
            if let Ok(mut stacks) = self.undo_stacks.lock() {
                stacks.remove_path(&doc.path);
            }
        }
    }

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
            closed_tabs: Mutex::new(ClosedTabStack::default()),
            close_undo: Mutex::new(CloseUndoStack::default()),
            recovery_bar: Mutex::new(RecoveryBarState::default()),
            split_view: Mutex::new(SplitViewState::default()),
            undo_stacks: Mutex::new(UndoStacks::default()),
            file_compare: Mutex::new(None),
            image_compare: Mutex::new(None),
            chrome_notice: Mutex::new(None),
        }
    }
}

pub type SharedWindowSession = Arc<WindowSession>;

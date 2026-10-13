//! Per-window tab set and related UI session state (split, undo, compare).

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

use unibrowser_document_model::HistoryLogEntry;

use unibrowser_engine::TabManager;

use crate::tabs_mutex::TabsMutex;
use unibrowser_mod_api::BrowserLocation;

use crate::close_undo::CloseUndoStack;
use crate::crash_launch::RecoveryBarState;
use crate::closed_tabs::ClosedTabStack;
use crate::state::{FileCompareState, ImageCompareState, SplitViewState};
use crate::undo_stack::UndoStacks;

pub struct WindowSession {
    pub tabs: TabsMutex,
    pub closed_tabs: Mutex<ClosedTabStack>,
    pub close_undo: Mutex<CloseUndoStack>,
    pub recovery_bar: Mutex<RecoveryBarState>,
    pub split_view: Mutex<SplitViewState>,
    pub undo_stacks: Mutex<UndoStacks>,
    pub file_compare: Mutex<Option<FileCompareState>>,
    pub image_compare: Mutex<Option<ImageCompareState>>,
    pub chrome_notice: Mutex<Option<String>>,
    pub document_history_redo: Mutex<HashMap<String, Vec<HistoryLogEntry>>>,
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

    pub fn new_with_home(_cwd: &Path) -> Self {
        let home = BrowserLocation::Internal {
            page: "home".into(),
        };
        let tab_mgr = TabManager::new_with_home(home);
        Self {
            tabs: TabsMutex::new(tab_mgr),
            closed_tabs: Mutex::new(ClosedTabStack::default()),
            close_undo: Mutex::new(CloseUndoStack::default()),
            recovery_bar: Mutex::new(RecoveryBarState::default()),
            split_view: Mutex::new(SplitViewState::default()),
            undo_stacks: Mutex::new(UndoStacks::default()),
            file_compare: Mutex::new(None),
            image_compare: Mutex::new(None),
            chrome_notice: Mutex::new(None),
            document_history_redo: Mutex::new(HashMap::new()),
        }
    }
}

pub type SharedWindowSession = Arc<WindowSession>;

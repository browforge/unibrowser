use unibrowser_mod_api::{BrowserLocation, TabId};

use crate::document::DocumentSession;

pub struct Tab {
    pub id: TabId,
    pub location: BrowserLocation,
    pub history: Vec<BrowserLocation>,
    pub history_index: usize,
    pub document: Option<DocumentSession>,
    /// Directory listing tab (no document backend).
    pub is_directory: bool,
}

#[derive(Default)]
pub struct TabManager {
    tabs: Vec<Tab>,
    active: usize,
    next_id: u64,
}

impl TabManager {
    pub fn new_with_home(home: BrowserLocation) -> Self {
        let mut mgr = Self {
            tabs: Vec::new(),
            active: 0,
            next_id: 1,
        };
        mgr.open_tab(home);
        mgr
    }

    pub fn tabs(&self) -> &[Tab] {
        &self.tabs
    }

    pub fn active_index(&self) -> usize {
        self.active
    }

    pub fn active_tab(&self) -> Option<&Tab> {
        self.tabs.get(self.active)
    }

    pub fn active_tab_mut(&mut self) -> Option<&mut Tab> {
        self.tabs.get_mut(self.active)
    }

    pub fn tab_by_id(&mut self, id: TabId) -> Option<&mut Tab> {
        self.tabs.iter_mut().find(|t| t.id == id)
    }

    pub fn open_tab(&mut self, location: BrowserLocation) -> TabId {
        let id = TabId(self.next_id);
        self.next_id += 1;
        let tab = Tab {
            id,
            location: location.clone(),
            history: vec![location],
            history_index: 0,
            document: None,
            is_directory: false,
        };
        self.tabs.push(tab);
        self.active = self.tabs.len() - 1;
        id
    }

    pub fn set_active(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.active = index;
        }
    }

    pub fn navigate_active(
        &mut self,
        location: BrowserLocation,
        is_directory: bool,
        document: Option<DocumentSession>,
    ) {
        if let Some(tab) = self.active_tab_mut() {
            if tab.history_index + 1 < tab.history.len() {
                tab.history.truncate(tab.history_index + 1);
            }
            tab.history.push(location.clone());
            tab.history_index = tab.history.len() - 1;
            tab.location = location;
            tab.is_directory = is_directory;
            tab.document = document;
        }
    }

    pub fn can_go_back(&self) -> bool {
        self.active_tab()
            .map(|t| t.history_index > 0)
            .unwrap_or(false)
    }

    pub fn can_go_forward(&self) -> bool {
        self.active_tab()
            .map(|t| t.history_index + 1 < t.history.len())
            .unwrap_or(false)
    }

    pub fn go_back(&mut self) -> Option<BrowserLocation> {
        let tab = self.active_tab_mut()?;
        if tab.history_index == 0 {
            return None;
        }
        tab.history_index -= 1;
        tab.location = tab.history[tab.history_index].clone();
        Some(tab.location.clone())
    }

    pub fn go_forward(&mut self) -> Option<BrowserLocation> {
        let tab = self.active_tab_mut()?;
        if tab.history_index + 1 >= tab.history.len() {
            return None;
        }
        tab.history_index += 1;
        tab.location = tab.history[tab.history_index].clone();
        Some(tab.location.clone())
    }

    pub fn reload_location(&self) -> Option<BrowserLocation> {
        self.active_tab().map(|t| t.location.clone())
    }
}

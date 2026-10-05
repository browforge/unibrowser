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
    /// Pinned tabs stay identifiable in the strip; middle-click close is ignored until unpinned.
    pub pinned: bool,
    /// Reveal this line in the text editor on the next snapshot (one-shot).
    pub reveal_line: Option<u32>,
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

    pub fn tab_mut(&mut self, index: usize) -> Option<&mut Tab> {
        self.tabs.get_mut(index)
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
            pinned: false,
            reveal_line: None,
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

    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    /// Close tab at `index`; keeps at least one tab open.
    /// Replace all tabs with `locations` (at least one). Documents are cleared; host must re-resolve.
    pub fn load_locations(&mut self, locations: Vec<BrowserLocation>, active: usize) {
        self.tabs.clear();
        self.next_id = 1;
        if locations.is_empty() {
            let home = BrowserLocation::File {
                path: std::path::PathBuf::from("/"),
            };
            self.open_tab(home);
            return;
        }
        for loc in locations {
            let id = TabId(self.next_id);
            self.next_id += 1;
            self.tabs.push(Tab {
                id,
                location: loc.clone(),
                history: vec![loc.clone()],
                history_index: 0,
                document: None,
                is_directory: false,
                pinned: false,
                reveal_line: None,
            });
        }
        self.active = active.min(self.tabs.len() - 1);
    }

    pub fn move_tab(&mut self, from: usize, to: usize) -> bool {
        if from >= self.tabs.len() || to >= self.tabs.len() || from == to {
            return false;
        }
        let active_id = self.tabs.get(self.active).map(|t| t.id);
        let item = self.tabs.remove(from);
        self.tabs.insert(to, item);
        if let Some(id) = active_id {
            self.active = self
                .tabs
                .iter()
                .position(|t| t.id == id)
                .unwrap_or(0);
        }
        true
    }

    pub fn set_tab_pinned(&mut self, index: usize, pinned: bool) -> bool {
        if let Some(tab) = self.tabs.get_mut(index) {
            tab.pinned = pinned;
            return true;
        }
        false
    }

    pub fn is_tab_pinned(&self, index: usize) -> bool {
        self.tabs.get(index).map(|t| t.pinned).unwrap_or(false)
    }

    pub fn close_tab(&mut self, index: usize) -> bool {
        if self.tabs.len() <= 1 || index >= self.tabs.len() {
            return false;
        }
        self.tabs.remove(index);
        if self.active >= self.tabs.len() {
            self.active = self.tabs.len() - 1;
        } else if index < self.active {
            self.active -= 1;
        }
        true
    }

    /// Copy location/history into a new active tab (`document` supplied by the shell).
    pub fn duplicate_tab(
        &mut self,
        index: usize,
        document: Option<DocumentSession>,
    ) -> Option<usize> {
        let src = self.tabs.get(index)?;
        let id = TabId(self.next_id);
        self.next_id += 1;
        let tab = Tab {
            id,
            location: src.location.clone(),
            history: src.history.clone(),
            history_index: src.history_index,
            document,
            is_directory: src.is_directory,
            pinned: false,
            reveal_line: None,
        };
        self.tabs.push(tab);
        self.active = self.tabs.len() - 1;
        Some(self.active)
    }

    /// Indices removed (high to low), excluding `keep_index`. Never removes the last tab.
    pub fn close_other_tabs(&mut self, keep_index: usize) -> Vec<usize> {
        if keep_index >= self.tabs.len() {
            return vec![];
        }
        let keep_id = self.tabs[keep_index].id;
        let mut removed = Vec::new();
        for i in (0..self.tabs.len()).rev() {
            if i != keep_index && self.tabs.len() > 1 {
                removed.push(i);
                self.tabs.remove(i);
            }
        }
        self.active = self
            .tabs
            .iter()
            .position(|t| t.id == keep_id)
            .unwrap_or(0);
        removed
    }

    pub fn close_tabs_to_right(&mut self, index: usize) -> Vec<usize> {
        if index >= self.tabs.len() {
            return vec![];
        }
        let keep_id = self.tabs[index].id;
        let mut removed = Vec::new();
        for i in (index + 1..self.tabs.len()).rev() {
            if self.tabs.len() > 1 {
                removed.push(i);
                self.tabs.remove(i);
            }
        }
        self.active = self
            .tabs
            .iter()
            .position(|t| t.id == keep_id)
            .unwrap_or(0);
        removed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use unibrowser_mod_api::BrowserLocation;

    #[test]
    fn load_locations_restores_active() {
        let home = BrowserLocation::File {
            path: PathBuf::from("/tmp"),
        };
        let mut mgr = TabManager::new_with_home(home);
        mgr.load_locations(
            vec![
                BrowserLocation::File {
                    path: PathBuf::from("/a"),
                },
                BrowserLocation::File {
                    path: PathBuf::from("/b"),
                },
            ],
            1,
        );
        assert_eq!(mgr.tab_count(), 2);
        assert_eq!(mgr.active_index(), 1);
    }

    #[test]
    fn duplicate_tab_copies_location() {
        let home = BrowserLocation::File {
            path: PathBuf::from("/tmp"),
        };
        let mut mgr = TabManager::new_with_home(home);
        mgr.open_tab(BrowserLocation::File {
            path: PathBuf::from("/tmp/a.txt"),
        });
        let idx = mgr.active_index();
        mgr.duplicate_tab(idx, None);
        assert_eq!(mgr.tab_count(), 3);
        assert_eq!(mgr.active_index(), 2);
        assert_eq!(
            mgr.active_tab().map(|t| t.location.clone()),
            Some(BrowserLocation::File {
                path: PathBuf::from("/tmp/a.txt"),
            })
        );
    }

    #[test]
    fn stress_open_twenty_tabs() {
        let home = BrowserLocation::File {
            path: PathBuf::from("/tmp"),
        };
        let mut mgr = TabManager::new_with_home(home);
        assert_eq!(mgr.tab_count(), 1);
        for i in 0..19 {
            mgr.open_tab(BrowserLocation::File {
                path: PathBuf::from(format!("/tmp/stress-tab-{}", i)),
            });
        }
        assert_eq!(mgr.tab_count(), 20);
        assert!(mgr.active_tab().is_some());
    }

    #[test]
    fn move_tab_preserves_active() {
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
        mgr.set_active(2);
        assert!(mgr.move_tab(0, 2));
        assert_eq!(mgr.active_index(), 1);
    }

    #[test]
    fn open_close_tabs_never_empty() {
        let home = BrowserLocation::File {
            path: PathBuf::from("/tmp"),
        };
        let mut mgr = TabManager::new_with_home(home);
        assert_eq!(mgr.tab_count(), 1);
        for i in 0..8 {
            mgr.open_tab(BrowserLocation::File {
                path: PathBuf::from(format!("/tmp/f{}", i)),
            });
        }
        assert_eq!(mgr.tab_count(), 9);
        while mgr.tab_count() > 1 {
            mgr.close_tab(mgr.active_index());
        }
        assert_eq!(mgr.tab_count(), 1);
        assert!(mgr.active_tab().is_some());
    }
}

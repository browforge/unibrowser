use serde::{Deserialize, Serialize};
use unibrowser_mod_api::commands;

/// User-persistable toolbar layout (ADR 0005).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolbarTab {
    pub id: String,
    pub label: String,
    pub commands: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolbarConfig {
    pub toolbar_tabs: Vec<ToolbarTab>,
    pub selected_tab: String,
}

impl Default for ToolbarConfig {
    fn default() -> Self {
        Self {
            toolbar_tabs: vec![
                ToolbarTab {
                    id: "text".into(),
                    label: "Text".into(),
                    commands: vec![
                        commands::FONT_SIZE_INCREASE.into(),
                        commands::FONT_SIZE_DECREASE.into(),
                    ],
                },
                ToolbarTab {
                    id: "image".into(),
                    label: "Image".into(),
                    commands: vec![
                        commands::FONT_SIZE_INCREASE.into(),
                        commands::FONT_SIZE_DECREASE.into(),
                    ],
                },
            ],
            selected_tab: "text".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ToolbarController {
    pub config: ToolbarConfig,
}

impl Default for ToolbarController {
    fn default() -> Self {
        Self {
            config: ToolbarConfig::default(),
        }
    }
}

impl ToolbarController {
    pub fn select_tab_for_mode(&mut self, mode: unibrowser_mod_api::Mode) {
        let tab = mode.tab_id();
        if self.config.toolbar_tabs.iter().any(|t| t.id == tab) {
            self.config.selected_tab = tab.to_string();
        }
    }

    pub fn active_tab(&self) -> Option<&ToolbarTab> {
        self.config
            .toolbar_tabs
            .iter()
            .find(|t| t.id == self.config.selected_tab)
    }
}

use serde::{Deserialize, Serialize};
use unibrowser_mod_api::{CommandContext, HostContext};

use crate::command::CommandRegistry;

/// Where the same logical command may appear (shared UI building blocks).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandSurface {
    CommandManagerTab,
    FloatingToolbar,
    CommandPalette,
    ContextMenu,
}

/// One button/slot in the UI backed by a registry command id.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UiCommandSlot {
    pub command_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolbarPhase {
    Active,
    Planned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagerTabLayout {
    pub id: String,
    pub label: String,
    pub phase: ToolbarPhase,
    pub commands: Vec<UiCommandSlot>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolbarAnchor {
    Docked,
    Floating,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloatingToolbarLayout {
    pub id: String,
    pub label: String,
    pub anchor: ToolbarAnchor,
    pub x: f64,
    pub y: f64,
    pub commands: Vec<UiCommandSlot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandManagerFile {
    pub selected_tab: String,
    pub tabs: Vec<ManagerTabLayout>,
    pub floating_bars: Vec<FloatingToolbarLayout>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedUiCommand {
    pub command_id: String,
    pub title: String,
    pub icon: String,
    pub enabled: bool,
    pub reason: Option<String>,
    pub surfaces: Vec<CommandSurface>,
}

#[derive(Debug, Clone)]
pub struct CommandManagerState {
    pub selected_tab: String,
    pub auto_context: CommandContext,
    pub tabs: Vec<ManagerTabLayout>,
    pub floating_bars: Vec<FloatingToolbarLayout>,
}

impl Default for CommandManagerFile {
    fn default() -> Self {
        Self::factory_default()
    }
}

impl CommandManagerFile {
    pub fn factory_default() -> Self {
        use unibrowser_mod_api::commands;
        Self {
            selected_tab: CommandContext::Text.tab_id().into(),
            tabs: vec![
                ManagerTabLayout {
                    id: CommandContext::Text.tab_id().into(),
                    label: CommandContext::Text.label().into(),
                    phase: ToolbarPhase::Active,
                    commands: vec![
                        slot(commands::TEXT_SAVE),
                        slot(commands::FONT_SIZE_INCREASE),
                        slot(commands::FONT_SIZE_DECREASE),
                    ],
                },
                ManagerTabLayout {
                    id: CommandContext::Office.tab_id().into(),
                    label: CommandContext::Office.label().into(),
                    phase: ToolbarPhase::Planned,
                    commands: vec![],
                },
                ManagerTabLayout {
                    id: CommandContext::Code.tab_id().into(),
                    label: CommandContext::Code.label().into(),
                    phase: ToolbarPhase::Planned,
                    commands: vec![
                        slot(commands::TEXT_SAVE),
                        slot(commands::FONT_SIZE_INCREASE),
                        slot(commands::FONT_SIZE_DECREASE),
                    ],
                },
                ManagerTabLayout {
                    id: CommandContext::Image.tab_id().into(),
                    label: CommandContext::Image.label().into(),
                    phase: ToolbarPhase::Active,
                    commands: vec![
                        slot(commands::FONT_SIZE_INCREASE),
                        slot(commands::FONT_SIZE_DECREASE),
                    ],
                },
                ManagerTabLayout {
                    id: CommandContext::Audio.tab_id().into(),
                    label: CommandContext::Audio.label().into(),
                    phase: ToolbarPhase::Planned,
                    commands: vec![],
                },
            ],
            floating_bars: vec![FloatingToolbarLayout {
                id: "quick-text".into(),
                label: "Quick".into(),
                anchor: ToolbarAnchor::Docked,
                x: 0.0,
                y: 0.0,
                commands: vec![slot(commands::TEXT_SAVE)],
            }],
        }
    }

    pub fn merge_user(&mut self, other: CommandManagerFile) {
        if !other.tabs.is_empty() {
            self.tabs = other.tabs;
        }
        if !other.floating_bars.is_empty() {
            self.floating_bars = other.floating_bars;
        }
        if !other.selected_tab.is_empty() {
            self.selected_tab = other.selected_tab;
        }
    }
}

fn slot(command_id: &str) -> UiCommandSlot {
    UiCommandSlot {
        command_id: command_id.into(),
    }
}

impl CommandManagerState {
    pub fn from_file(file: &CommandManagerFile, auto_context: CommandContext) -> Self {
        let selected = if file.tabs.iter().any(|t| t.id == file.selected_tab) {
            file.selected_tab.clone()
        } else {
            auto_context.tab_id().to_string()
        };
        Self {
            selected_tab: selected,
            auto_context,
            tabs: file.tabs.clone(),
            floating_bars: file.floating_bars.clone(),
        }
    }

    pub fn select_tab_for_context(&mut self, ctx: CommandContext) {
        self.auto_context = ctx;
        let id = ctx.tab_id();
        if self.tabs.iter().any(|t| t.id == id) {
            self.selected_tab = id.to_string();
        }
    }

    pub fn active_tab(&self) -> Option<&ManagerTabLayout> {
        self.tabs.iter().find(|t| t.id == self.selected_tab)
    }

    pub fn resolve_tab_commands(
        &self,
        registry: &CommandRegistry,
        host: &HostContext,
        doc: Option<&crate::document::DocumentSession>,
        tab_id: &str,
    ) -> Vec<ResolvedUiCommand> {
        let tab = self.tabs.iter().find(|t| t.id == tab_id);
        match tab {
            Some(tab) => self.resolve_slots(registry, host, doc, &tab.commands),
            None => Vec::new(),
        }
    }

    pub fn resolve_floating_bar(
        &self,
        registry: &CommandRegistry,
        host: &HostContext,
        doc: Option<&crate::document::DocumentSession>,
        bar_id: &str,
    ) -> Option<Vec<ResolvedUiCommand>> {
        let bar = self.floating_bars.iter().find(|b| b.id == bar_id)?;
        Some(self.resolve_slots(registry, host, doc, &bar.commands))
    }

    fn resolve_slots(
        &self,
        registry: &CommandRegistry,
        host: &HostContext,
        doc: Option<&crate::document::DocumentSession>,
        slots: &[UiCommandSlot],
    ) -> Vec<ResolvedUiCommand> {
        slots
            .iter()
            .filter_map(|slot| {
                let meta = registry.command_meta(&slot.command_id)?;
                let en = if let Some(doc) = doc {
                    registry.enablement(&slot.command_id, doc)
                } else {
                    registry.enablement_ctx(&slot.command_id, host)
                };
                Some(ResolvedUiCommand {
                    command_id: slot.command_id.clone(),
                    title: meta.title.to_string(),
                    icon: meta.icon.to_string(),
                    enabled: en.is_enabled(),
                    reason: en.disabled_reason().map(String::from),
                    surfaces: vec![
                        CommandSurface::CommandManagerTab,
                        CommandSurface::FloatingToolbar,
                        CommandSurface::CommandPalette,
                    ],
                })
            })
            .collect()
    }

    pub fn to_file(&self) -> CommandManagerFile {
        CommandManagerFile {
            selected_tab: self.selected_tab.clone(),
            tabs: self.tabs.clone(),
            floating_bars: self.floating_bars.clone(),
        }
    }
}

pub fn host_context_for_browse() -> HostContext {
    HostContext {
        mode: unibrowser_mod_api::Mode::Browse,
        doc_type: String::new(),
        backend_id: String::new(),
        selection: unibrowser_mod_api::SelectionSnapshot::default(),
        capabilities: unibrowser_mod_api::Capabilities::default(),
    }
}


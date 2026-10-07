use serde::{Deserialize, Serialize};
use unibrowser_mod_api::{
    CapabilityState, CommandContext, HostContext, ResolvedControl, ResolvedFamily,
    enablement_from_capability,
};

use crate::command::CommandRegistry;
use crate::command_system::{
    CapabilityResolver, LayoutStore, ResolutionContext, core_tab_layout,
};
use crate::ModRuntimeRegistry;

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

/// How `x` / `y` for floating bars are interpreted when persisted in `toolbars.json`.
/// Where a floating bar is rendered (main webview overlay vs its own OS window).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ToolbarPresentation {
    #[default]
    InWindow,
    DetachedWindow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ToolbarCoordSpace {
    /// Legacy: CSS `left` / `top` in the webview viewport (client pixels).
    #[default]
    Viewport,
    /// Physical screen position; shell converts using `window.screenX` / `screenY` on load.
    Screen,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloatingToolbarLayout {
    pub id: String,
    pub label: String,
    pub anchor: ToolbarAnchor,
    pub x: f64,
    pub y: f64,
    #[serde(default)]
    pub coord_space: ToolbarCoordSpace,
    #[serde(default)]
    pub presentation: ToolbarPresentation,
    pub commands: Vec<UiCommandSlot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandManagerFile {
    pub selected_tab: String,
    /// When set, auto context changes do not change `selected_tab` (SOLIDWORKS-style pin).
    #[serde(default)]
    pub context_pin: Option<String>,
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
    /// Tab id pinned by user; `None` = follow `auto_context`.
    pub context_pin: Option<String>,
    /// Manual CM tab browse (cleared when document context changes unless pinned).
    pub transient_tab: Option<String>,
    pub tabs: Vec<ManagerTabLayout>,
    pub floating_bars: Vec<FloatingToolbarLayout>,
    pub resolver: CapabilityResolver,
    pub layout_store: LayoutStore,
    pub context_revision: u64,
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
            context_pin: None,
            tabs: vec![
                ManagerTabLayout {
                    id: CommandContext::Text.tab_id().into(),
                    label: CommandContext::Text.label().into(),
                    phase: ToolbarPhase::Active,
                    commands: vec![
                        slot(commands::DOCUMENT_UNDO),
                        slot(commands::DOCUMENT_REDO),
                        slot(commands::FONT_SIZE_INCREASE),
                        slot(commands::FONT_SIZE_DECREASE),
                        slot(commands::FORMAT_DOCUMENT),
                        slot(commands::FIND_IN_DOCUMENT),
                        slot(commands::PROOFING_TOGGLE),
                        slot(commands::EXPORT_PRINT_PDF),
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
            floating_bars: vec![],
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
        if other.context_pin.is_some() {
            self.context_pin = other.context_pin;
        }
        self.sanitize_layout();
    }

    /// Drop legacy factory floating bars and upgrade the Text tab command seed.
    pub fn sanitize_layout(&mut self) {
        self.floating_bars
            .retain(|b| b.id != "quick-text" && !b.commands.is_empty());
        let factory = CommandManagerFile::factory_default();
        let text_id = CommandContext::Text.tab_id();
        let factory_text = factory
            .tabs
            .iter()
            .find(|t| t.id == text_id)
            .map(|t| t.commands.clone());
        if let Some(seed) = factory_text {
            if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == text_id) {
                let legacy = tab.commands.len() <= 3;
                if legacy {
                    tab.commands = seed;
                }
            }
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
            context_pin: file.context_pin.clone(),
            transient_tab: None,
            tabs: file.tabs.clone(),
            floating_bars: file.floating_bars.clone(),
            resolver: CapabilityResolver::default(),
            layout_store: LayoutStore::default(),
            context_revision: 1,
        }
    }

    pub fn with_layout_path(file: &CommandManagerFile, auto_context: CommandContext, path: Option<&std::path::Path>) -> Self {
        let mut state = Self::from_file(file, auto_context);
        state.layout_store = LayoutStore::load(path, file);
        let ctx_id = auto_context.tab_id();
        for id in state.layout_store.hidden_for_context(ctx_id) {
            state.resolver.hidden_controls.insert(id, true);
        }
        for (control, provider) in state.layout_store.provider_overrides_for(ctx_id) {
            state.resolver.user_provider_overrides.insert(control, provider);
        }
        state
    }

    pub fn bump_context_revision(&mut self) {
        self.context_revision += 1;
    }

    pub fn resolve_families_for_tab(
        &self,
        registry: &CommandRegistry,
        host: &HostContext,
        doc: Option<&crate::document::DocumentSession>,
        tab_id: &str,
        fallback_doc_path: Option<&str>,
        mod_contributions: Option<&[crate::command_system::LoadedModContribution]>,
        mod_runtime: Option<&ModRuntimeRegistry>,
    ) -> Vec<ResolvedFamily> {
        let structured_tab = matches!(tab_id, "start" | "text" | "code" | "image" | "office")
            || tab_id == self.auto_context.tab_id();
        if !structured_tab {
            return Vec::new();
        }
        let doc_path = doc
            .map(|d| d.path.clone())
            .or_else(|| fallback_doc_path.map(String::from));
        let ctx = ResolutionContext::new(
            self.auto_context,
            host.clone(),
            doc_path,
            self.context_revision,
        );
        let mut families = core_tab_layout(&ctx, &self.resolver);
        if let Some(contributions) = mod_contributions {
            let mut diag = crate::command_system::ContributionMergeDiagnostics::default();
            crate::command_system::merge_mod_contributions(&mut families, contributions, &mut diag);
            let _ = diag;
        }
        self.refine_capabilities(registry, host, doc, &ctx, mod_runtime, &mut families);
        families
    }

    fn refine_capabilities(
        &self,
        registry: &CommandRegistry,
        host: &HostContext,
        doc: Option<&crate::document::DocumentSession>,
        ctx: &ResolutionContext,
        mod_runtime: Option<&ModRuntimeRegistry>,
        families: &mut [ResolvedFamily],
    ) {
        for family in families.iter_mut() {
            for group in &mut family.groups {
                for control in &mut group.controls {
                    if matches!(control.capability, CapabilityState::HiddenByUser) {
                        continue;
                    }
                    let primary = control
                        .command_ids
                        .first()
                        .map(String::as_str)
                        .unwrap_or("");
                    control.capability = self.resolver.resolve_with_registry(
                        registry,
                        doc,
                        host,
                        &control.control_id,
                        primary,
                        ctx,
                    );
                    if let Some(runtime) = mod_runtime {
                        runtime.apply_to_control(control);
                    }
                }
                group.controls.retain(|c| !matches!(c.capability, CapabilityState::HiddenByUser));
            }
            family.groups.retain(|g| !g.controls.is_empty());
        }
    }

    pub fn flatten_resolved_controls(families: &[ResolvedFamily]) -> Vec<ResolvedUiCommand> {
        let mut out = Vec::new();
        for family in families {
            for group in &family.groups {
                for control in &group.controls {
                    let primary = control
                        .command_ids
                        .first()
                        .cloned()
                        .unwrap_or_else(|| control.control_id.clone());
                    let en = enablement_from_capability(&control.capability);
                    out.push(ResolvedUiCommand {
                        command_id: primary,
                        title: control.label.clone(),
                        icon: control.icon.clone(),
                        enabled: en.is_enabled(),
                        reason: en.disabled_reason().map(String::from),
                        surfaces: vec![
                            CommandSurface::CommandManagerTab,
                            CommandSurface::FloatingToolbar,
                            CommandSurface::CommandPalette,
                        ],
                    });
                }
            }
        }
        out
    }

    pub fn select_tab_manual(&mut self, tab_id: &str) {
        if self.tabs.iter().any(|t| t.id == tab_id) {
            self.transient_tab = Some(tab_id.to_string());
            self.selected_tab = tab_id.to_string();
        }
    }

    pub fn select_tab_for_context(&mut self, ctx: CommandContext) {
        self.auto_context = ctx;
        if self.context_pin.is_some() {
            return;
        }
        self.transient_tab = None;
        let id = ctx.tab_id();
        if self.tabs.iter().any(|t| t.id == id) {
            self.selected_tab = id.to_string();
        }
    }

    pub fn pin_selected_tab(&mut self) {
        self.context_pin = Some(self.selected_tab.clone());
        self.transient_tab = None;
    }

    pub fn clear_context_pin(&mut self) {
        self.context_pin = None;
        self.transient_tab = None;
        let id = self.auto_context.tab_id();
        if self.tabs.iter().any(|t| t.id == id) {
            self.selected_tab = id.to_string();
        }
    }

    pub fn is_context_pinned(&self) -> bool {
        self.context_pin.is_some()
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
        let families = self.resolve_families_for_tab(registry, host, doc, tab_id, None, None, None);
        if !families.is_empty() {
            return Self::flatten_resolved_controls(&families);
        }
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
            context_pin: self.context_pin.clone(),
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


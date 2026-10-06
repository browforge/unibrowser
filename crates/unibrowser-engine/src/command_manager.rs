use std::path::Path;

use unibrowser_mod_api::CommandContext;

use crate::ui_command::CommandManagerFile;

pub use crate::ui_command::{
    CommandManagerState, CommandSurface, FloatingToolbarLayout, ManagerTabLayout,
    ResolvedUiCommand, ToolbarAnchor, ToolbarCoordSpace, ToolbarPhase, ToolbarPresentation,
    UiCommandSlot,
};

pub fn load_command_manager_file(path: Option<&Path>) -> CommandManagerFile {
    let mut file = CommandManagerFile::factory_default();
    if let Some(path) = path {
        if let Ok(data) = std::fs::read_to_string(path) {
            if let Ok(user) = serde_json::from_str::<CommandManagerFile>(&data) {
                file.merge_user(user);
            }
        }
    }
    file
}

pub fn save_command_manager_file(path: &Path, file: &CommandManagerFile) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let data = serde_json::to_string_pretty(file).map_err(|e| e.to_string())?;
    std::fs::write(path, data).map_err(|e| e.to_string())
}

pub fn infer_context(
    is_directory: bool,
    is_http: bool,
    doc_path: Option<&str>,
    backend_mode: Option<unibrowser_mod_api::Mode>,
) -> CommandContext {
    if is_directory {
        return CommandContext::Browse;
    }
    if is_http {
        return CommandContext::Web;
    }
    if let (Some(mode), Some(path)) = (backend_mode, doc_path) {
        return CommandContext::from_mode_and_path(mode, path);
    }
    CommandContext::Browse
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui_command::{CommandManagerFile, CommandManagerState};
    use unibrowser_mod_api::{CommandContext, Mode};

    #[test]
    fn code_context_for_rust_file() {
        let ctx = infer_context(false, false, Some("main.rs"), Some(Mode::Text));
        assert_eq!(ctx, CommandContext::Code);
    }

    #[test]
    fn text_context_for_plain_txt() {
        let ctx = infer_context(false, false, Some("notes.txt"), Some(Mode::Text));
        assert_eq!(ctx, CommandContext::Text);
    }

    #[test]
    fn manual_cm_tab_clears_on_context_change() {
        let file = CommandManagerFile::factory_default();
        let mut state = CommandManagerState::from_file(&file, CommandContext::Text);
        state.select_tab_manual(CommandContext::Office.tab_id());
        assert_eq!(state.selected_tab, CommandContext::Office.tab_id());
        state.select_tab_for_context(CommandContext::Image);
        assert_eq!(state.auto_context, CommandContext::Image);
        assert_eq!(state.selected_tab, "image");
        assert!(state.transient_tab.is_none());
    }

    #[test]
    fn context_pin_blocks_auto_tab_switch() {
        let file = CommandManagerFile::factory_default();
        let mut state = CommandManagerState::from_file(&file, CommandContext::Text);
        state.pin_selected_tab();
        state.select_tab_for_context(CommandContext::Image);
        assert_eq!(state.auto_context, CommandContext::Image);
        assert_eq!(state.selected_tab, "text");
        state.clear_context_pin();
        assert_eq!(state.selected_tab, "image");
    }
}

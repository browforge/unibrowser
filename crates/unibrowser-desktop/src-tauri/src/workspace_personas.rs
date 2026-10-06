//! Affinity-style studio personas: one-click Command Manager context + tab focus.

use serde::Serialize;
use unibrowser_mod_api::CommandContext;

use crate::config::toolbars_file_path;
use crate::state::AppState;
use unibrowser_engine::save_command_manager_file;

#[derive(Debug, Clone, Serialize)]
pub struct WorkspacePersonaDto {
    pub id: String,
    pub name: String,
    pub description: String,
}

pub fn list_personas() -> Vec<WorkspacePersonaDto> {
    vec![
        WorkspacePersonaDto {
            id: "text-studio".into(),
            name: "Text studio".into(),
            description: "Focus Text commands; follow document context.".into(),
        },
        WorkspacePersonaDto {
            id: "office-studio".into(),
            name: "Office studio".into(),
            description: "Focus Office / documents; follow document context.".into(),
        },
        WorkspacePersonaDto {
            id: "image-studio".into(),
            name: "Image studio".into(),
            description: "Focus Image commands; follow document context.".into(),
        },
        WorkspacePersonaDto {
            id: "web-studio".into(),
            name: "Web studio".into(),
            description: "Focus Code + web tooling; follow document context.".into(),
        },
    ]
}

fn persona_tab(id: &str) -> Option<&'static str> {
    match id {
        "text-studio" => Some(CommandContext::Text.tab_id()),
        "office-studio" => Some(CommandContext::Office.tab_id()),
        "image-studio" => Some(CommandContext::Image.tab_id()),
        "web-studio" => Some(CommandContext::Code.tab_id()),
        _ => None,
    }
}

pub fn apply_persona(state: &AppState, persona_id: &str) -> Result<(), String> {
    let tab = persona_tab(persona_id).ok_or_else(|| format!("unknown persona: {}", persona_id))?;
    let mut cm = state.command_manager.lock().map_err(|e| e.to_string())?;
    cm.context_pin = None;
    cm.transient_tab = None;
    cm.selected_tab = tab.to_string();
    let file = cm.to_file();
    let path = toolbars_file_path().ok_or("config path unavailable")?;
    save_command_manager_file(&path, &file)?;
    Ok(())
}

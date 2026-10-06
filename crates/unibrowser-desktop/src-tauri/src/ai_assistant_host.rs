//! AI assistant prefs, grants, and connection test (ADR 0026).

use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::State;
use unibrowser_ai_assistant::{
    ActiveDocContext, AiPermission, AiScope, AssistantSession, ChatClientConfig, ChatMessage,
    PermissionStore, chat_completions, chat_completions_stream_collect,
};
use tauri::{AppHandle, Emitter};
use unibrowser_mod_api::TextProjection;
use unibrowser_mod_api::{BrowserLocation, Enablement};
use unibrowser_projections::{ProjectionExtras, project_document};

use crate::preferences::load_preferences;
use crate::project_backend::UnibrowBackend;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AiGrantDto {
    pub permission: String,
    pub scope_kind: String,
    pub scope_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AiAssistantPrefs {
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub api_key: Option<String>,
    pub grants: Vec<AiGrantDto>,
    pub panel_visible: bool,
    pub panel_x: f64,
    pub panel_y: f64,
    pub panel_docked: bool,
}

fn prefs_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("unibrowser").join("ai-assistant.json"))
}

pub fn load_prefs() -> AiAssistantPrefs {
    let path = prefs_path();
    if let Some(p) = path {
        if let Ok(text) = fs::read_to_string(&p) {
            if let Ok(prefs) = serde_json::from_str(&text) {
                return prefs;
            }
        }
    }
    AiAssistantPrefs::default()
}

pub fn save_prefs(prefs: &AiAssistantPrefs) -> Result<(), String> {
    let path = prefs_path().ok_or("config dir unavailable")?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let text = serde_json::to_string_pretty(prefs).map_err(|e| e.to_string())?;
    fs::write(&path, text).map_err(|e| e.to_string())?;
    Ok(())
}

fn grants_to_store(grants: &[AiGrantDto]) -> PermissionStore {
    let mut store = PermissionStore::default();
    for g in grants {
        let permission = match g.permission.as_str() {
            "read" => AiPermission::Read,
            "write" => AiPermission::Write,
            "ask" => AiPermission::Ask,
            _ => continue,
        };
        let scope = match g.scope_kind.as_str() {
            "global" => AiScope::Global,
            "workspace" => AiScope::Workspace {
                root: g.scope_value.clone().unwrap_or_default(),
            },
            "file_type" => AiScope::FileType {
                ext: g.scope_value.clone().unwrap_or_default(),
            },
            "site" => AiScope::SiteOrigin {
                origin: g.scope_value.clone().unwrap_or_default(),
            },
            _ => AiScope::Global,
        };
        store.grant(permission, scope);
    }
    store
}

static SESSION: Mutex<Option<AssistantSession>> = Mutex::new(None);

fn session_from_prefs(prefs: &AiAssistantPrefs) -> AssistantSession {
    AssistantSession::new(grants_to_store(&prefs.grants))
}

fn active_context(state: &AppState, window_label: &str) -> Result<ActiveDocContext, String> {
    let prefs = load_preferences();
    let sess = state.session(window_label)?;
    let tabs = sess.tabs.lock().map_err(|e| e.to_string())?;
    let tab = tabs.active_tab().ok_or("no active tab")?;
    let (ext, origin) = match &tab.location {
        BrowserLocation::File { path } => (
            path.extension()
                .and_then(|e| e.to_str())
                .map(|s| s.to_string()),
            None,
        ),
        BrowserLocation::Http { url } => {
            let origin = url::Url::parse(url.as_str())
                .ok()
                .map(|u| u.origin().ascii_serialization());
            (None, origin)
        }
        BrowserLocation::Internal { .. } => (None, None),
    };
    Ok(ActiveDocContext {
        workspace_root: prefs.workspace_root.clone(),
        file_ext: ext,
        site_origin: origin,
    })
}

#[derive(Serialize)]
pub struct AiAssistantStatusDto {
    pub configured: bool,
    pub panel_visible: bool,
    pub can_see_tab: bool,
    pub pending: Vec<PendingActionDto>,
}

#[derive(Serialize)]
pub struct PendingActionDto {
    pub id: u64,
    pub summary: String,
    pub preview: String,
}

#[tauri::command]
pub fn get_ai_assistant_prefs() -> AiAssistantPrefs {
    load_prefs()
}

#[tauri::command]
pub fn set_ai_assistant_prefs(prefs: AiAssistantPrefs) -> Result<(), String> {
    save_prefs(&prefs)?;
    if let Ok(mut s) = SESSION.lock() {
        *s = Some(session_from_prefs(&prefs));
    }
    Ok(())
}

#[tauri::command]
pub fn test_ai_connection(
    base_url: Option<String>,
    model: Option<String>,
    api_key: Option<String>,
) -> Result<String, String> {
    let cfg = ChatClientConfig {
        base_url,
        api_key,
    };
    let model = model.filter(|m| !m.is_empty()).unwrap_or_else(|| "test".into());
    chat_completions(
        &cfg,
        &model,
        &[unibrowser_ai_assistant::ChatMessage {
            role: "user".into(),
            content: "ping".into(),
        }],
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn ai_assistant_invoke(window: tauri::Window, state: State<AppState>) -> Result<(), String> {
    let prefs = load_prefs();
    let mut guard = SESSION.lock().map_err(|e| e.to_string())?;
    if guard.is_none() {
        *guard = Some(session_from_prefs(&prefs));
    }
    guard.as_mut().expect("session").user_invoke();
    Ok(())
}

#[tauri::command]
pub fn ai_assistant_status(
    window: tauri::Window,
    state: State<AppState>,
) -> Result<AiAssistantStatusDto, String> {
    let prefs = load_prefs();
    let configured = prefs
        .base_url
        .as_ref()
        .map(|s| !s.is_empty())
        .unwrap_or(false);
    let ctx = active_context(state.inner(), window.label())?;
    let mut guard = SESSION.lock().map_err(|e| e.to_string())?;
    if guard.is_none() {
        *guard = Some(session_from_prefs(&prefs));
    }
    let session = guard.as_mut().expect("session");
    let can_see = session.can_see_active_tab(&ctx);
    let pending = session
        .pending_actions()
        .iter()
        .map(|p| PendingActionDto {
            id: p.id,
            summary: p.summary.clone(),
            preview: p.replacement_text.clone(),
        })
        .collect();
    Ok(AiAssistantStatusDto {
        configured,
        panel_visible: prefs.panel_visible,
        can_see_tab: can_see,
        pending,
    })
}

#[derive(Deserialize)]
pub struct ProjectionExtrasDto {
    pub web_dom_text: Option<String>,
    pub ocr_text: Option<String>,
    pub epub_chapter_text: Option<String>,
    pub pdf_page_text: Option<String>,
    pub docx_preview_text: Option<String>,
}

#[derive(Serialize)]
pub struct DocumentProjectionDto {
    pub text: String,
    pub raster_width: Option<u32>,
    pub raster_height: Option<u32>,
}

fn document_projection_snapshot_inner(
    window_label: &str,
    state: &AppState,
    extras: ProjectionExtrasDto,
) -> Result<DocumentProjectionDto, String> {
    let sess = state.session(window_label)?;
    let mut tabs = sess.tabs.lock().map_err(|e| e.to_string())?;
    let tab = tabs.active_tab_mut().ok_or("no active tab")?;
    let doc = tab.document.as_mut().ok_or("no document")?;
    let mut projection_extras = ProjectionExtras {
        web_dom_text: extras.web_dom_text,
        ocr_text: extras.ocr_text,
        epub_chapter_text: extras.epub_chapter_text,
        pdf_page_text: extras.pdf_page_text,
        docx_preview_text: extras.docx_preview_text,
        unibrow_manifest: None,
    };
    if let Some(ub) = doc
        .backend
        .as_any_mut()
        .downcast_ref::<UnibrowBackend>()
    {
        projection_extras.unibrow_manifest = Some(ub.package.manifest.clone());
    }
    let bundle = project_document(doc, &projection_extras);
    Ok(DocumentProjectionDto {
        text: bundle.text,
        raster_width: bundle.raster.as_ref().map(|r| r.width),
        raster_height: bundle.raster.as_ref().map(|r| r.height),
    })
}

#[tauri::command]
pub fn document_projection_snapshot(
    window: tauri::Window,
    state: State<AppState>,
    extras: ProjectionExtrasDto,
) -> Result<DocumentProjectionDto, String> {
    document_projection_snapshot_inner(window.label(), state.inner(), extras)
}

#[derive(Serialize)]
pub struct AiToolDto {
    pub command_id: String,
    pub title: String,
    pub enabled: bool,
    pub reason: Option<String>,
}

#[tauri::command]
pub fn ai_assistant_tools(
    window: tauri::Window,
    state: State<AppState>,
) -> Result<Vec<AiToolDto>, String> {
    let sess = state.session(window.label())?;
    let registry = state.registry.lock().map_err(|e| e.to_string())?;
    let tabs = sess.tabs.lock().map_err(|e| e.to_string())?;
    let tab = tabs.active_tab().ok_or("no active tab")?;
    let doc = tab.document.as_ref();
    let mut out = Vec::new();
    for id in registry.sorted_command_ids() {
        let meta = registry.command_meta(&id).expect("meta");
        let en = doc
            .map(|d| registry.enablement(&id, d))
            .unwrap_or(Enablement::Disabled {
                reason: "No active document".into(),
            });
        let enabled = en.is_enabled();
        let reason = en.disabled_reason().map(|s| s.to_string());
        out.push(AiToolDto {
            command_id: id,
            title: meta.title.to_string(),
            enabled,
            reason: if enabled { None } else { reason },
        });
    }
    out.sort_by(|a, b| a.title.cmp(&b.title));
    Ok(out)
}

struct ProjectionText(String);
impl TextProjection for ProjectionText {
    fn text_projection(&self) -> String {
        self.0.clone()
    }
}

#[tauri::command]
pub fn ai_assistant_read_tab_text(
    window: tauri::Window,
    state: State<AppState>,
    extras: ProjectionExtrasDto,
) -> Result<String, String> {
    let dto = document_projection_snapshot_inner(window.label(), state.inner(), extras)?;
    let ctx = active_context(state.inner(), window.label())?;
    let mut guard = SESSION.lock().map_err(|e| e.to_string())?;
    if guard.is_none() {
        *guard = Some(session_from_prefs(&load_prefs()));
    }
    let session = guard.as_mut().ok_or("session")?;
    session.read_active_projection(&ProjectionText(dto.text), &ctx)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn ai_assistant_propose_write(
    window: tauri::Window,
    state: State<AppState>,
    summary: String,
    replacement: String,
) -> Result<Option<u64>, String> {
    let ctx = active_context(state.inner(), window.label())?;
    let mut guard = SESSION.lock().map_err(|e| e.to_string())?;
    if guard.is_none() {
        *guard = Some(session_from_prefs(&load_prefs()));
    }
    let session = guard.as_mut().ok_or("session")?;
    session
        .propose_write(&ctx, summary, replacement)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn ai_assistant_approve_pending(id: u64) -> Result<String, String> {
    let mut guard = SESSION.lock().map_err(|e| e.to_string())?;
    let session = guard.as_mut().ok_or("session not started")?;
    session
        .approve_pending(id)
        .ok_or_else(|| "pending action not found".into())
}

#[derive(Deserialize)]
pub struct AiChatRequest {
    pub messages: Vec<ChatMessageDto>,
    pub extras: ProjectionExtrasDto,
    pub include_tab_context: bool,
}

#[derive(Deserialize, Clone)]
pub struct ChatMessageDto {
    pub role: String,
    pub content: String,
}

#[derive(Serialize, Clone)]
pub struct AiChatDeltaPayload {
    pub delta: String,
}

#[derive(Serialize, Clone)]
pub struct AiChatDonePayload {
    pub content: String,
}

#[derive(Serialize, Clone)]
pub struct AiChatErrorPayload {
    pub message: String,
}

#[tauri::command]
pub fn ai_assistant_chat(
    window: tauri::Window,
    app: AppHandle,
    state: State<AppState>,
    req: AiChatRequest,
) -> Result<(), String> {
    let prefs = load_prefs();
    let cfg = ChatClientConfig {
        base_url: prefs.base_url.clone(),
        api_key: prefs.api_key.clone(),
    };
    let model = prefs
        .model
        .clone()
        .filter(|m| !m.is_empty())
        .unwrap_or_else(|| "gpt-4o-mini".into());
    let ctx = active_context(state.inner(), window.label())?;
    let mut guard = SESSION.lock().map_err(|e| e.to_string())?;
    if guard.is_none() {
        *guard = Some(session_from_prefs(&prefs));
    }
    let session = guard.as_mut().ok_or("session")?;
    session.user_invoke();
    let mut messages: Vec<ChatMessage> = req
        .messages
        .iter()
        .map(|m| ChatMessage {
            role: m.role.clone(),
            content: m.content.clone(),
        })
        .collect();
    if req.include_tab_context {
        if !session.can_see_active_tab(&ctx) {
            return Err("read not granted for active tab".into());
        }
        let dto = document_projection_snapshot_inner(window.label(), state.inner(), req.extras)?;
        let context = format!(
            "Active document projection (user granted Read):\n\n{}",
            dto.text.chars().take(24_000).collect::<String>()
        );
        messages.insert(
            0,
            ChatMessage {
                role: "system".into(),
                content: context,
            },
        );
    }
    std::thread::spawn(move || {
        let mut on_delta = |delta: &str| {
            let _ = app.emit(
                "ai-assistant-chat-delta",
                AiChatDeltaPayload {
                    delta: delta.to_string(),
                },
            );
        };
        let result = chat_completions_stream_collect(&cfg, &model, &messages, &mut on_delta);
        match result {
            Ok(full) => {
                let _ = app.emit("ai-assistant-chat-done", AiChatDonePayload { content: full });
            }
            Err(e) => {
                let _ = app.emit(
                    "ai-assistant-chat-error",
                    AiChatErrorPayload {
                        message: e.to_string(),
                    },
                );
            }
        }
    });
    Ok(())
}

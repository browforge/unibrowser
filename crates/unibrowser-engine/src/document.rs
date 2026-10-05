use unibrowser_mod_api::{Capabilities, Mode, SelectionSnapshot};

/// Per-document backend state (implemented by mods).
pub trait DocumentBackend: Send {
    fn backend_id(&self) -> &str;
    fn doc_type(&self) -> &str;
    fn display_name(&self) -> &str;
    fn mode(&self) -> Mode;
    fn capabilities(&self) -> Capabilities;
    fn selection(&self) -> SelectionSnapshot;
    fn increase_font_size(&mut self) -> Result<f32, String>;
    fn decrease_font_size(&mut self) -> Result<f32, String>;
    fn status_line(&self) -> String;
    /// Demo hook for skeleton UI (simulates text selection).
    fn set_demo_text_selection(&mut self, selected: bool);
}

pub struct DocumentSession {
    pub path: String,
    pub backend: Box<dyn DocumentBackend>,
}

impl DocumentSession {
    pub fn new(path: impl Into<String>, backend: Box<dyn DocumentBackend>) -> Self {
        Self {
            path: path.into(),
            backend,
        }
    }

    pub fn host_context(&self) -> unibrowser_mod_api::HostContext {
        unibrowser_mod_api::HostContext {
            mode: self.backend.mode(),
            doc_type: self.backend.doc_type().to_string(),
            backend_id: self.backend.backend_id().to_string(),
            selection: self.backend.selection(),
            capabilities: self.backend.capabilities(),
        }
    }

    pub fn host_context_mut(&mut self) -> unibrowser_mod_api::HostContext {
        self.host_context()
    }
}

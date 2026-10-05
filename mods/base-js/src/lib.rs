use std::path::Path;

use rhai::{Engine, Scope};
use unibrowser_engine::CommandRegistry;
use unibrowser_mod_api::{Capabilities, Mode, SelectionSnapshot};

pub const BACKEND_ID: &str = "base-js";
pub const VIEWER_KIND: &str = "script";

pub struct ScriptBackend {
    pub path: String,
    pub source: String,
    pub last_output: String,
}

impl ScriptBackend {
    pub fn from_path(path: &Path) -> Result<Self, String> {
        let source = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        Ok(Self {
            path: path.to_string_lossy().into_owned(),
            source,
            last_output: String::new(),
        })
    }

    pub fn eval_snippet(source: &str) -> Result<String, String> {
        let engine = Engine::new();
        let mut scope = Scope::new();
        let value = engine
            .eval_with_scope::<rhai::Dynamic>(&mut scope, source)
            .map_err(|e| e.to_string())?;
        Ok(format!("{}", value))
    }
}

impl unibrowser_engine::document::DocumentBackend for ScriptBackend {
    fn backend_id(&self) -> &str {
        BACKEND_ID
    }
    fn doc_type(&self) -> &str {
        "application/x-unibrowser-script"
    }
    fn display_name(&self) -> &str {
        "Script"
    }
    fn mode(&self) -> Mode {
        Mode::Text
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities::default()
    }
    fn selection(&self) -> SelectionSnapshot {
        SelectionSnapshot::default()
    }
    fn increase_font_size(&mut self) -> Result<f32, String> {
        Err("N/A".into())
    }
    fn decrease_font_size(&mut self) -> Result<f32, String> {
        Err("N/A".into())
    }
    fn set_demo_text_selection(&mut self, _selected: bool) {}
    fn status_line(&self) -> String {
        if self.last_output.is_empty() {
            format!("Script · {} (Rhai)", self.path)
        } else {
            format!("Script · last: {}", self.last_output)
        }
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

pub fn register(_registry: &mut CommandRegistry) {}

pub fn open_file(path: &Path) -> Result<Box<dyn unibrowser_engine::document::DocumentBackend>, String> {
    Ok(Box::new(ScriptBackend::from_path(path)?))
}

pub fn try_script_backend(
    backend: &mut dyn unibrowser_engine::document::DocumentBackend,
) -> Option<&mut ScriptBackend> {
    backend.as_any_mut().downcast_mut::<ScriptBackend>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rhai_eval_adds() {
        assert_eq!(ScriptBackend::eval_snippet("1 + 2").unwrap(), "3");
    }
}

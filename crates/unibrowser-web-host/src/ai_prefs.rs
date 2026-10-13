use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiAssistantPrefsWeb {
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub api_key: Option<String>,
    pub grants: Vec<Value>,
    pub panel_visible: bool,
    pub panel_x: f64,
    pub panel_y: f64,
    pub panel_docked: bool,
    pub panel_width: f64,
}

impl Default for AiAssistantPrefsWeb {
    fn default() -> Self {
        Self {
            base_url: None,
            model: None,
            api_key: None,
            grants: vec![],
            panel_visible: false,
            panel_x: 24.0,
            panel_y: 96.0,
            panel_docked: true,
            panel_width: 320.0,
        }
    }
}

#[derive(Serialize)]
pub struct AiAssistantStatusWeb {
    pub configured: bool,
    pub panel_visible: bool,
    pub can_see_tab: bool,
    pub pending: Vec<Value>,
}

impl AiAssistantPrefsWeb {
    pub fn merge_from_args(&mut self, args: &Value) {
        let prefs = args.get("prefs").or(Some(args));
        if let Some(p) = prefs {
            if let Some(v) = p.get("base_url").and_then(|v| v.as_str()) {
                self.base_url = Some(v.to_string());
            }
            if let Some(v) = p.get("model").and_then(|v| v.as_str()) {
                self.model = Some(v.to_string());
            }
            if let Some(v) = p.get("panel_visible").and_then(|v| v.as_bool()) {
                self.panel_visible = v;
            }
            if let Some(v) = p.get("panel_docked").and_then(|v| v.as_bool()) {
                self.panel_docked = v;
            }
            if let Some(v) = p.get("panel_width").and_then(|v| v.as_f64()) {
                self.panel_width = v;
            }
            if let Some(v) = p.get("panel_x").and_then(|v| v.as_f64()) {
                self.panel_x = v;
            }
            if let Some(v) = p.get("panel_y").and_then(|v| v.as_f64()) {
                self.panel_y = v;
            }
        }
    }

    pub fn status(&self) -> AiAssistantStatusWeb {
        AiAssistantStatusWeb {
            configured: false,
            panel_visible: self.panel_visible,
            can_see_tab: false,
            pending: vec![],
        }
    }
}

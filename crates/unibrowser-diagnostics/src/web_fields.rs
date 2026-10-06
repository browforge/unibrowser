use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WebFieldKind {
    TextInput,
    Search,
    Email,
    TextArea,
    ContentEditable,
    Password,
    CreditCard,
    OneTimeCode,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebFieldProbe {
    pub tag: String,
    pub input_type: Option<String>,
    pub autocomplete: Option<String>,
    pub spellcheck: Option<bool>,
}

pub fn classify_field(probe: &WebFieldProbe) -> WebFieldKind {
    let tag = probe.tag.to_lowercase();
    if tag == "input" {
        let t = probe
            .input_type
            .as_deref()
            .unwrap_or("text")
            .to_lowercase();
        return match t.as_str() {
            "password" => WebFieldKind::Password,
            "email" => WebFieldKind::Email,
            "search" => WebFieldKind::Search,
            "text" => WebFieldKind::TextInput,
            _ => WebFieldKind::Skipped,
        };
    }
    if tag == "textarea" {
        return WebFieldKind::TextArea;
    }
    if probe.autocomplete.as_deref().is_some_and(|a| {
        let a = a.to_lowercase();
        a.contains("cc-") || a == "one-time-code"
    }) {
        return if probe
            .autocomplete
            .as_deref()
            .is_some_and(|a| a.to_lowercase().contains("cc-"))
        {
            WebFieldKind::CreditCard
        } else {
            WebFieldKind::OneTimeCode
        };
    }
    WebFieldKind::Skipped
}

pub fn field_may_proof(probe: &WebFieldProbe) -> bool {
    if probe.spellcheck == Some(false) {
        return false;
    }
    match classify_field(probe) {
        WebFieldKind::Password | WebFieldKind::CreditCard | WebFieldKind::OneTimeCode | WebFieldKind::Skipped => {
            false
        }
        WebFieldKind::TextInput
        | WebFieldKind::Search
        | WebFieldKind::Email
        | WebFieldKind::TextArea
        | WebFieldKind::ContentEditable => true,
    }
}

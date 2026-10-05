//! Minimal `activationEvents` evaluation for the VSIX stub host ([ADR 0018]).

use std::sync::Mutex;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct ActivationRecord {
    pub extension_id: String,
    pub matched_event: String,
    pub command_id: String,
    pub at_ms: u64,
}

static ACTIVATED: Mutex<Vec<ActivationRecord>> = Mutex::new(Vec::new());

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Returns the `onCommand:` activation event that matched, if any.
pub fn matching_on_command_event(command_id: &str, activation_events: &[String]) -> Option<String> {
    let needle = format!("onCommand:{}", command_id);
    activation_events
        .iter()
        .find(|e| e.trim() == needle)
        .cloned()
}

/// Record activation when a contributed command runs and `onCommand:` matches.
/// Returns `(record, fresh)` when the event matched; `fresh` is true the first time per extension.
pub fn record_command_activation(
    extension_id: &str,
    command_id: &str,
    activation_events: &[String],
) -> Option<(ActivationRecord, bool)> {
    let matched = matching_on_command_event(command_id, activation_events)?;
    if let Ok(mut list) = ACTIVATED.lock() {
        if let Some(existing) = list.iter().find(|r| r.extension_id == extension_id) {
            return Some((existing.clone(), false));
        }
        let record = ActivationRecord {
            extension_id: extension_id.to_string(),
            matched_event: matched,
            command_id: command_id.to_string(),
            at_ms: now_ms(),
        };
        list.push(record.clone());
        return Some((record, true));
    }
    None
}

pub fn activated_extensions() -> Vec<ActivationRecord> {
    ACTIVATED
        .lock()
        .map(|list| list.clone())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn on_command_event_matches() {
        let events = vec!["onCommand:demo.hello".into(), "onLanguage:json".into()];
        assert_eq!(
            matching_on_command_event("demo.hello", &events).as_deref(),
            Some("onCommand:demo.hello")
        );
        assert!(matching_on_command_event("other", &events).is_none());
    }
}

//! OpenAI-compatible chat client — no default cloud endpoint (ADR 0026).

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Default)]
pub struct ChatClientConfig {
    /// User-configured base URL (e.g. `http://127.0.0.1:11434/v1`). None = not configured.
    pub base_url: Option<String>,
    pub api_key: Option<String>,
}

#[derive(Debug, Error)]
pub enum ChatError {
    #[error("no API base URL configured")]
    NoBaseUrl,
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("invalid response: {0}")]
    Response(String),
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ChatMessage,
}

pub fn chat_completions(
    config: &ChatClientConfig,
    model: &str,
    messages: &[ChatMessage],
) -> Result<String, ChatError> {
    let base = config
        .base_url
        .as_deref()
        .filter(|s| !s.is_empty())
        .ok_or(ChatError::NoBaseUrl)?;
    let url = format!(
        "{}/chat/completions",
        base.trim_end_matches('/')
    );
    let body = ChatRequest {
        model: model.to_string(),
        messages: messages.to_vec(),
        stream: None,
    };
    let mut req = ureq::post(&url).set("Content-Type", "application/json");
    if let Some(key) = config.api_key.as_deref().filter(|k| !k.is_empty()) {
        req = req.set("Authorization", &format!("Bearer {}", key));
    }
    let resp = req
        .send_json(body)
        .map_err(|e| ChatError::Http(e.to_string()))?;
    if resp.status() >= 400 {
        return Err(ChatError::Http(format!("status {}", resp.status())));
    }
    let parsed: ChatResponse = resp
        .into_json()
        .map_err(|e| ChatError::Response(e.to_string()))?;
    let content = parsed
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| ChatError::Response("empty choices".into()))?;
    Ok(content)
}

/// Collect streamed SSE deltas (OpenAI-compatible `data: {...}` lines).
pub fn chat_completions_stream_collect(
    config: &ChatClientConfig,
    model: &str,
    messages: &[ChatMessage],
    on_delta: &mut dyn FnMut(&str),
) -> Result<String, ChatError> {
    let base = config
        .base_url
        .as_deref()
        .filter(|s| !s.is_empty())
        .ok_or(ChatError::NoBaseUrl)?;
    let url = format!("{}/chat/completions", base.trim_end_matches('/'));
    let body = ChatRequest {
        model: model.to_string(),
        messages: messages.to_vec(),
        stream: Some(true),
    };
    let mut req = ureq::post(&url).set("Content-Type", "application/json");
    if let Some(key) = config.api_key.as_deref().filter(|k| !k.is_empty()) {
        req = req.set("Authorization", &format!("Bearer {}", key));
    }
    let resp = req
        .send_json(body)
        .map_err(|e| ChatError::Http(e.to_string()))?;
    if resp.status() >= 400 {
        return Err(ChatError::Http(format!("status {}", resp.status())));
    }
    let mut reader = resp.into_reader();
    let mut buf = Vec::new();
    let mut chunk = [0u8; 4096];
    let mut full = String::new();
    while let Ok(n) = std::io::Read::read(&mut reader, &mut chunk) {
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&chunk[..n]);
        while let Some(pos) = buf.iter().position(|&b| b == b'\n') {
            let line = String::from_utf8_lossy(&buf[..pos]).trim().to_string();
            buf.drain(..pos + 1);
            if !line.starts_with("data:") {
                continue;
            }
            let payload = line.trim_start_matches("data:").trim();
            if payload == "[DONE]" {
                break;
            }
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(payload) {
                let delta = v
                    .pointer("/choices/0/delta/content")
                    .and_then(|c| c.as_str())
                    .unwrap_or("");
                if !delta.is_empty() {
                    full.push_str(delta);
                    on_delta(delta);
                }
            }
        }
    }
    Ok(full)
}

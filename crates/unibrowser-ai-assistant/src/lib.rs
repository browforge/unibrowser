//! AI assistant mod core — OpenAI-compatible, permissioned, projection-based (ADR 0026).

mod assistant;
mod openai;
mod permissions;
mod redact;

pub use assistant::{
    AssistantError, AssistantSession, PendingAction, default_read_grant,
};
pub use openai::{ChatClientConfig, ChatError, ChatMessage, chat_completions};
pub use permissions::{
    ActiveDocContext, AiPermission, AiScope, Grant, PermissionStore, WriteMode,
};
pub use redact::redact_sensitive_projection;

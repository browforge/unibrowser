//! Canonical document model and format serialisers (ADR 0035).

pub mod docx;
pub mod history;
pub mod history_file;
pub mod markdown;
pub mod model;
pub mod reparse;
pub mod session;

pub use docx::{model_to_docx_html, parse_docx_to_model};
pub use history::{apply_history_op, invert_history_op, DocumentHistoryOp};
pub use markdown::{parse_markdown, serialize_markdown, MarkdownParseResult};
pub use model::{
    validate_document_model, Block, DocumentMetadata, DocumentModel, InlineMark, MarkKind,
    OpaqueBlock, SourceMap, SourceMapEntry, SCHEMA_ID,
};
pub use history_file::{
    append_document_op, list_ops, load_head, sidecar_path_for_file, HistoryLogEntry,
};
pub use reparse::{merge_source_text, mode_switch_preserves_bytes, source_projection, ModeSwitchReport, SourceMergeResult};
pub use session::DocumentEditSession;

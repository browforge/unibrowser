//! Canonical document model and format serialisers (ADR 0035).

pub mod docx;
pub mod history;
pub mod history_admin;
pub mod history_file;
pub mod history_pins;
pub mod history_replay;
pub mod history_undo;
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
    append_document_op, list_ops, load_head, pop_last_op, push_op_entry, sidecar_path_for_file,
    truncate_head, HistoryLogEntry, LOG_MARKER,
};
pub use history_admin::{
    jump_to_seq, session_state_at_seq, squash_op_range, trim_history_before,
};
pub use history_pins::{format_pin_header, parse_pin_header, upsert_pin_from_entry};
pub use history_replay::{
    copy_history_sidecar, header_pin_mismatch, replay_session_with_catalog, HistoryReplayBlock,
    HistoryReplayReport,
};
pub use history_undo::{
    decode_snapshot_blob, encode_snapshot_blob, serialize_session_state, session_at_history_head,
    session_state_after_bytes, session_state_after_ops, HistorySessionState,
};
pub use reparse::{merge_source_text, mode_switch_preserves_bytes, source_projection, ModeSwitchReport, SourceMergeResult};
pub use session::DocumentEditSession;

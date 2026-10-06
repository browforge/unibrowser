//! Legacy shared proofing (superseded by `unibrowser-diagnostics` + mods; ADR 0021–0022).

mod code_regions;
mod dictionaries;
mod engine;
mod grammar;
mod spelling;
mod web_fields;

pub use code_regions::{code_check_segments, CodeSegment, CodeSegmentKind};
pub use dictionaries::{
    DictionaryStore, IgnoreScope, ProofingDictionaries, WordListFile,
};
pub use engine::{
    CheckContext, Diagnostic, DiagnosticKind, ProofingEngine, ProofingLanguage, ProofingSurface,
};
pub use web_fields::{classify_field, field_may_proof, WebFieldKind, WebFieldProbe};

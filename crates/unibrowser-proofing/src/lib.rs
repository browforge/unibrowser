//! Shared offline proofing: spelling (Hunspell via spellbook) and grammar (Harper).

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

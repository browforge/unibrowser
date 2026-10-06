use unibrowser_proofing::{
    CheckContext, DictionaryStore, ProofingEngine, ProofingLanguage, ProofingSurface,
};

#[test]
fn spelling_flags_unknown_word() {
    let mut engine = ProofingEngine::with_bundled_dictionaries().expect("engine");
    let ctx = CheckContext {
        language: ProofingLanguage::EnNz,
        surface: ProofingSurface::PlainText,
        grammar_enabled: false,
        ..Default::default()
    };
    let diags = engine.check("hello wrng", &ctx);
    assert!(
        diags.iter().any(|d| d.message.contains("wrng")),
        "expected spelling diagnostic"
    );
}

#[test]
fn workspace_dictionary_overrides_global_for_added_word() {
    let mut engine = ProofingEngine::with_bundled_dictionaries().expect("engine");
    let mut workspace = DictionaryStore::default();
    workspace.added.insert("bearjcc".into());
    engine.load_user_dictionaries(DictionaryStore::default(), workspace);
    let ctx = CheckContext {
        grammar_enabled: false,
        ..Default::default()
    };
    let diags = engine.check("bearjcc", &ctx);
    assert!(diags.is_empty(), "workspace added word should not squiggle");
}

#[test]
fn ignore_list_suppresses_spelling() {
    let mut engine = ProofingEngine::with_bundled_dictionaries().expect("engine");
    let mut global = DictionaryStore::default();
    global.ignored.insert("wrng".into());
    engine.load_user_dictionaries(global, DictionaryStore::default());
    let ctx = CheckContext {
        grammar_enabled: false,
        ..Default::default()
    };
    let diags = engine.check("wrng", &ctx);
    assert!(diags.is_empty());
}

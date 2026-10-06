use unibrowser_diagnostics::DictionarySnapshot;

#[test]
fn workspace_dictionary_overrides_global_added() {
    let mut snap = DictionarySnapshot::default();
    snap.global.added.insert("colour".into());
    snap.workspace.added.insert("bearjcc".into());
    assert!(snap.workspace_wins_over_global_added("bearjcc"));
    assert!(!snap.should_suppress_spelling("unknownword"));
    snap.workspace.added.insert("unknownword".into());
    assert!(snap.should_suppress_spelling("unknownword"));
}

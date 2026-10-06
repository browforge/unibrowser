use unibrowser_proofing::{code_check_segments, CodeSegmentKind, CheckContext, ProofingEngine};

#[test]
fn code_adapter_checks_comments_not_identifiers() {
    let source = "fn main() { // wrng comment\n let x = \"ok\"; }";
    let segments = code_check_segments(source, "rust");
    assert!(segments.iter().any(|s| s.kind == CodeSegmentKind::Comment));
    assert!(segments.iter().any(|s| s.kind == CodeSegmentKind::Identifier));
    let ranges = segments
        .iter()
        .filter(|s| s.kind == CodeSegmentKind::Comment || s.kind == CodeSegmentKind::StringLiteral)
        .map(|s| (s.start, s.end))
        .collect();
    let mut engine = ProofingEngine::with_bundled_dictionaries().expect("engine");
    let ctx = CheckContext {
        surface: unibrowser_proofing::ProofingSurface::Code,
        check_ranges: Some(ranges),
        grammar_enabled: false,
        ..Default::default()
    };
    let diags = engine.check(source, &ctx);
    assert!(
        diags.iter().any(|d| source[d.start as usize..d.end as usize].contains("wrng")),
        "comment typo should be reported"
    );
    assert!(
        !diags.iter().any(|d| &source[d.start as usize..d.end as usize] == "main"),
        "identifier should not be checked"
    );
}

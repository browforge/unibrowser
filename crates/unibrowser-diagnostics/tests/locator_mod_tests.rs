use dxf_checker::DxfChecker;
use unibrow_checker::UnibrowChecker;
use unibrowser_diagnostics::{DiagnosticsHost, Locator};

#[test]
fn dxf_locator_diagnostic() {
    let mut host = DiagnosticsHost::new();
    host.register_checker(Box::new(DxfChecker));
    let problems = host.check_dxf("/x/a.dxf", "0\nTEXT\nwrng\n");
    assert!(problems.iter().any(|p| matches!(p.locator, Locator::DxfEntity { .. })));
}

#[test]
fn timeline_locator_diagnostic() {
    let mut host = DiagnosticsHost::new();
    host.register_checker(Box::new(UnibrowChecker));
    let json = r#"{"timeline":{"clips":[{"id":"c1","label":"t","title_text":"wrng title"}]}}"#;
    let problems = host.check_unibrow(json);
    assert!(problems.iter().any(|p| matches!(
        p.locator,
        Locator::TimelineRange { .. }
    )));
}

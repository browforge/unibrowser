use unibrowser_diagnostics::{field_may_proof, WebFieldProbe};

#[test]
fn password_field_skipped() {
    let probe = WebFieldProbe {
        tag: "input".into(),
        input_type: Some("password".into()),
        autocomplete: None,
        spellcheck: None,
    };
    assert!(!field_may_proof(&probe));
}

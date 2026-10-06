use unibrowser_proofing::{field_may_proof, WebFieldKind, WebFieldProbe};

#[test]
fn web_adapter_skips_password_fields() {
    let probe = WebFieldProbe {
        tag: "input".into(),
        input_type: Some("password".into()),
        autocomplete: None,
        spellcheck: None,
    };
    assert_eq!(unibrowser_proofing::classify_field(&probe), WebFieldKind::Password);
    assert!(!field_may_proof(&probe));
}

#[test]
fn web_adapter_allows_text_input() {
    let probe = WebFieldProbe {
        tag: "input".into(),
        input_type: Some("text".into()),
        autocomplete: None,
        spellcheck: None,
    };
    assert!(field_may_proof(&probe));
}

#[test]
fn web_adapter_respects_spellcheck_false() {
    let probe = WebFieldProbe {
        tag: "textarea".into(),
        input_type: None,
        autocomplete: None,
        spellcheck: Some(false),
    };
    assert!(!field_may_proof(&probe));
}

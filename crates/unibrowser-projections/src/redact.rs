pub fn redact_sensitive_text(text: &str) -> String {
    let mut out = text.to_string();
    for marker in [
        "type=\"password\"",
        "type='password'",
        "autocomplete=\"cc-number\"",
        "autocomplete=\"cc-csc\"",
    ] {
        if out.contains(marker) {
            out = out.replace(marker, "[redacted]");
        }
    }
    out
}

//! Strip sensitive fields from text projections before any model call.

/// Remove password-like and card fields from HTML-ish text surfaces.
pub fn redact_sensitive_projection(text: &str) -> String {
    let mut out = text.to_string();
    for marker in [
        "autocomplete=\"cc-number\"",
        "autocomplete=\"cc-csc\"",
        "type=\"password\"",
        "type='password'",
    ] {
        if out.contains(marker) {
            out = out.replace(marker, "[redacted]");
        }
    }
    out
}

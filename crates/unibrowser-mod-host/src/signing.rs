//! T1 mod signing (stub). See `docs/plugin-sdk/signing.md`.

use std::path::Path;

use crate::ModHostError;

const SIG_FILE: &str = "ub-mod.sig";

/// When `UNIBROWSER_REQUIRE_SIGNED_MODS=1`, mods must ship a valid `ub-mod.sig` (format stub only).
pub fn require_signed_mods() -> bool {
    std::env::var("UNIBROWSER_REQUIRE_SIGNED_MODS").as_deref() == Ok("1")
}

/// Verify signature metadata beside the native mod. Stub: accepts `t1;sha256=<hex>;sig=<base64>` lines.
pub fn verify_mod_dir(mod_dir: &Path) -> Result<(), ModHostError> {
    if !require_signed_mods() {
        return Ok(());
    }
    let sig_path = mod_dir.join(SIG_FILE);
    if !sig_path.is_file() {
        return Err(ModHostError::Dynamic(format!(
            "unsigned mod (set {} or add {})",
            SIG_FILE,
            SIG_FILE
        )));
    }
    let text = std::fs::read_to_string(&sig_path).map_err(ModHostError::Io)?;
    let line = text.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
    if !line.starts_with("t1;sha256=") || !line.contains(";sig=") {
        return Err(ModHostError::Dynamic(format!(
            "invalid {} (expected t1;sha256=<hex>;sig=<base64>)",
            SIG_FILE
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn unsigned_ok_when_not_required() {
        let dir = std::env::temp_dir().join(format!("ub-sign-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        verify_mod_dir(&dir).expect("unsigned allowed by default");
        let _ = fs::remove_dir_all(&dir);
    }
}

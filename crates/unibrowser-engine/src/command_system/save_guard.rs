use unibrowser_mod_api::{CapabilityState, commands};

pub fn save_commands_always_supported(command_id: &str) -> bool {
    matches!(
        command_id,
        commands::TEXT_SAVE | commands::RESTORE_AUTOSAVE
    )
}

pub fn assert_save_independent(cap: &CapabilityState) {
    assert!(
        matches!(cap, CapabilityState::Supported),
        "save capability must remain supported"
    );
}

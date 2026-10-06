use unibrowser_engine::CommandRegistry;

pub const MOD_ID: &str = "grammar-harper";

/// Grammar runs via `harper-ls` through the diagnostics LSP client (ADR 0022).
pub fn register(_registry: &mut CommandRegistry) {}

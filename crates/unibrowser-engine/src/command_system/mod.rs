mod context;
mod contribution_merge;
mod families;
mod layout;
mod provider;
mod resolver;
mod save_guard;

#[cfg(test)]
mod tests;

pub use context::{ResolutionContext, command_context_display_label, infer_work_context};
pub use contribution_merge::{
    ContributionMergeDiagnostics, LoadedModContribution, diagnose_contributions,
    merge_mod_contributions, merge_toolbar_layouts,
};
pub use families::core_tab_layout;
pub use layout::{LayoutStore, migrate_toolbars_json};
pub use provider::{ProviderRegistry, ProviderRuntime};
pub use resolver::CapabilityResolver;
pub use save_guard::save_commands_always_supported;

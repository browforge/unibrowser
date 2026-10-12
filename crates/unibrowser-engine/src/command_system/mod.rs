mod context;
mod contribution_merge;
mod families;
mod layout;
mod kind_host;
mod provider;
mod resolver;
mod save_guard;
mod shell_contributions;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod worked_context_contract;

pub use context::{
    ResolutionContext, command_context_display_label, command_context_for_manager_tab,
    infer_work_context,
};
pub use contribution_merge::{
    ContributionMergeDiagnostics, LoadedModContribution, diagnose_contributions,
    merge_mod_contributions, merge_toolbar_layouts,
};
pub use families::core_tab_layout;
pub use kind_host::KindHostCapabilityMatrix;
pub use layout::{LayoutStore, migrate_toolbars_json};
pub use provider::{ProviderRegistry, ProviderRuntime};
pub use resolver::CapabilityResolver;
pub use shell_contributions::{
    ResolvedPanelContribution, ResolvedServiceContribution, ResolvedStatusContribution,
    ResolvedViewContribution, ShellContributionSnapshot, merge_shell_contributions,
};
pub use save_guard::save_commands_always_supported;

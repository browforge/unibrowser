pub mod command;
pub mod command_manager;
pub mod document;
pub mod keybinding;
pub mod mode;
pub mod navigation;
pub mod palette;
pub mod tabs;
pub mod toolbar;
pub mod ui_command;

pub use command::{CommandRegistry, HandlerRegistration};
pub use document::{DocumentBackend, DocumentSession};
pub use mode::ModeController;
pub use keybinding::{
    KeyChord, KeybindingFile, KeybindingMap, COMMAND_PALETTE_TOGGLE, DEFAULT_PALETTE_CHORD,
    DEFAULT_PALETTE_CHORD_MACOS,
};
pub use palette::{PaletteEntry, search_commands};
pub use tabs::{Tab, TabManager};
pub use command_manager::{
    infer_context, load_command_manager_file, save_command_manager_file, CommandManagerState,
    CommandSurface, FloatingToolbarLayout, ManagerTabLayout, ResolvedUiCommand, ToolbarAnchor,
    ToolbarPhase, UiCommandSlot,
};
pub use toolbar::{ToolbarConfig, ToolbarController};
pub use ui_command::{CommandManagerFile, host_context_for_browse};

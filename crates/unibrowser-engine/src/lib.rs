pub mod command;
pub mod document;
pub mod mode;
pub mod navigation;
pub mod palette;
pub mod tabs;
pub mod toolbar;

pub use command::{CommandRegistry, HandlerRegistration};
pub use document::{DocumentBackend, DocumentSession};
pub use mode::ModeController;
pub use palette::{PaletteEntry, search_commands};
pub use tabs::{Tab, TabManager};
pub use toolbar::{ToolbarConfig, ToolbarController};

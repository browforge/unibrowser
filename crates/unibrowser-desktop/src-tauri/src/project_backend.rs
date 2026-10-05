use unibrowser_mod_api::{Capabilities, Mode, SelectionSnapshot};
use unibrowser_project::ProjectPackage;

pub const BACKEND_ID: &str = "unibrow-project";
pub const VIEWER_KIND: &str = "unibrow";

pub struct UnibrowBackend {
    pub package: ProjectPackage,
}

impl UnibrowBackend {
    pub fn open(package: ProjectPackage) -> Self {
        Self { package }
    }
}

impl unibrowser_engine::document::DocumentBackend for UnibrowBackend {
    fn backend_id(&self) -> &str {
        BACKEND_ID
    }
    fn doc_type(&self) -> &str {
        "application/vnd.unibrowser.project"
    }
    fn display_name(&self) -> &str {
        "Unibrowser project"
    }
    fn mode(&self) -> Mode {
        Mode::Browse
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities::default()
    }
    fn selection(&self) -> SelectionSnapshot {
        SelectionSnapshot::default()
    }
    fn increase_font_size(&mut self) -> Result<f32, String> {
        Err("Not applicable to project shell".into())
    }
    fn decrease_font_size(&mut self) -> Result<f32, String> {
        Err("Not applicable to project shell".into())
    }
    fn set_demo_text_selection(&mut self, _selected: bool) {}
    fn status_line(&self) -> String {
        format!(
            "Project “{}” · {} nodes · {}",
            self.package.manifest.title,
            self.package.manifest.nodes.len(),
            self.package.root.display()
        )
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[allow(dead_code)]
pub fn try_unibrow_backend(
    backend: &mut dyn unibrowser_engine::document::DocumentBackend,
) -> Option<&mut UnibrowBackend> {
    if backend.backend_id() != BACKEND_ID {
        return None;
    }
    backend.as_any_mut().downcast_mut::<UnibrowBackend>()
}

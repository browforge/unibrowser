use unibrowser_mod_api::{CommandContext, HostContext, Mode, WorkContext};

pub fn command_context_for_manager_tab(tab_id: &str, auto_context: CommandContext) -> CommandContext {
    match tab_id {
        "start" => CommandContext::Start,
        "text" => CommandContext::Text,
        "code" => CommandContext::Code,
        "image" => CommandContext::Image,
        "office" => CommandContext::Office,
        _ => auto_context,
    }
}

#[derive(Debug, Clone)]
pub struct ResolutionContext {
    pub work: WorkContext,
    pub command_context: CommandContext,
    pub host: HostContext,
    pub doc_path: Option<String>,
    pub revision: u64,
    pub markdown_inline_html: bool,
    pub host_format_id: String,
    pub selection_content_kinds: Vec<String>,
    pub selection_node_paths: Vec<String>,
    pub content_tree_revision: u64,
}

impl ResolutionContext {
    pub fn new(
        command_context: CommandContext,
        host: HostContext,
        doc_path: Option<String>,
        revision: u64,
    ) -> Self {
        let work = infer_work_context(command_context, host.mode, doc_path.as_deref());
        let path = doc_path.as_deref().unwrap_or("");
        let host_format_id = crate::content_tree::host_format_from_path(path);
        let tree = crate::content_tree::document_content_tree(&host.backend_id, path);
        let selection_node_paths = default_selection_paths(&tree, &work);
        let selection_content_kinds =
            crate::content_tree::selection_content_kinds(&tree, &selection_node_paths);
        let mut ctx = Self {
            work,
            command_context,
            host,
            doc_path,
            revision,
            markdown_inline_html: true,
            host_format_id,
            selection_content_kinds,
            selection_node_paths,
            content_tree_revision: 0,
        };
        if std::env::var("UNIBROWSER_SELECT_EMBEDDED_IMAGE")
            .map(|v| v == "1")
            .unwrap_or(false)
        {
            ctx = ctx.with_embedded_image_selection();
        }
        ctx
    }

    pub fn with_embedded_image_selection(mut self) -> Self {
        if let Some(node) = self
            .selection_node_paths
            .iter()
            .find(|p| p.contains("embedded-image"))
            .cloned()
        {
            self.selection_node_paths = vec![node];
        } else {
            let fallback = match self.host_format_id.as_str() {
                "md" | "markdown" => "embedded-image-md",
                _ => "embedded-image-1",
            };
            self.selection_node_paths = vec![fallback.into()];
        }
        let path = self.doc_path.as_deref().unwrap_or("");
        let tree = crate::content_tree::document_content_tree(&self.host.backend_id, path);
        self.selection_content_kinds =
            crate::content_tree::selection_content_kinds(&tree, &self.selection_node_paths);
        self
    }
}

fn default_selection_paths(
    tree: &unibrowser_mod_api::DocumentContentTree,
    work: &WorkContext,
) -> Vec<String> {
    if *work == WorkContext::Image {
        if let Some(node) = tree
            .nodes
            .iter()
            .find(|n| n.kind_id == unibrowser_mod_api::content_kind::ids::IMAGE)
        {
            return vec![node.node_id.clone()];
        }
    }
    vec!["root".into()]
}

/// User-visible Command Manager context label (follows active document work context).
pub fn command_context_display_label(
    command_context: CommandContext,
    mode: Mode,
    path: Option<&str>,
) -> &'static str {
    match infer_work_context(command_context, mode, path) {
        WorkContext::Start => "Start",
        WorkContext::Pdf => "PDF Annotation",
        WorkContext::Image => "Image",
        WorkContext::Audio => "Audio",
        WorkContext::Cad3d => "CAD / 3D",
        WorkContext::Project => "Project",
        WorkContext::CodeEditor => "Code",
        WorkContext::Markdown | WorkContext::PlainText => "Text",
        WorkContext::RichDocument => "Office",
        WorkContext::Browse => "Browse",
        WorkContext::Settings => "Settings",
        WorkContext::Mods => "Mods",
        WorkContext::Web => "Web",
    }
}

pub fn infer_work_context(
    command_context: CommandContext,
    mode: Mode,
    path: Option<&str>,
) -> WorkContext {
    let path = path.unwrap_or("");
    if path.eq_ignore_ascii_case("unibrowser://settings") {
        return WorkContext::Settings;
    }
    if path.eq_ignore_ascii_case("unibrowser://mods") {
        return WorkContext::Mods;
    }
    let ext = path.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    if ext == "unibrow" {
        return WorkContext::Project;
    }
    if ext == "pdf" {
        return WorkContext::Pdf;
    }
    if matches!(
        ext.as_str(),
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "bmp" | "ico" | "avif"
    ) {
        return WorkContext::Image;
    }
    if matches!(
        ext.as_str(),
        "aac" | "flac" | "m4a" | "mkv" | "mov" | "mp3" | "mp4" | "ogg" | "wav" | "webm"
    ) {
        return WorkContext::Audio;
    }
    // Command Manager Image tab (or auto context while an embedded image is selected)
    // must resolve image families even inside .md / .docx hosts.
    if command_context == CommandContext::Image {
        return WorkContext::Image;
    }
    if ext == "md" || ext == "markdown" {
        return WorkContext::Markdown;
    }
    if matches!(ext.as_str(), "dxf" | "step" | "stp" | "iges" | "igs") {
        return WorkContext::Cad3d;
    }
    match command_context {
        CommandContext::Start => WorkContext::Start,
        CommandContext::Browse => WorkContext::Browse,
        CommandContext::Web => WorkContext::Web,
        CommandContext::Image => WorkContext::Image,
        CommandContext::Code => WorkContext::CodeEditor,
        CommandContext::Office => {
            if is_office_path(path) {
                WorkContext::RichDocument
            } else {
                WorkContext::PlainText
            }
        }
        CommandContext::Text => {
            if is_code_path(path) {
                WorkContext::CodeEditor
            } else if is_office_path(path) {
                WorkContext::RichDocument
            } else {
                WorkContext::PlainText
            }
        }
        CommandContext::Audio => WorkContext::Audio,
    }
}

fn is_code_path(path: &str) -> bool {
    let ext = path.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    matches!(
        ext.as_str(),
        "rs" | "ts"
            | "tsx"
            | "js"
            | "jsx"
            | "py"
            | "go"
            | "java"
            | "c"
            | "cpp"
            | "h"
            | "hpp"
            | "cs"
            | "rb"
            | "swift"
            | "kt"
            | "sh"
            | "bash"
            | "zsh"
            | "json"
            | "toml"
            | "yaml"
            | "yml"
    )
}

fn is_office_path(path: &str) -> bool {
    let ext = path.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    matches!(
        ext.as_str(),
        "doc" | "docx" | "odt" | "xls" | "xlsx" | "ods" | "ppt" | "pptx" | "odp"
    )
}

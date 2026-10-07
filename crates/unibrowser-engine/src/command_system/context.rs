use unibrowser_mod_api::{CommandContext, HostContext, Mode, WorkContext};

#[derive(Debug, Clone)]
pub struct ResolutionContext {
    pub work: WorkContext,
    pub command_context: CommandContext,
    pub host: HostContext,
    pub doc_path: Option<String>,
    pub revision: u64,
    pub markdown_inline_html: bool,
}

impl ResolutionContext {
    pub fn new(
        command_context: CommandContext,
        host: HostContext,
        doc_path: Option<String>,
        revision: u64,
    ) -> Self {
        let work = infer_work_context(command_context, host.mode, doc_path.as_deref());
        Self {
            work,
            command_context,
            host,
            doc_path,
            revision,
            markdown_inline_html: true,
        }
    }
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
        WorkContext::Cad3d => "CAD / 3D",
        WorkContext::CodeEditor => "Code",
        WorkContext::Markdown | WorkContext::PlainText => "Text",
        WorkContext::RichDocument => "Office",
        WorkContext::Browse => "Browse",
        WorkContext::Settings => "Settings",
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
    let ext = path.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    if ext == "pdf" {
        return WorkContext::Pdf;
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
        CommandContext::Audio => WorkContext::PlainText,
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

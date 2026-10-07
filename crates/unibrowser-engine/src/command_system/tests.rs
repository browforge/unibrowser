#[cfg(test)]
mod resolver_tests {
    use unibrowser_mod_api::{
        commands, control_ids, CapabilityState, CommandContext, HostContext, Mode, WorkContext,
    };

    use crate::command_system::context::{
        command_context_display_label, infer_work_context, ResolutionContext,
    };
    use crate::command_system::resolver::CapabilityResolver;
    use crate::command_system::save_guard::{
        assert_save_independent, save_commands_always_supported,
    };

    fn ctx(work: WorkContext, path: Option<&str>) -> ResolutionContext {
        let command_context = CommandContext::Text;
        let host = HostContext {
            mode: Mode::Text,
            doc_type: String::new(),
            backend_id: String::new(),
            selection: Default::default(),
            capabilities: Default::default(),
        };
        let mut rc = ResolutionContext::new(command_context, host, path.map(String::from), 1);
        rc.work = work;
        rc
    }

    #[test]
    fn plain_text_strikethrough_unsupported_with_reason() {
        let resolver = CapabilityResolver::default();
        let cap = resolver.resolve_control(
            control_ids::TEXT_STRIKETHROUGH,
            commands::TEXT_STRIKETHROUGH,
            &ctx(WorkContext::PlainText, Some("notes.txt")),
        );
        assert!(matches!(cap, CapabilityState::Unsupported { .. }));
        assert!(cap.unsupported_reason().unwrap().contains("Plain text"));
    }

    #[test]
    fn markdown_subscript_emulated_with_note() {
        let resolver = CapabilityResolver::default();
        let cap = resolver.resolve_control(
            control_ids::TEXT_SUBSCRIPT,
            commands::TEXT_SUBSCRIPT,
            &ctx(WorkContext::Markdown, Some("readme.md")),
        );
        assert!(matches!(cap, CapabilityState::Emulated { .. }));
    }

    #[test]
    fn save_commands_never_blocked() {
        let resolver = CapabilityResolver::default();
        let cap = resolver.resolve_control(
            "unibrowser.control.save",
            commands::TEXT_SAVE,
            &ctx(WorkContext::PlainText, Some("notes.txt")),
        );
        assert_save_independent(&cap);
        assert!(save_commands_always_supported(commands::TEXT_SAVE));
    }

    #[test]
    fn stale_provider_response_rejected() {
        let mut resolver = CapabilityResolver::default();
        let accepted = resolver.accept_provider_response(2, 1, CapabilityState::Supported);
        assert!(accepted.is_none());
    }

    #[test]
    fn infer_markdown_path() {
        assert_eq!(
            infer_work_context(CommandContext::Text, Mode::Text, Some("x.md")),
            WorkContext::Markdown
        );
    }

    #[test]
    fn browse_context_uses_only_browse_controls() {
        let context = ctx(WorkContext::Browse, Some("/workspace"));
        let families =
            super::super::families::core_tab_layout(&context, &CapabilityResolver::new());
        let control_ids: Vec<&str> = families
            .iter()
            .flat_map(|family| family.groups.iter())
            .flat_map(|group| group.controls.iter())
            .map(|control| control.control_id.as_str())
            .collect();
        assert!(!control_ids.is_empty());
        assert!(control_ids
            .iter()
            .all(|id| id.starts_with("unibrowser.control.browse.")));
    }

    #[test]
    fn pdf_and_image_context_labels() {
        assert_eq!(
            command_context_display_label(CommandContext::Office, Mode::Document, Some("a.pdf")),
            "PDF Annotation"
        );
        assert_eq!(
            command_context_display_label(CommandContext::Image, Mode::Vector, Some("a.svg")),
            "Image"
        );
    }
}

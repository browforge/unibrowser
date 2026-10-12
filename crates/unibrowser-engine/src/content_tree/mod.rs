//! Embedded content tree builders for format backends (CMD-13).

use unibrowser_mod_api::content_kind::ids as ck;
use unibrowser_mod_api::content_tree::{ContentKindNode, DocumentContentTree};

pub fn host_format_from_path(path: &str) -> String {
    if path.eq_ignore_ascii_case("unibrowser://mods") {
        return "mods".into();
    }
    if path.eq_ignore_ascii_case("unibrowser://settings") {
        return "settings".into();
    }
    let ext = path.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    if ext.is_empty() {
        "unknown".into()
    } else {
        ext
    }
}

pub fn document_content_tree(backend_id: &str, path: &str) -> DocumentContentTree {
    let host = host_format_from_path(path);
    let mut nodes = Vec::new();
    let root_id = "root".to_string();
    let root_kind = root_content_kind(backend_id, &host);
    nodes.push(ContentKindNode {
        node_id: root_id.clone(),
        kind_id: root_kind,
        parent_id: None,
        host_format_id: host.clone(),
        blob_ref: None,
        label: Some("Document root".into()),
    });
    for embedded in embedded_nodes_for_backend(backend_id, &host) {
        nodes.push(embedded);
    }
    DocumentContentTree {
        root_host_format_id: host,
        nodes,
    }
}

fn root_content_kind(backend_id: &str, host: &str) -> String {
    match backend_id {
        "base-png" | "base-svg" => ck::IMAGE.into(),
        "base-pdf" => ck::PDF_PAGE.into(),
        "base-docx" => ck::TEXT_RICH.into(),
        "toy-markdown" => ck::TEXT_MARKDOWN.into(),
        "base-text" | "base-hex" | "base-js" => ck::TEXT_PLAIN.into(),
        "base-dxf" => ck::VECTOR_CAD.into(),
        "base-audio" => ck::AUDIO_CLIP.into(),
        "base-video" => ck::VIDEO_CLIP.into(),
        "base-csv" => ck::TABULAR.into(),
        "unibrow" => ck::PROJECT_GRAPH.into(),
        _ if host == "unibrow" => ck::PROJECT_GRAPH.into(),
        _ => ck::TEXT_PLAIN.into(),
    }
}

fn embedded_nodes_for_backend(backend_id: &str, host: &str) -> Vec<ContentKindNode> {
    match backend_id {
        "base-docx" => vec![embedded_image_node("embedded-image-1", "root", host)],
        "toy-markdown" => vec![embedded_image_node("embedded-image-md", "root", host)],
        "base-pdf" => vec![
            ContentKindNode {
                node_id: "page-1".into(),
                kind_id: ck::PDF_PAGE.into(),
                parent_id: Some("root".into()),
                host_format_id: host.to_string(),
                blob_ref: None,
                label: Some("Page 1".into()),
            },
            embedded_image_node("embedded-image-pdf", "page-1", host),
        ],
        "base-video" => (0..3)
            .map(|i| ContentKindNode {
                node_id: format!("frame-{}", i + 1),
                kind_id: ck::IMAGE.into(),
                parent_id: Some("root".into()),
                host_format_id: host.to_string(),
                blob_ref: Some(format!("frame:{}", i + 1)),
                label: Some(format!("Frame {}", i + 1)),
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn embedded_image_node(node_id: &str, parent: &str, host: &str) -> ContentKindNode {
    ContentKindNode {
        node_id: node_id.into(),
        kind_id: ck::IMAGE.into(),
        parent_id: Some(parent.into()),
        host_format_id: host.to_string(),
        blob_ref: Some("embedded:demo".into()),
        label: Some("Embedded image".into()),
    }
}

pub fn selection_content_kinds(
    tree: &DocumentContentTree,
    selected_paths: &[String],
) -> Vec<String> {
    if selected_paths.is_empty() {
        return tree.root_kind().into_iter().collect();
    }
    tree.kinds_for_paths(selected_paths)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn docx_fixture_exposes_embedded_image_kind() {
        let tree = document_content_tree("base-docx", "/tmp/sample.docx");
        let kinds = selection_content_kinds(&tree, &["embedded-image-1".into()]);
        assert_eq!(kinds, vec![ck::IMAGE]);
    }

    #[test]
    fn mixed_root_and_embedded_selection_is_deterministic() {
        let tree = document_content_tree("base-docx", "x.docx");
        let kinds = selection_content_kinds(
            &tree,
            &["root".into(), "embedded-image-1".into()],
        );
        assert_eq!(kinds.len(), 2);
        assert!(kinds.contains(&ck::TEXT_RICH.to_string()));
        assert!(kinds.contains(&ck::IMAGE.to_string()));
    }
}

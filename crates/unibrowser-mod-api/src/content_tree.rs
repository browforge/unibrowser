//! Document content tree nodes (CMD-13, ADR 0028).

use serde::{Deserialize, Serialize};

pub const CONTENT_TREE_SCHEMA_VERSION: &str = "1.0.0";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContentKindNode {
    pub node_id: String,
    pub kind_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    pub host_format_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DocumentContentTree {
    pub root_host_format_id: String,
    pub nodes: Vec<ContentKindNode>,
}

impl DocumentContentTree {
    pub fn node(&self, node_id: &str) -> Option<&ContentKindNode> {
        self.nodes.iter().find(|n| n.node_id == node_id)
    }

    pub fn kinds_for_paths(&self, paths: &[String]) -> Vec<String> {
        let mut kinds = Vec::new();
        for path in paths {
            if let Some(node) = self.node(path) {
                if !kinds.contains(&node.kind_id) {
                    kinds.push(node.kind_id.clone());
                }
            }
        }
        kinds
    }

    pub fn root_kind(&self) -> Option<String> {
        self.nodes
            .iter()
            .find(|n| n.parent_id.is_none())
            .map(|n| n.kind_id.clone())
    }
}

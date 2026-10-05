//! Per-document undo/redo sketch (text + SVG content strings).

use std::collections::HashMap;

const MAX_DEPTH: usize = 48;
const MAX_TRACKED_DOCS: usize = 48;

#[derive(Default)]
struct DocStacks {
    undo: Vec<String>,
    redo: Vec<String>,
}

#[derive(Default)]
pub struct UndoStacks {
    by_path: HashMap<String, DocStacks>,
    order: Vec<String>,
}

impl UndoStacks {
    fn touch_doc(&mut self, path: &str) {
        self.order.retain(|p| p != path);
        self.order.push(path.to_string());
        while self.order.len() > MAX_TRACKED_DOCS {
            let evict = self.order.remove(0);
            self.by_path.remove(&evict);
        }
    }

    pub fn record_change(&mut self, path: &str, before: String, after: &str) {
        if before == after {
            return;
        }
        self.touch_doc(path);
        let stacks = self.by_path.entry(path.to_string()).or_default();
        stacks.undo.push(before);
        if stacks.undo.len() > MAX_DEPTH {
            stacks.undo.remove(0);
        }
        if stacks.redo.len() > MAX_DEPTH {
            stacks.redo.remove(0);
        }
        stacks.redo.clear();
    }

    pub fn undo(&mut self, path: &str, current: &str) -> Option<String> {
        let stacks = self.by_path.get_mut(path)?;
        let prev = stacks.undo.pop()?;
        stacks.redo.push(current.to_string());
        Some(prev)
    }

    pub fn redo(&mut self, path: &str, current: &str) -> Option<String> {
        let stacks = self.by_path.get_mut(path)?;
        let next = stacks.redo.pop()?;
        stacks.undo.push(current.to_string());
        Some(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evicts_oldest_when_many_docs() {
        let mut stacks = UndoStacks::default();
        for i in 0..(MAX_TRACKED_DOCS + 5) {
            stacks.record_change(&format!("/doc-{i}"), "a".into(), "b");
        }
        assert!(stacks.by_path.len() <= MAX_TRACKED_DOCS);
    }

    #[test]
    fn undo_redo_roundtrip() {
        let mut stacks = UndoStacks::default();
        stacks.record_change("/a", "one".into(), "two");
        stacks.record_change("/a", "two".into(), "three");
        let u = stacks.undo("/a", "three").expect("undo");
        assert_eq!(u, "two");
        let r = stacks.redo("/a", "two").expect("redo");
        assert_eq!(r, "three");
    }
}

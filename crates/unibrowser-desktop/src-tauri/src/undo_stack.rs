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

    pub fn remove_path(&mut self, path: &str) {
        self.by_path.remove(path);
        self.order.retain(|p| p != path);
    }

    pub fn can_undo(&self, path: &str) -> bool {
        self.by_path
            .get(path)
            .map(|s| !s.undo.is_empty())
            .unwrap_or(false)
    }

    pub fn can_redo(&self, path: &str) -> bool {
        self.by_path
            .get(path)
            .map(|s| !s.redo.is_empty())
            .unwrap_or(false)
    }

    pub fn tracked_path_count(&self) -> usize {
        self.by_path.len()
    }
}

pub fn apply_wrap_transaction(
    stacks: &mut UndoStacks,
    path: &str,
    content: &str,
    start: usize,
    end: usize,
    before: &str,
    after: &str,
) -> Result<(String, usize, usize), String> {
    let start = start.min(content.len());
    let end = end.min(content.len()).max(start);
    if !content.is_char_boundary(start) || !content.is_char_boundary(end) {
        return Err("Selection is not on a text boundary".into());
    }
    let mut next = String::with_capacity(content.len() + before.len() + after.len());
    next.push_str(&content[..start]);
    next.push_str(before);
    next.push_str(&content[start..end]);
    next.push_str(after);
    next.push_str(&content[end..]);
    stacks.record_change(path, content.to_string(), &next);
    Ok((next, start + before.len(), end + before.len()))
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
        assert!(stacks.can_undo("/a"));
        assert!(!stacks.can_redo("/a"));
        let u = stacks.undo("/a", "three").expect("undo");
        assert_eq!(u, "two");
        assert!(stacks.can_redo("/a"));
        let r = stacks.redo("/a", "two").expect("redo");
        assert_eq!(r, "three");
    }

    #[test]
    fn host_wrap_transaction_is_one_undo_step() {
        let mut stacks = UndoStacks::default();
        let (next, start, end) =
            apply_wrap_transaction(&mut stacks, "/note.md", "H2O", 1, 2, "<sub>", "</sub>")
                .expect("wrap");
        assert_eq!(next, "H<sub>2</sub>O");
        assert_eq!((start, end), (6, 7));
        assert_eq!(stacks.undo("/note.md", &next).as_deref(), Some("H2O"));
        assert!(!stacks.can_undo("/note.md"));
    }
}

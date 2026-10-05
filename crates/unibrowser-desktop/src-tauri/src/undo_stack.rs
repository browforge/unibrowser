//! Per-document undo/redo sketch (text + SVG content strings).

use std::collections::HashMap;

const MAX_DEPTH: usize = 48;

#[derive(Default)]
struct DocStacks {
    undo: Vec<String>,
    redo: Vec<String>,
}

#[derive(Default)]
pub struct UndoStacks {
    by_path: HashMap<String, DocStacks>,
}

impl UndoStacks {
    pub fn record_change(&mut self, path: &str, before: String, after: &str) {
        if before == after {
            return;
        }
        let stacks = self.by_path.entry(path.to_string()).or_default();
        stacks.undo.push(before);
        if stacks.undo.len() > MAX_DEPTH {
            stacks.undo.remove(0);
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

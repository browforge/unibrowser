//! Per-window stack of recently closed tab locations (paths / URLs only).

use std::collections::VecDeque;

use unibrowser_mod_api::BrowserLocation;

const MAX_CLOSED: usize = 10;

#[derive(Default)]
pub struct ClosedTabStack {
    items: VecDeque<BrowserLocation>,
}

impl ClosedTabStack {
    pub fn push(&mut self, loc: BrowserLocation) {
        match &loc {
            BrowserLocation::File { .. } | BrowserLocation::Http { .. } => {
                self.items.push_front(loc);
                while self.items.len() > MAX_CLOSED {
                    self.items.pop_back();
                }
            }
            BrowserLocation::Internal { .. } => {}
        }
    }

    pub fn pop(&mut self) -> Option<BrowserLocation> {
        self.items.pop_front()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn stack_keeps_last_ten_paths_and_urls() {
        let mut s = ClosedTabStack::default();
        for i in 0..12 {
            s.push(BrowserLocation::File {
                path: PathBuf::from(format!("/tmp/{}", i)),
            });
        }
        assert_eq!(s.len(), MAX_CLOSED);
        let first = s.pop().expect("pop");
        assert_eq!(
            first,
            BrowserLocation::File {
                path: PathBuf::from("/tmp/11"),
            }
        );
    }

    #[test]
    fn internal_pages_are_not_recorded() {
        let mut s = ClosedTabStack::default();
        s.push(BrowserLocation::Internal {
            page: "home".into(),
        });
        assert_eq!(s.len(), 0);
    }
}

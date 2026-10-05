use unibrowser_mod_api::{CommandMeta, Enablement, HostContext};

use crate::command::CommandRegistry;

#[derive(Debug, Clone)]
pub struct PaletteEntry {
    pub command_id: String,
    pub title: String,
    pub score: i32,
    pub enablement: Enablement,
}

/// Subsequence fuzzy match score (higher is better).
pub fn fuzzy_score(query: &str, target: &str) -> i32 {
    let q = query.trim().to_lowercase();
    let t = target.to_lowercase();
    if q.is_empty() {
        return 1;
    }
    if t == q {
        return 500;
    }
    if t.starts_with(&q) {
        return 420 + (60 - t.len() as i32).max(0);
    }
    if t.contains(&q) {
        let idx = t.find(&q).unwrap_or(0);
        let at_word = idx == 0 || t.as_bytes().get(idx - 1) == Some(&b' ');
        return 150 + if at_word { 40 } else { 0 } + (80 - t.len() as i32).max(0);
    }
    let mut qi = 0;
    let qb: Vec<char> = q.chars().collect();
    let mut score = 0;
    let mut prev_match: Option<usize> = None;
    for (i, ch) in t.chars().enumerate() {
        if qi < qb.len() && ch == qb[qi] {
            score += 12 - (i as i32).min(10);
            if let Some(prev) = prev_match {
                if prev + 1 == i {
                    score += 8;
                }
            }
            if i == 0 || t.as_bytes().get(i - 1) == Some(&b' ') {
                score += 6;
            }
            prev_match = Some(i);
            qi += 1;
        }
    }
    if qi == qb.len() {
        score
    } else {
        -1
    }
}

pub fn search_commands(
    registry: &CommandRegistry,
    ctx: &HostContext,
    query: &str,
) -> Vec<PaletteEntry> {
    let mut entries: Vec<PaletteEntry> = registry
        .commands()
        .filter_map(|meta| {
            let best = best_field_score(query, meta);
            if best < 0 {
                return None;
            }
            Some(PaletteEntry {
                command_id: meta.id.to_string(),
                title: meta.title.to_string(),
                score: best,
                enablement: registry.enablement_ctx(meta.id, ctx),
            })
        })
        .collect();
    entries.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.title.cmp(&b.title)));
    entries
}

/// Boost palette score for recently executed commands (palette history).
pub fn apply_history_boost(command_id: &str, score: i32, history: &[String]) -> i32 {
    if let Some(pos) = history.iter().position(|id| id == command_id) {
        let weight = (history.len() - pos) as i32;
        return score + weight * 35;
    }
    score
}

fn best_field_score(query: &str, meta: &CommandMeta) -> i32 {
    let mut best = fuzzy_score(query, meta.title);
    best = best.max(fuzzy_score(query, meta.id));
    for alias in meta.aliases {
        best = best.max(fuzzy_score(query, alias));
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuzzy_subsequence() {
        assert!(fuzzy_score("inc", "Increase font size") > 0);
        assert_eq!(fuzzy_score("zzz", "Increase font size"), -1);
    }

    #[test]
    fn fuzzy_prefix_beats_subsequence() {
        let prefix = fuzzy_score("save", "Save document");
        let subseq = fuzzy_score("svd", "Save document");
        assert!(prefix > subseq);
    }

    #[test]
    fn history_boost_ranks_recent_higher() {
        let history = vec!["a".into(), "b".into(), "c".into()];
        let boosted_a = apply_history_boost("a", 100, &history);
        let boosted_c = apply_history_boost("c", 100, &history);
        assert!(boosted_a > boosted_c);
    }
}

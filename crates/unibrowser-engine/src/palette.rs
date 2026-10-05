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
    let q = query.to_lowercase();
    let t = target.to_lowercase();
    if q.is_empty() {
        return 1;
    }
    if t.contains(&q) {
        return 100 + (100 - t.len() as i32);
    }
    let mut qi = 0;
    let qb: Vec<char> = q.chars().collect();
    let mut score = 0;
    for (i, ch) in t.chars().enumerate() {
        if qi < qb.len() && ch == qb[qi] {
            score += 10 - (i as i32).min(9);
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
}

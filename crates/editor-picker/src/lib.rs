#![doc = r#"Generic fuzzy list providers, picker state, and preview surfaces."#]

use std::cmp::Reverse;

/// Human-readable summary of this crate's responsibility.
pub const ROLE: &str = "Generic fuzzy list providers, picker state, and preview surfaces.";

/// Returns the responsibility summary for this crate.
pub const fn role() -> &'static str {
    ROLE
}

/// One selectable entry in a generic picker list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickerItem {
    id: String,
    label: String,
    detail: String,
    preview: Option<String>,
}

impl PickerItem {
    /// Creates a new picker entry.
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        detail: impl Into<String>,
        preview: Option<impl Into<String>>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            detail: detail.into(),
            preview: preview.map(Into::into),
        }
    }

    /// Returns the stable item identifier.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the primary item label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns the secondary detail text.
    pub fn detail(&self) -> &str {
        &self.detail
    }

    /// Returns the preview content, when available.
    pub fn preview(&self) -> Option<&str> {
        self.preview.as_deref()
    }
}

/// Scored fuzzy-match result for a picker item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickerMatch {
    item: PickerItem,
    score: i64,
    matched_positions: Vec<usize>,
}

impl PickerMatch {
    /// Returns the matched item.
    pub fn item(&self) -> &PickerItem {
        &self.item
    }

    /// Returns the fuzzy match score.
    pub const fn score(&self) -> i64 {
        self.score
    }

    /// Returns the matched character positions in the item label.
    pub fn matched_positions(&self) -> &[usize] {
        &self.matched_positions
    }
}

/// Ordering strategy for picker results after fuzzy matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickerResultOrder {
    /// Rank matches by fuzzy-match score and then by label.
    ScoreThenLabel,
    /// Preserve the order of the underlying items.
    Source,
}

/// Mutable fuzzy picker session that tracks query, matches, and selection.
#[derive(Debug, Clone)]
pub struct PickerSession {
    title: String,
    items: Vec<PickerItem>,
    query: String,
    matches: Vec<PickerMatch>,
    selected_index: usize,
    result_limit: usize,
    result_order: PickerResultOrder,
}

impl PickerSession {
    /// Creates a new picker session and computes the initial match set.
    pub fn new(title: impl Into<String>, items: Vec<PickerItem>) -> Self {
        let mut session = Self {
            title: title.into(),
            items,
            query: String::new(),
            matches: Vec::new(),
            selected_index: 0,
            result_limit: usize::MAX,
            result_order: PickerResultOrder::ScoreThenLabel,
        };
        session.recompute_matches();
        session
    }

    /// Returns the picker title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the current fuzzy query.
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Returns the backing item count.
    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    /// Returns the current match count.
    pub fn match_count(&self) -> usize {
        self.matches.len()
    }

    /// Returns the current ordered match set.
    pub fn matches(&self) -> &[PickerMatch] {
        &self.matches
    }

    /// Returns the selected match, if one exists.
    pub fn selected(&self) -> Option<&PickerMatch> {
        self.matches.get(self.selected_index)
    }

    /// Limits the number of retained matches to protect large picker lists.
    pub fn with_result_limit(mut self, result_limit: usize) -> Self {
        self.result_limit = result_limit.max(1);
        self.recompute_matches();
        self
    }

    /// Configures how matches are ordered after fuzzy matching.
    pub fn with_result_order(mut self, result_order: PickerResultOrder) -> Self {
        self.result_order = result_order;
        self.recompute_matches();
        self
    }

    /// Preserves the order of the underlying items when computing matches.
    pub fn with_preserve_order(self) -> Self {
        self.with_result_order(PickerResultOrder::Source)
    }

    /// Updates the retained result limit and recomputes matches.
    pub fn set_result_limit(&mut self, result_limit: usize) {
        self.result_limit = result_limit.max(1);
        self.recompute_matches();
    }

    /// Updates the query and recomputes fuzzy matches.
    pub fn set_query(&mut self, query: impl Into<String>) {
        self.query = query.into();
        self.recompute_matches();
    }

    /// Replaces the picker items and recomputes matches using the current query.
    pub fn set_items(&mut self, items: Vec<PickerItem>) {
        self.items = items;
        self.recompute_matches();
    }

    /// Updates the selected match index when matches are available.
    pub fn set_selected_index(&mut self, index: usize) {
        if self.matches.is_empty() {
            self.selected_index = 0;
        } else {
            self.selected_index = index.min(self.matches.len() - 1);
        }
    }

    /// Moves the selection down by one entry.
    pub fn select_next(&mut self) {
        if self.matches.is_empty() {
            self.selected_index = 0;
            return;
        }

        self.selected_index = (self.selected_index + 1) % self.matches.len();
    }

    /// Moves the selection up by one entry.
    pub fn select_previous(&mut self) {
        if self.matches.is_empty() {
            self.selected_index = 0;
            return;
        }

        self.selected_index = self
            .selected_index
            .checked_sub(1)
            .unwrap_or(self.matches.len() - 1);
    }

    fn recompute_matches(&mut self) {
        self.matches = self
            .items
            .iter()
            .filter_map(|item| match_item(&self.query, item))
            .collect();
        match self.result_order {
            PickerResultOrder::ScoreThenLabel => {
                self.matches.sort_by_key(|matched| {
                    (Reverse(matched.score), matched.item.label().to_owned())
                });
            }
            PickerResultOrder::Source => {}
        }
        if self.matches.len() > self.result_limit {
            self.matches.truncate(self.result_limit);
        }

        if self.matches.is_empty() {
            self.selected_index = 0;
        } else {
            self.selected_index = self.selected_index.min(self.matches.len() - 1);
        }
    }
}

fn match_item(query: &str, item: &PickerItem) -> Option<PickerMatch> {
    let query_terms = query
        .split_whitespace()
        .filter(|term| !term.is_empty())
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>();
    if query_terms.is_empty() {
        return Some(PickerMatch {
            item: item.clone(),
            score: 0,
            matched_positions: Vec::new(),
        });
    }

    let label_chars: Vec<char> = item.label().chars().collect();
    let label_lower = item.label().to_ascii_lowercase();
    let mut score = 0i64;
    let mut matched_positions = Vec::new();

    for (term_index, term) in query_terms.iter().enumerate() {
        let matched = match_term(term, &label_chars, &label_lower)?;
        score += matched.score;
        if term_index == 0 && label_lower.starts_with(term) {
            score += 24;
        }
        matched_positions.extend(matched.matched_positions);
    }

    matched_positions.sort_unstable();
    matched_positions.dedup();
    score -= label_chars.len() as i64;

    Some(PickerMatch {
        item: item.clone(),
        score,
        matched_positions,
    })
}

struct TermMatch {
    score: i64,
    matched_positions: Vec<usize>,
}

fn match_term(term: &str, label_chars: &[char], label_lower: &str) -> Option<TermMatch> {
    let query_chars = term.chars().collect::<Vec<_>>();
    if query_chars.is_empty() {
        return None;
    }

    let mut matched_positions = Vec::with_capacity(query_chars.len());
    let mut query_index = 0usize;
    let mut score = 0i64;
    let mut previous_match = None;

    for (label_index, character) in label_chars.iter().copied().enumerate() {
        if query_index >= query_chars.len() {
            break;
        }

        if character.to_ascii_lowercase() != query_chars[query_index] {
            continue;
        }

        matched_positions.push(label_index);
        score += 10;

        if label_index == 0 {
            score += 18;
        }

        if let Some(previous) = previous_match
            && label_index == previous + 1
        {
            score += 14;
        }

        let boundary = label_index == 0
            || matches!(
                label_chars[label_index.saturating_sub(1)],
                '.' | ':' | '-' | '_' | '/' | '\\' | ' '
            );
        if boundary {
            score += 10;
        }

        previous_match = Some(label_index);
        query_index += 1;
    }

    if query_index != query_chars.len() {
        return None;
    }

    if label_lower.starts_with(term) {
        score += 12;
    }

    Some(TermMatch {
        score,
        matched_positions,
    })
}

#[cfg(test)]
mod tests {
    use super::{PickerItem, PickerResultOrder, PickerSession};

    fn item(id: &str, label: &str) -> PickerItem {
        PickerItem::new(id, label, label, None::<&str>)
    }

    #[test]
    fn empty_query_returns_all_items_in_sorted_order() {
        let session = PickerSession::new(
            "Commands",
            vec![item("b", "buffer.save"), item("a", "terminal.open")],
        );

        assert_eq!(session.match_count(), 2);
        assert_eq!(
            session
                .matches()
                .iter()
                .map(|matched| matched.item().label())
                .collect::<Vec<_>>(),
            vec!["buffer.save", "terminal.open"]
        );
    }

    #[test]
    fn source_order_preserves_input_order() {
        let session = PickerSession::new(
            "Commands",
            vec![item("z", "zeta"), item("a", "alpha"), item("m", "mu")],
        )
        .with_result_order(PickerResultOrder::Source);

        assert_eq!(
            session
                .matches()
                .iter()
                .map(|matched| matched.item().label())
                .collect::<Vec<_>>(),
            vec!["zeta", "alpha", "mu"]
        );
    }

    #[test]
    fn fuzzy_query_prefers_prefix_and_contiguous_matches() {
        let mut session = PickerSession::new(
            "Commands",
            vec![
                item("term", "terminal.open"),
                item("term-short", "term.open"),
                item("tabs", "workspace.open-scratch"),
            ],
        );
        session.set_query("term");

        let labels = session
            .matches()
            .iter()
            .map(|matched| matched.item().label())
            .collect::<Vec<_>>();
        assert_eq!(labels[0], "term.open");
        assert!(labels.contains(&"terminal.open"));
    }

    #[test]
    fn selection_wraps_across_match_list() {
        let mut session = PickerSession::new(
            "Commands",
            vec![item("a", "alpha"), item("b", "beta"), item("c", "gamma")],
        );

        assert_eq!(session.selected().map(|item| item.item().id()), Some("a"));
        session.select_previous();
        assert_eq!(session.selected().map(|item| item.item().id()), Some("c"));
        session.select_next();
        assert_eq!(session.selected().map(|item| item.item().id()), Some("a"));
    }

    #[test]
    fn result_limit_caps_large_match_sets() {
        let items = (0..128)
            .map(|index| item("cmd", &format!("command-{index:03}")))
            .collect::<Vec<_>>();
        let mut session = PickerSession::new("Commands", items).with_result_limit(16);
        session.set_query("command");

        assert_eq!(session.match_count(), 16);
    }

    #[test]
    fn whitespace_query_matches_multiple_terms() {
        let items = vec![
            item("pick-mode", "acp.pick-mode"),
            item("cycle-mode", "acp.cycle-mode"),
            item("workspace", "workspace.list-files"),
        ];
        let mut session = PickerSession::new("Commands", items);

        session.set_query("acp mode");

        assert_eq!(session.match_count(), 2);
        assert!(
            session
                .matches()
                .iter()
                .any(|matched| matched.item().label() == "acp.pick-mode")
        );
        assert!(
            session
                .matches()
                .iter()
                .any(|matched| matched.item().label() == "acp.cycle-mode")
        );
    }

    #[test]
    fn whitespace_query_requires_all_terms() {
        let items = vec![
            item("pick-mode", "acp.pick-mode"),
            item("workspace", "workspace.list-files"),
        ];
        let mut session = PickerSession::new("Commands", items);

        session.set_query("acp files");

        assert_eq!(session.match_count(), 0);
    }
}

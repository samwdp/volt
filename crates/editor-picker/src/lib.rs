#![doc = r#"Generic fuzzy list providers, picker state, and preview surfaces."#]

mod extra_dispatch;

pub use extra_dispatch::{
    PickerExportableRow, PickerExtraDispatch, PickerExtraKeybind, PickerOneShotContext,
    PickerSelectedRow, resolve_picker_extra,
};

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
    search_text: String,
    preview: Option<String>,
    fringe: Option<String>,
    divider: bool,
}

impl PickerItem {
    /// Creates a new picker entry.
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        detail: impl Into<String>,
        preview: Option<impl Into<String>>,
    ) -> Self {
        let label = label.into();
        Self {
            id: id.into(),
            search_text: label.clone(),
            label,
            detail: detail.into(),
            preview: preview.map(Into::into),
            fringe: None,
            divider: false,
        }
    }

    /// Creates a non-selectable horizontal divider row.
    pub fn divider(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: String::new(),
            detail: String::new(),
            search_text: String::new(),
            preview: None,
            fringe: None,
            divider: true,
        }
    }

    /// Returns whether this row is a non-selectable section divider.
    pub const fn is_divider(&self) -> bool {
        self.divider
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

    /// Overrides the text used for fuzzy matching without changing the visible label.
    pub fn with_search_text(mut self, search_text: impl Into<String>) -> Self {
        self.search_text = search_text.into();
        self
    }

    /// Returns the preview content, when available.
    pub fn preview(&self) -> Option<&str> {
        self.preview.as_deref()
    }

    /// Attaches optional left-fringe content such as an icon glyph.
    pub fn with_fringe(mut self, fringe: impl Into<String>) -> Self {
        self.fringe = Some(fringe.into());
        self
    }

    /// Returns the left-fringe content, when available.
    pub fn fringe(&self) -> Option<&str> {
        self.fringe.as_deref()
    }

    fn search_text(&self) -> &str {
        &self.search_text
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

    /// Replaces the title shown by picker renderers.
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
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
        self.select_first_match();
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

        self.selected_index = self.selectable_index(self.selected_index, 1);
    }

    /// Moves the selection up by one entry.
    pub fn select_previous(&mut self) {
        if self.matches.is_empty() {
            self.selected_index = 0;
            return;
        }

        self.selected_index = self.selectable_index(self.selected_index, -1);
    }

    fn selectable_index(&self, start: usize, direction: i32) -> usize {
        let len = self.matches.len();
        if len == 0 {
            return 0;
        }
        let mut index = start;
        for _ in 0..len {
            if direction > 0 {
                index = (index + 1) % len;
            } else {
                index = index.checked_sub(1).unwrap_or(len - 1);
            }
            if !self.matches[index].item().is_divider() {
                return index;
            }
        }
        start
    }

    fn ensure_selectable_selection(&mut self) {
        if self.matches.is_empty() {
            self.selected_index = 0;
            return;
        }
        if self.matches[self.selected_index].item().is_divider() {
            self.selected_index = self.selectable_index(self.selected_index, 1);
        }
    }

    fn select_first_match(&mut self) {
        if self.matches.is_empty() {
            self.selected_index = 0;
            return;
        }
        self.selected_index = 0;
        self.ensure_selectable_selection();
    }

    fn recompute_matches(&mut self) {
        let selected_id = self.matches.get(self.selected_index).and_then(|matched| {
            if matched.item().is_divider() {
                None
            } else {
                Some(matched.item().id().to_owned())
            }
        });
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
        } else if let Some(selected_id) = selected_id
            && let Some(index) = self
                .matches
                .iter()
                .position(|matched| matched.item().id() == selected_id)
        {
            self.selected_index = index;
        } else {
            self.selected_index = self.selected_index.min(self.matches.len() - 1);
            self.ensure_selectable_selection();
        }
    }
}

fn match_item(query: &str, item: &PickerItem) -> Option<PickerMatch> {
    if item.is_divider() {
        if query.split_whitespace().any(|term| !term.is_empty()) {
            return None;
        }
        return Some(PickerMatch {
            item: item.clone(),
            score: 0,
            matched_positions: Vec::new(),
        });
    }

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

    let search_chars: Vec<char> = item.search_text().chars().collect();
    let search_lower = item.search_text().to_ascii_lowercase();
    let mut score = 0i64;
    let mut matched_positions = Vec::new();

    for (term_index, term) in query_terms.iter().enumerate() {
        let matched = match_term(term, &search_chars, &search_lower)?;
        score += matched.score;
        score += best_contiguous_substring_bonus(term, &search_chars, &search_lower);
        if term_index == 0 && search_lower.starts_with(term) {
            score += 24;
        }
        matched_positions.extend(matched.matched_positions);
    }

    matched_positions.sort_unstable();
    matched_positions.dedup();
    score -= search_chars.len() as i64;

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

fn best_contiguous_substring_bonus(term: &str, label_chars: &[char], label_lower: &str) -> i64 {
    label_lower
        .match_indices(term)
        .map(|(start_byte, _)| {
            contiguous_substring_bonus(start_byte, term, label_chars, label_lower)
        })
        .max()
        .unwrap_or(0)
}

fn contiguous_substring_bonus(
    start_byte: usize,
    term: &str,
    label_chars: &[char],
    label_lower: &str,
) -> i64 {
    let start_index = label_lower[..start_byte].chars().count();
    let end_index = start_index + term.chars().count();
    let mut bonus = 48;

    if is_match_boundary(label_chars, start_index) {
        bonus += 24;
    }
    if is_match_end_boundary(label_chars, end_index) {
        bonus += 18;
    }

    bonus
}

fn is_match_boundary(label_chars: &[char], index: usize) -> bool {
    index == 0
        || label_chars
            .get(index.saturating_sub(1))
            .copied()
            .is_some_and(is_boundary_separator)
}

fn is_match_end_boundary(label_chars: &[char], index: usize) -> bool {
    index >= label_chars.len()
        || label_chars
            .get(index)
            .copied()
            .is_some_and(is_boundary_separator)
}

fn is_boundary_separator(character: char) -> bool {
    matches!(character, '.' | ':' | '-' | '_' | '/' | '\\' | ' ')
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

        let boundary = is_match_boundary(label_chars, label_index);
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
    fn contiguous_substring_beats_split_path_match() {
        let mut session = PickerSession::new(
            "Files",
            vec![
                item(
                    "asset-model",
                    "src/AssetFusion.Shared/Model/StaticData/AssetDefinition.cs",
                ),
                item(
                    "report-model",
                    "src/AssetFusion.Shared/Model/StaticData/ReportDefinition.cs",
                ),
                item(
                    "asset-service",
                    "src/AssetFusion.Shared/Services/Sql/AssetDefinitionService.cs",
                ),
                item(
                    "vehicle-model",
                    "src/AssetFusion.Shared/Model/StaticData/VehicleDefinition.cs",
                ),
            ],
        );
        session.set_query("assetdefinition");

        let labels = session
            .matches()
            .iter()
            .map(|matched| matched.item().label())
            .collect::<Vec<_>>();
        let first_non_asset = labels
            .iter()
            .position(|label| !label.contains("AssetDefinition"))
            .expect("expected weaker non-asset fuzzy matches");

        assert!(
            labels[..first_non_asset]
                .iter()
                .all(|label| label.contains("AssetDefinition"))
        );
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

    #[test]
    fn fringe_metadata_survives_matching() {
        let session = PickerSession::new(
            "Files",
            vec![item("alpha", "src/main.rs").with_fringe("icon")],
        );

        assert_eq!(
            session
                .selected()
                .and_then(|selected| selected.item().fringe()),
            Some("icon")
        );
    }

    #[test]
    fn divider_visible_with_empty_query_and_hidden_when_filtering() {
        let mut session = PickerSession::new(
            "Workspaces",
            vec![
                item("open", "default"),
                PickerItem::divider("divider"),
                item("project", "volt"),
            ],
        )
        .with_result_order(PickerResultOrder::Source);

        assert_eq!(session.match_count(), 3);
        assert!(
            session
                .matches()
                .iter()
                .any(|matched| matched.item().is_divider())
        );

        session.set_query("volt");
        assert_eq!(session.match_count(), 1);
        assert!(
            session
                .matches()
                .iter()
                .all(|matched| !matched.item().is_divider())
        );
    }

    #[test]
    fn query_change_resets_selection_to_first_match() {
        let mut session = PickerSession::new(
            "Commands",
            vec![item("a", "alpha"), item("b", "beta"), item("c", "gamma")],
        );

        session.select_next();
        session.select_next();
        assert_eq!(session.selected().map(|item| item.item().id()), Some("c"));

        session.set_query("a");
        assert_eq!(session.selected().map(|item| item.item().id()), Some("a"));

        session.set_query("b");
        assert_eq!(session.selected().map(|item| item.item().id()), Some("b"));
    }

    #[test]
    fn selection_skips_divider_rows() {
        let mut session = PickerSession::new(
            "Workspaces",
            vec![
                item("open", "default"),
                PickerItem::divider("divider"),
                item("project", "volt"),
            ],
        )
        .with_result_order(PickerResultOrder::Source);

        assert_eq!(
            session.selected().map(|item| item.item().id()),
            Some("open")
        );
        session.select_next();
        assert_eq!(
            session.selected().map(|item| item.item().id()),
            Some("project")
        );
        session.select_previous();
        assert_eq!(
            session.selected().map(|item| item.item().id()),
            Some("open")
        );
    }

    #[test]
    fn custom_search_text_matches_hidden_path_segments() {
        let mut session = PickerSession::new(
            "Files",
            vec![item("main", "main.rs").with_search_text("src/deep/nested/main.rs")],
        );

        session.set_query("deep nested");

        assert_eq!(session.match_count(), 1);
        assert_eq!(
            session.selected().map(|matched| matched.item().label()),
            Some("main.rs")
        );
    }
}

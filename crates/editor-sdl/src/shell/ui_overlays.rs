use super::*;

#[derive(Debug, Clone)]
pub(super) struct AutocompleteProviderSpec {
    pub(super) id: String,
    pub(super) label: String,
    pub(super) icon: String,
    pub(super) item_icon: String,
    pub(super) or_group: Option<String>,
    pub(super) buffer_kind: Option<String>,
    pub(super) items: Vec<editor_plugin_api::AutocompleteProviderItem>,
    pub(super) kind: AutocompleteProviderKind,
}

#[derive(Debug, Clone)]
pub(super) struct AutocompleteRegistry {
    pub(super) result_limit: usize,
    pub(super) providers: Vec<AutocompleteProviderSpec>,
}

impl AutocompleteRegistry {
    pub(super) fn from_user_config(user_library: &dyn UserLibrary) -> Self {
        let providers = user_library
            .autocomplete_providers()
            .into_iter()
            .filter_map(|provider| {
                let kind = if !provider.items.is_empty() {
                    AutocompleteProviderKind::Manual
                } else {
                    match provider.id.as_str() {
                        AUTOCOMPLETE_BUFFER_PROVIDER => AutocompleteProviderKind::Buffer,
                        AUTOCOMPLETE_LSP_PROVIDER => AutocompleteProviderKind::Lsp,
                        _ => return None,
                    }
                };
                Some(AutocompleteProviderSpec {
                    id: provider.id,
                    label: provider.label,
                    icon: provider.icon,
                    item_icon: provider.item_icon,
                    or_group: provider.or_group,
                    buffer_kind: provider.buffer_kind,
                    items: provider.items,
                    kind,
                })
            })
            .collect();
        Self {
            result_limit: user_library.autocomplete_result_limit().max(1),
            providers,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct AutocompleteQuery {
    pub(super) prefix: String,
    pub(super) token: String,
    pub(super) replace_range: TextRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct AutocompleteEntry {
    pub(super) provider_id: String,
    pub(super) provider_label: String,
    pub(super) provider_icon: String,
    pub(super) item_icon: String,
    pub(super) label: String,
    pub(super) replacement: String,
    pub(super) detail: Option<String>,
    pub(super) documentation: Option<String>,
}

#[derive(Debug, Clone)]
pub(super) struct AutocompleteOverlay {
    pub(super) buffer_id: BufferId,
    pub(super) buffer_revision: u64,
    pub(super) query: AutocompleteQuery,
    pub(super) entries: Vec<AutocompleteEntry>,
    pub(super) selected_index: usize,
    pub(super) loading: bool,
}

impl AutocompleteOverlay {
    pub(super) fn new(buffer_id: BufferId, buffer_revision: u64, query: AutocompleteQuery) -> Self {
        Self {
            buffer_id,
            buffer_revision,
            query,
            entries: Vec::new(),
            selected_index: 0,
            loading: true,
        }
    }

    pub(super) fn selected(&self) -> Option<&AutocompleteEntry> {
        self.entries.get(self.selected_index)
    }

    pub(super) fn entries(&self) -> &[AutocompleteEntry] {
        &self.entries
    }

    pub(super) fn is_visible(&self) -> bool {
        !self.loading && !self.entries.is_empty()
    }

    pub(super) fn mark_loading(&mut self, buffer_revision: u64, query: AutocompleteQuery) {
        self.buffer_revision = buffer_revision;
        self.query = query;
        self.loading = true;
    }

    pub(super) fn set_entries(&mut self, entries: Vec<AutocompleteEntry>) {
        let previous = self
            .selected()
            .map(|entry| (entry.provider_id.clone(), entry.replacement.clone()));
        self.entries = entries;
        self.loading = false;
        self.selected_index = previous
            .and_then(|(provider_id, replacement)| {
                self.entries.iter().position(|entry| {
                    entry.provider_id == provider_id && entry.replacement == replacement
                })
            })
            .unwrap_or(0);
        if self.entries.is_empty() {
            self.selected_index = 0;
        } else {
            self.selected_index = self.selected_index.min(self.entries.len() - 1);
        }
    }

    pub(super) fn select_next(&mut self) {
        if self.entries.is_empty() {
            self.selected_index = 0;
            return;
        }
        self.selected_index = (self.selected_index + 1) % self.entries.len();
    }

    pub(super) fn select_previous(&mut self) {
        if self.entries.is_empty() {
            self.selected_index = 0;
            return;
        }
        self.selected_index = self
            .selected_index
            .checked_sub(1)
            .unwrap_or(self.entries.len() - 1);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum HoverProviderKind {
    TestHover,
    Lsp,
    SignatureHelp,
    Diagnostics,
    Manual,
}

#[derive(Debug, Clone)]
pub(super) struct HoverProviderSpec {
    pub(super) label: String,
    pub(super) icon: String,
    pub(super) buffer_kind: Option<String>,
    pub(super) topics: Vec<editor_plugin_api::HoverProviderTopic>,
    pub(super) kind: HoverProviderKind,
}

#[derive(Debug, Clone)]
pub(super) struct HoverRegistry {
    pub(super) line_limit: usize,
    pub(super) providers: Vec<HoverProviderSpec>,
}

impl HoverRegistry {
    pub(super) fn from_user_config(user_library: &dyn UserLibrary) -> Self {
        let providers = user_library
            .hover_providers()
            .into_iter()
            .filter_map(|provider| {
                let kind = if !provider.topics.is_empty() {
                    HoverProviderKind::Manual
                } else {
                    match provider.id.as_str() {
                        HOVER_PROVIDER_TEST => HoverProviderKind::TestHover,
                        HOVER_PROVIDER_LSP => HoverProviderKind::Lsp,
                        HOVER_PROVIDER_SIGNATURE_HELP => HoverProviderKind::SignatureHelp,
                        HOVER_PROVIDER_DIAGNOSTICS => HoverProviderKind::Diagnostics,
                        _ => return None,
                    }
                };
                Some(HoverProviderSpec {
                    label: provider.label,
                    icon: provider.icon,
                    buffer_kind: provider.buffer_kind,
                    topics: provider.topics,
                    kind,
                })
            })
            .collect();
        Self {
            line_limit: user_library.hover_line_limit().max(1),
            providers,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct HoverProviderContent {
    pub(super) provider_label: String,
    pub(super) provider_icon: String,
    pub(super) lines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct HoverOverlay {
    pub(super) buffer_id: BufferId,
    pub(super) anchor: TextPoint,
    pub(super) token: String,
    pub(super) providers: Vec<HoverProviderContent>,
    pub(super) provider_index: usize,
    pub(super) scroll_offset: usize,
    pub(super) focused: bool,
    pub(super) line_limit: usize,
    pub(super) pending_g_prefix: bool,
    pub(super) count: Option<usize>,
}

impl HoverOverlay {
    pub(super) fn current_provider(&self) -> Option<&HoverProviderContent> {
        self.providers.get(self.provider_index)
    }

    pub(super) fn select_next_provider(&mut self) {
        if self.providers.is_empty() {
            self.provider_index = 0;
            return;
        }
        self.provider_index = (self.provider_index + 1) % self.providers.len();
        self.scroll_offset = 0;
        self.clear_navigation_state();
    }

    pub(super) fn select_previous_provider(&mut self) {
        if self.providers.is_empty() {
            self.provider_index = 0;
            return;
        }
        self.provider_index = self
            .provider_index
            .checked_sub(1)
            .unwrap_or(self.providers.len() - 1);
        self.scroll_offset = 0;
        self.clear_navigation_state();
    }

    pub(super) fn clear_navigation_state(&mut self) {
        self.pending_g_prefix = false;
        self.count = None;
    }

    pub(super) fn push_count_digit(&mut self, digit: usize) {
        let next = self
            .count
            .unwrap_or(0)
            .saturating_mul(10)
            .saturating_add(digit);
        self.count = Some(next);
        self.pending_g_prefix = false;
    }

    pub(super) fn take_count(&mut self) -> Option<usize> {
        self.count.take()
    }

    pub(super) fn take_count_or_one(&mut self) -> usize {
        self.take_count().unwrap_or(1).max(1)
    }

    pub(super) fn max_scroll_offset(&self) -> usize {
        self.current_provider()
            .map(|provider| provider.lines.len().saturating_sub(self.line_limit))
            .unwrap_or(0)
    }

    pub(super) fn page_scroll_lines(&self) -> usize {
        self.line_limit.max(1)
    }

    pub(super) fn half_page_scroll_lines(&self) -> usize {
        (self.line_limit / 2).max(1)
    }

    pub(super) fn scroll_by(&mut self, delta: i32) {
        let Some(provider) = self.current_provider() else {
            self.scroll_offset = 0;
            return;
        };
        let max_offset = provider.lines.len().saturating_sub(self.line_limit);
        if delta.is_negative() {
            self.scroll_offset = self
                .scroll_offset
                .saturating_sub(delta.unsigned_abs() as usize);
        } else {
            self.scroll_offset = (self.scroll_offset + delta as usize).min(max_offset);
        }
    }

    pub(super) fn scroll_to_start(&mut self) {
        self.scroll_offset = 0;
        self.clear_navigation_state();
    }

    pub(super) fn scroll_to_end(&mut self) {
        self.scroll_offset = self.max_scroll_offset();
        self.clear_navigation_state();
    }

    pub(super) fn scroll_to_line(&mut self, line_index: usize) {
        self.scroll_offset = line_index.min(self.max_scroll_offset());
        self.clear_navigation_state();
    }

    pub(super) fn visible_lines(&self) -> &[String] {
        let Some(provider) = self.current_provider() else {
            return &[];
        };
        let start = self.scroll_offset.min(provider.lines.len());
        let end = (start + self.line_limit).min(provider.lines.len());
        &provider.lines[start..end]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum NotificationSeverity {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct NotificationProgress {
    pub(super) percentage: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ShellNotification {
    pub(super) key: String,
    pub(super) severity: NotificationSeverity,
    pub(super) title: String,
    pub(super) body_lines: Vec<String>,
    pub(super) progress: Option<NotificationProgress>,
    pub(super) active: bool,
    pub(super) action: Option<NotificationAction>,
    pub(super) updated_at: Instant,
    pub(super) expires_at: Option<Instant>,
    pub(super) sequence: u64,
}

impl ShellNotification {
    pub(super) fn is_visible(&self, now: Instant) -> bool {
        self.active || self.expires_at.is_none_or(|expires_at| now <= expires_at)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct NotificationUpdate {
    pub(super) key: String,
    pub(super) severity: NotificationSeverity,
    pub(super) title: String,
    pub(super) body_lines: Vec<String>,
    pub(super) progress: Option<NotificationProgress>,
    pub(super) active: bool,
    pub(super) action: Option<NotificationAction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum NotificationAction {
    OpenAcpPermissionPicker { request_id: u64 },
}

#[derive(Debug, Clone, Default)]
pub(super) struct NotificationCenter {
    pub(super) revision: u64,
    pub(super) next_sequence: u64,
    pub(super) entries: Vec<ShellNotification>,
}

impl NotificationCenter {
    pub(super) fn apply(&mut self, update: NotificationUpdate, now: Instant) -> bool {
        let expires_at = (!update.active).then_some(now + NOTIFICATION_AUTO_DISMISS);
        let body_lines = update
            .body_lines
            .into_iter()
            .filter(|line| !line.trim().is_empty())
            .collect::<Vec<_>>();
        if let Some(existing) = self
            .entries
            .iter_mut()
            .find(|entry| entry.key == update.key)
        {
            existing.severity = update.severity;
            existing.title = update.title;
            existing.body_lines = body_lines;
            existing.progress = update.progress;
            existing.active = update.active;
            existing.action = update.action;
            existing.updated_at = now;
            existing.expires_at = expires_at;
            existing.sequence = self.next_sequence;
            self.next_sequence = self.next_sequence.saturating_add(1);
            self.revision = self.revision.saturating_add(1);
            self.trim();
            return true;
        }
        self.entries.push(ShellNotification {
            key: update.key,
            severity: update.severity,
            title: update.title,
            body_lines,
            progress: update.progress,
            active: update.active,
            action: update.action,
            updated_at: now,
            expires_at,
            sequence: self.next_sequence,
        });
        self.next_sequence = self.next_sequence.saturating_add(1);
        self.revision = self.revision.saturating_add(1);
        self.trim();
        true
    }

    pub(super) fn prune_expired(&mut self, now: Instant) -> bool {
        let before = self.entries.len();
        self.entries.retain(|entry| entry.is_visible(now));
        let changed = before != self.entries.len();
        if changed {
            self.revision = self.revision.saturating_add(1);
        }
        changed
    }

    pub(super) fn visible(&self, now: Instant) -> Vec<&ShellNotification> {
        let mut visible = self
            .entries
            .iter()
            .filter(|entry| entry.is_visible(now))
            .collect::<Vec<_>>();
        visible.sort_by(|left, right| {
            right
                .active
                .cmp(&left.active)
                .then_with(|| right.updated_at.cmp(&left.updated_at))
                .then_with(|| right.sequence.cmp(&left.sequence))
        });
        visible.truncate(NOTIFICATION_VISIBLE_LIMIT);
        visible
    }

    pub(super) fn next_deadline(&self, now: Instant) -> Option<Instant> {
        self.entries
            .iter()
            .filter_map(|entry| (!entry.active).then_some(entry.expires_at).flatten())
            .filter(|deadline| *deadline >= now)
            .min()
    }

    pub(super) fn revision(&self) -> u64 {
        self.revision
    }

    pub(super) fn trim(&mut self) {
        self.entries.sort_by(|left, right| {
            right
                .updated_at
                .cmp(&left.updated_at)
                .then_with(|| right.sequence.cmp(&left.sequence))
        });
        if self.entries.len() > NOTIFICATION_MAX_STORED {
            self.entries.truncate(NOTIFICATION_MAX_STORED);
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct NotificationOverlayLayout {
    pub(super) rect: Rect,
    pub(super) title: String,
    pub(super) body_lines: Vec<String>,
    pub(super) status_text: Option<String>,
    pub(super) severity: NotificationSeverity,
    pub(super) progress: Option<NotificationProgress>,
    pub(super) active: bool,
    pub(super) action: Option<NotificationAction>,
}

#[derive(Debug, Clone)]
pub(super) enum PickerMode {
    Static,
    VimSearch(VimSearchDirection),
    WorkspaceSearch { root: PathBuf },
}

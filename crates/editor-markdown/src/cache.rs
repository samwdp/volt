//! Cache Markdown Pretty plans by buffer revision (or ephemeral source hash).

use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use super::{MarkdownPrettyConfig, MarkdownPrettyPlan};

/// Cheap line/byte counts so Pretty Kill-switch can trip without flattening a rope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MarkdownPrettySourceStats {
    pub line_count: usize,
    pub byte_count: usize,
}

/// Buffer-backed vs hover/ACP ephemeral identity for a cached plan.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MarkdownPrettyCacheIdentity {
    Buffer {
        buffer_id: u64,
        revision: u64,
    },
    /// Hover/ACP strings have no buffer revision; key by source hash.
    Ephemeral {
        source_hash: u64,
    },
}

/// Cache key: identity + Pretty enable + Pretty Kill-switch + Forced Language id.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MarkdownPrettyCacheKey {
    pub identity: MarkdownPrettyCacheIdentity,
    pub enabled: bool,
    pub kill_switch_enabled: bool,
    pub kill_switch_max_lines: usize,
    pub kill_switch_max_bytes: usize,
    pub language_id: Option<String>,
}

impl MarkdownPrettyCacheKey {
    pub fn for_buffer(
        buffer_id: u64,
        revision: u64,
        enabled: bool,
        config: &MarkdownPrettyConfig,
        language_id: Option<String>,
    ) -> Self {
        Self {
            identity: MarkdownPrettyCacheIdentity::Buffer {
                buffer_id,
                revision,
            },
            enabled,
            kill_switch_enabled: config.kill_switch_enabled,
            kill_switch_max_lines: config.kill_switch_max_lines,
            kill_switch_max_bytes: config.kill_switch_max_bytes,
            language_id,
        }
    }

    pub fn for_ephemeral(
        source_hash: u64,
        enabled: bool,
        config: &MarkdownPrettyConfig,
        language_id: Option<String>,
    ) -> Self {
        Self {
            identity: MarkdownPrettyCacheIdentity::Ephemeral { source_hash },
            enabled,
            kill_switch_enabled: config.kill_switch_enabled,
            kill_switch_max_lines: config.kill_switch_max_lines,
            kill_switch_max_bytes: config.kill_switch_max_bytes,
            language_id,
        }
    }

    fn kill_switch_trips(&self, stats: MarkdownPrettySourceStats) -> bool {
        self.kill_switch_enabled
            && (stats.line_count > self.kill_switch_max_lines
                || stats.byte_count > self.kill_switch_max_bytes)
    }
}

/// Cached Markdown Pipeline plan, plus flattened source when the planner needed it.
#[derive(Debug, Clone)]
pub struct CachedMarkdownPretty {
    pub plan: Arc<MarkdownPrettyPlan>,
    pub source: Option<Arc<str>>,
}

/// Last-N Markdown Pretty plans. Capacity 1 is the buffer hot path (one revision).
#[derive(Debug, Clone)]
pub struct MarkdownPrettyPlanCache {
    capacity: usize,
    entries: BTreeMap<MarkdownPrettyCacheKey, CachedMarkdownPretty>,
}

impl Default for MarkdownPrettyPlanCache {
    fn default() -> Self {
        Self::with_capacity(1)
    }
}

impl MarkdownPrettyPlanCache {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            entries: BTreeMap::new(),
        }
    }

    pub fn get(&self, key: &MarkdownPrettyCacheKey) -> Option<&CachedMarkdownPretty> {
        self.entries.get(key)
    }

    /// Reuse a plan when `key` matches. On miss, skip `build` for disable / Pretty Kill-switch.
    pub fn get_or_insert_with(
        &mut self,
        key: MarkdownPrettyCacheKey,
        stats: MarkdownPrettySourceStats,
        build: impl FnOnce() -> (MarkdownPrettyPlan, Option<String>),
    ) -> CachedMarkdownPretty {
        if let Some(cached) = self.entries.get(&key) {
            return cached.clone();
        }

        let cached = if !key.enabled {
            CachedMarkdownPretty {
                plan: Arc::new(MarkdownPrettyPlan::default()),
                source: None,
            }
        } else if key.kill_switch_trips(stats) {
            CachedMarkdownPretty {
                plan: Arc::new(MarkdownPrettyPlan {
                    lines: BTreeMap::new(),
                    skipped_by_kill_switch: true,
                }),
                source: None,
            }
        } else {
            let (plan, source) = build();
            CachedMarkdownPretty {
                plan: Arc::new(plan),
                source: source.map(Arc::from),
            }
        };

        while self.entries.len() >= self.capacity {
            let Some(old_key) = self.entries.keys().next().cloned() else {
                break;
            };
            self.entries.remove(&old_key);
        }
        self.entries.insert(key, cached.clone());
        cached
    }

    pub fn last_plan(&self) -> Option<Arc<MarkdownPrettyPlan>> {
        self.entries
            .values()
            .next()
            .map(|cached| Arc::clone(&cached.plan))
    }
}

/// Hash used for hover/ACP ephemeral cache keys.
pub fn hash_markdown_source(text: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut hasher);
    hasher.finish()
}

/// Pretty Kill-switch using buffer stats (no rope flatten).
pub fn pretty_kill_switch_trips(
    config: &MarkdownPrettyConfig,
    stats: MarkdownPrettySourceStats,
) -> bool {
    config.kill_switch_enabled
        && (stats.line_count > config.kill_switch_max_lines
            || stats.byte_count > config.kill_switch_max_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MarkdownPrettyRequest, plan_markdown_pretty, pretty_display_line};
    use std::cell::Cell;
    use std::sync::Arc;

    fn cfg() -> MarkdownPrettyConfig {
        MarkdownPrettyConfig::default()
    }

    fn fixture_text() -> &'static str {
        "# Title\n- item\n"
    }

    fn stats_for(text: &str) -> MarkdownPrettySourceStats {
        MarkdownPrettySourceStats {
            line_count: text.lines().count(),
            byte_count: text.len(),
        }
    }

    fn build_plan(
        text: &str,
        config: &MarkdownPrettyConfig,
    ) -> (MarkdownPrettyPlan, Option<String>) {
        let request = MarkdownPrettyRequest {
            text,
            config,
            buffer_enabled: Some(true),
            buffer_path: None,
            workspace_root: None,
            cursor_line: None,
            visual_lines: None,
            visible_lines: None,
        };
        (plan_markdown_pretty(&request, None), Some(text.to_owned()))
    }

    #[test]
    fn cached_plan_reuses_arc_for_same_revision() {
        let mut cache = MarkdownPrettyPlanCache::default();
        let config = cfg();
        let text = fixture_text();
        let key = MarkdownPrettyCacheKey::for_buffer(7, 3, true, &config, Some("markdown".into()));
        let stats = stats_for(text);
        let builds = Cell::new(0);
        let first = cache.get_or_insert_with(key.clone(), stats, || {
            builds.set(builds.get() + 1);
            build_plan(text, &config)
        });
        let second = cache.get_or_insert_with(key, stats, || {
            builds.set(builds.get() + 1);
            build_plan(text, &config)
        });
        assert_eq!(builds.get(), 1);
        assert!(Arc::ptr_eq(&first.plan, &second.plan));
        let display_a = pretty_display_line(&first.plan, false, 0, "# Title");
        let display_b = pretty_display_line(&second.plan, false, 0, "# Title");
        assert_eq!(display_a, display_b);
        assert!(
            display_a.contains("Title") && !display_a.starts_with("# "),
            "pretty heading should conceal markers: {display_a:?}"
        );
    }

    #[test]
    fn cached_plan_rebuilds_when_revision_changes() {
        let mut cache = MarkdownPrettyPlanCache::default();
        let config = cfg();
        let text_a = "# Title\n";
        let text_b = "## Other\n";
        let key_a =
            MarkdownPrettyCacheKey::for_buffer(7, 3, true, &config, Some("markdown".into()));
        let key_b =
            MarkdownPrettyCacheKey::for_buffer(7, 4, true, &config, Some("markdown".into()));
        let builds = Cell::new(0);
        let first = cache.get_or_insert_with(key_a, stats_for(text_a), || {
            builds.set(builds.get() + 1);
            build_plan(text_a, &config)
        });
        let second = cache.get_or_insert_with(key_b, stats_for(text_b), || {
            builds.set(builds.get() + 1);
            build_plan(text_b, &config)
        });
        assert_eq!(builds.get(), 2);
        assert!(!Arc::ptr_eq(&first.plan, &second.plan));
        let display_a = pretty_display_line(&first.plan, false, 0, "# Title");
        let display_b = pretty_display_line(&second.plan, false, 0, "## Other");
        assert_ne!(display_a, display_b);
    }

    #[test]
    fn anti_conceal_overlay_reuses_cached_plan() {
        let mut cache = MarkdownPrettyPlanCache::default();
        let config = cfg();
        let text = fixture_text();
        let key = MarkdownPrettyCacheKey::for_buffer(7, 3, true, &config, Some("markdown".into()));
        let stats = stats_for(text);
        let builds = Cell::new(0);
        let cached = cache.get_or_insert_with(key.clone(), stats, || {
            builds.set(builds.get() + 1);
            build_plan(text, &config)
        });
        let source = "# Title";
        let pretty = pretty_display_line(&cached.plan, false, 0, source);
        let raw = pretty_display_line(&cached.plan, true, 0, source);
        assert_ne!(pretty, raw);
        assert_eq!(raw, source);
        let reused = cache.get_or_insert_with(key, stats, || {
            builds.set(builds.get() + 1);
            panic!("Anti-conceal must not rebuild the Pretty plan");
        });
        assert_eq!(builds.get(), 1);
        assert!(Arc::ptr_eq(&cached.plan, &reused.plan));
        let list_pretty = pretty_display_line(&cached.plan, false, 1, "- item");
        let list_raw = pretty_display_line(&cached.plan, true, 1, "- item");
        assert_eq!(list_raw, "- item");
        assert_ne!(list_pretty, list_raw);
    }

    #[test]
    fn kill_switch_sentinel_skips_build_and_reuses_plan() {
        let mut cache = MarkdownPrettyPlanCache::default();
        let mut config = cfg();
        config.kill_switch_enabled = true;
        config.kill_switch_max_lines = 0;
        let text = fixture_text();
        let key = MarkdownPrettyCacheKey::for_buffer(1, 1, true, &config, Some("markdown".into()));
        let stats = stats_for(text);
        let builds = Cell::new(0);
        let first = cache.get_or_insert_with(key.clone(), stats, || {
            builds.set(builds.get() + 1);
            build_plan(text, &config)
        });
        let second = cache.get_or_insert_with(key, stats, || {
            builds.set(builds.get() + 1);
            panic!("Pretty Kill-switch sentinel must not rebuild");
        });
        assert_eq!(builds.get(), 0);
        assert!(first.plan.skipped_by_kill_switch);
        assert!(first.plan.lines.is_empty());
        assert!(Arc::ptr_eq(&first.plan, &second.plan));
    }

    #[test]
    fn disabled_pretty_sentinel_skips_build() {
        let mut cache = MarkdownPrettyPlanCache::default();
        let config = cfg();
        let text = fixture_text();
        let key = MarkdownPrettyCacheKey::for_buffer(7, 3, false, &config, Some("markdown".into()));
        let builds = Cell::new(0);
        let cached = cache.get_or_insert_with(key, stats_for(text), || {
            builds.set(builds.get() + 1);
            build_plan(text, &config)
        });
        assert_eq!(builds.get(), 0);
        assert!(!cached.plan.skipped_by_kill_switch);
        assert!(cached.plan.lines.is_empty());
        assert_eq!(
            pretty_display_line(&cached.plan, false, 0, "# Title"),
            "# Title"
        );
    }
}

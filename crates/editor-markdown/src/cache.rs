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
#[path = "cache_tests.rs"]
mod tests;

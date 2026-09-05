
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

fn build_plan(text: &str, config: &MarkdownPrettyConfig) -> (MarkdownPrettyPlan, Option<String>) {
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
    let key_a = MarkdownPrettyCacheKey::for_buffer(7, 3, true, &config, Some("markdown".into()));
    let key_b = MarkdownPrettyCacheKey::for_buffer(7, 4, true, &config, Some("markdown".into()));
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

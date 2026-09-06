use super::*;

fn cfg() -> MarkdownPrettyConfig {
    MarkdownPrettyConfig::default()
}

#[test]
fn plans_headings_lists_hr_links_and_images() {
    let text = "\
# Title
- item
- [ ] todo
---
![alt](./pic.png)
See [docs](https://example.com).
";
    let config = cfg();
    let request = MarkdownPrettyRequest {
        text,
        config: &config,
        buffer_enabled: None,
        buffer_path: Some(Path::new("P:/proj/README.md")),
        workspace_root: Some(Path::new("P:/proj")),
        cursor_line: None,
        visual_lines: None,
        visible_lines: None,
    };
    let plan = plan_markdown_pretty(&request, None);
    assert!(plan.line(0).is_some_and(|l| !l.icons.is_empty()));
    assert!(plan.line(1).is_some_and(|l| !l.icons.is_empty()));
    assert!(plan.line(2).is_some_and(|l| {
        l.icons
            .iter()
            .any(|i| i.kind == "task_list_marker_unchecked")
    }));
    assert!(plan.line(3).is_some_and(|l| l.horizontal_rule));
    let image = plan.line(4).and_then(|l| l.image.clone());
    assert!(matches!(
        image.map(|i| i.destination),
        Some(ImageDestination::Local(path)) if path.ends_with("pic.png")
    ));
    assert!(plan.line(5).is_some_and(|l| {
        l.icons.iter().any(|i| i.kind == "inline_link") && !l.conceal.is_empty()
    }));
}

#[test]
fn ephemeral_plan_reuses_hash_key() {
    let config = cfg();
    let text = "# Title\n- item\n";
    let first = plan_markdown_pretty_ephemeral(text, &config, Some(true), None);
    let second = plan_markdown_pretty_ephemeral(text, &config, Some(true), None);
    assert!(std::sync::Arc::ptr_eq(&first, &second));
    let display = pretty_display_line(&first, false, 0, "# Title");
    assert!(
        display.contains("Title") && !display.starts_with("# "),
        "ephemeral Pretty should conceal heading markers: {display:?}"
    );
}

#[test]
fn kill_switch_skips_when_enabled() {
    let text = "# hi\n";
    let mut config = cfg();
    config.kill_switch_enabled = true;
    config.kill_switch_max_lines = 0;
    let request = MarkdownPrettyRequest {
        text,
        config: &config,
        buffer_enabled: None,
        buffer_path: None,
        workspace_root: None,
        cursor_line: None,
        visual_lines: None,
        visible_lines: None,
    };
    let plan = plan_markdown_pretty(&request, None);
    assert!(plan.skipped_by_kill_switch);
    assert!(plan.lines.is_empty());
}

#[test]
fn anti_conceal_detects_cursor_and_visual() {
    let plan = MarkdownPrettyPlan::default();
    let config = cfg();
    let request = MarkdownPrettyRequest {
        text: "a\nb\nc\n",
        config: &config,
        buffer_enabled: None,
        buffer_path: None,
        workspace_root: None,
        cursor_line: Some(1),
        visual_lines: Some(2..2),
        visible_lines: None,
    };
    assert!(plan.line_is_anti_concealed(&request, 1));
    assert!(plan.line_is_anti_concealed(&request, 2));
    assert!(!plan.line_is_anti_concealed(&request, 0));
}

#[test]
fn resolve_relative_dot_slash_from_buffer_dir() {
    let dest = resolve_image_destination(
        "./crates/volt/assets/banner.png",
        Some(Path::new("P:/volt/README.md")),
        None,
    );
    match dest {
        ImageDestination::Local(path) => {
            assert!(
                path.ends_with("crates/volt/assets/banner.png")
                    || path.ends_with(r"crates\volt\assets\banner.png"),
                "unexpected path: {}",
                path.display()
            );
        }
        other => panic!("expected local path, got {other:?}"),
    }
}

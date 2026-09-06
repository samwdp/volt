
use super::*;

#[test]
fn render_lines_respects_collapsed_state() {
    let mut child = Section::new("child", "Child");
    child.push_item(SectionItem::new("child item"));
    let mut root = Section::new("root", "Root");
    root.push_item(SectionItem::new("root item"));
    root.push_child(child);
    let tree = SectionTree::new(vec![root]);

    let lines = tree.render_lines(&SectionCollapseState::default());
    assert_eq!(lines.len(), 4);

    let mut state = SectionCollapseState::default();
    state.toggle("root");
    let collapsed = tree.render_lines(&state);
    assert_eq!(collapsed.len(), 1);
    assert!(matches!(
        collapsed[0].kind,
        SectionRenderLineKind::Header { .. }
    ));
}

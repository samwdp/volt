use super::{SplitAxis, SplitChild, SplitNode, layout_split_tree, pane_rects_with_weights};
use crate::{PixelRect, rect_tuple};

#[test]
fn nested_dashboard_tree_places_sidebar_and_editor_output() {
    let tree = SplitNode::columns(vec![
        SplitChild::node(
            SplitNode::rows(vec![SplitChild::leaf(0, 1, 0), SplitChild::leaf(1, 3, 0)]),
            1,
            0,
        ),
        SplitChild::node(
            SplitNode::rows(vec![SplitChild::leaf(2, 3, 0), SplitChild::leaf(3, 2, 0)]),
            3,
            0,
        ),
    ]);
    let leaves = layout_split_tree(PixelRect::new(0, 0, 400, 200), &tree, 0);
    let mut by_index = [None; 4];
    for (index, rect) in leaves {
        by_index[index] = Some(rect_tuple(rect));
    }
    assert_eq!(by_index[0], Some((0, 0, 100, 50)));
    assert_eq!(by_index[1], Some((0, 50, 100, 150)));
    assert_eq!(by_index[2], Some((100, 0, 300, 119)));
    assert_eq!(by_index[3], Some((100, 119, 300, 81)));
}

#[test]
fn weighted_columns_make_left_pane_smaller() {
    let rects = pane_rects_with_weights(400, 200, 2, SplitAxis::Columns, &[1, 3]);
    assert_eq!(rect_tuple(rects[0]), (0, 0, 100, 200));
    assert_eq!(rect_tuple(rects[1]), (100, 0, 300, 200));
}

#[test]
fn gap_is_inserted_between_siblings() {
    let tree = SplitNode::rows(vec![SplitChild::leaf(0, 1, 10), SplitChild::leaf(1, 1, 10)]);
    let leaves = layout_split_tree(PixelRect::new(8, 4, 100, 42), &tree, 8);
    assert_eq!(rect_tuple(leaves[0].1), (8, 4, 100, 17));
    assert_eq!(rect_tuple(leaves[1].1), (8, 29, 100, 17));
}

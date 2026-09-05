use super::PixelRect;

/// Split axis for a layout node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitAxis {
    /// Children stacked top to bottom.
    Rows,
    /// Children placed left to right.
    Columns,
}

/// One child of a split node: a leaf or a nested split.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SplitChildKind {
    /// Leaf identified by caller-assigned index.
    Leaf(usize),
    /// Nested split tree.
    Node(SplitNode),
}

/// Weighted child of a split.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitChild {
    /// Relative weight used to distribute leftover space. Clamped to at least 1.
    pub weight: u32,
    /// Minimum size along the parent axis, in pixels.
    pub min_px: u32,
    /// Leaf or nested node.
    pub kind: SplitChildKind,
}

impl SplitChild {
    /// Creates a weighted child.
    pub const fn new(weight: u32, min_px: u32, kind: SplitChildKind) -> Self {
        Self {
            weight,
            min_px,
            kind,
        }
    }

    /// Creates a leaf child.
    pub const fn leaf(index: usize, weight: u32, min_px: u32) -> Self {
        Self::new(weight, min_px, SplitChildKind::Leaf(index))
    }

    /// Creates a nested-split child.
    pub const fn node(node: SplitNode, weight: u32, min_px: u32) -> Self {
        Self::new(weight, min_px, SplitChildKind::Node(node))
    }
}

/// Recursive split describing a pane or section layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitNode {
    /// Axis along which children are placed.
    pub axis: SplitAxis,
    /// Direct children in visual order.
    pub children: Vec<SplitChild>,
}

impl SplitNode {
    /// Creates a split node.
    pub const fn new(axis: SplitAxis, children: Vec<SplitChild>) -> Self {
        Self { axis, children }
    }

    /// Creates a row (top-to-bottom) split.
    pub const fn rows(children: Vec<SplitChild>) -> Self {
        Self::new(SplitAxis::Rows, children)
    }

    /// Creates a column (left-to-right) split.
    pub const fn columns(children: Vec<SplitChild>) -> Self {
        Self::new(SplitAxis::Columns, children)
    }
}

/// Lays out `tree` inside `bounds`, returning `(leaf_index, rect)` pairs.
///
/// `gap` pixels are inserted between siblings. Extra space after minimums is
/// distributed by child weight; the last child absorbs rounding remainder.
pub fn layout_split_tree(bounds: PixelRect, tree: &SplitNode, gap: u32) -> Vec<(usize, PixelRect)> {
    let mut leaves = Vec::new();
    layout_node(bounds, tree, gap, &mut leaves);
    leaves
}

/// Splits `total` into `pane_count` rectangles along `axis` using `weights`.
///
/// When `weights` is shorter than `pane_count`, missing weights default to 1.
/// Extra weights are ignored. Golden-ratio callers can skip this helper.
pub fn pane_rects_with_weights(
    width: u32,
    content_height: u32,
    pane_count: usize,
    axis: SplitAxis,
    weights: &[u32],
) -> Vec<PixelRect> {
    if pane_count == 0 {
        return Vec::new();
    }
    if pane_count == 1 {
        return vec![PixelRect::new(0, 0, width, content_height)];
    }
    let children = (0..pane_count)
        .map(|index| SplitChild::leaf(index, weights.get(index).copied().unwrap_or(1), 1))
        .collect();
    let tree = SplitNode::new(axis, children);
    let bounds = PixelRect::new(0, 0, width, content_height);
    let mut rects = vec![PixelRect::new(0, 0, 0, 0); pane_count];
    for (index, rect) in layout_split_tree(bounds, &tree, 0) {
        if let Some(slot) = rects.get_mut(index) {
            *slot = rect;
        }
    }
    rects
}

fn layout_node(
    bounds: PixelRect,
    node: &SplitNode,
    gap: u32,
    leaves: &mut Vec<(usize, PixelRect)>,
) {
    if node.children.is_empty() {
        return;
    }
    if node.children.len() == 1 {
        layout_child(bounds, &node.children[0], gap, leaves);
        return;
    }

    let sizes = split_sizes(node, along_size(bounds, node.axis), gap);
    let mut offset = 0u32;
    for (child, size) in node.children.iter().zip(sizes.iter().copied()) {
        let child_bounds = child_rect(bounds, node.axis, offset, size);
        layout_child(child_bounds, child, gap, leaves);
        offset = offset.saturating_add(size).saturating_add(gap);
    }
}

fn layout_child(
    bounds: PixelRect,
    child: &SplitChild,
    gap: u32,
    leaves: &mut Vec<(usize, PixelRect)>,
) {
    match &child.kind {
        SplitChildKind::Leaf(index) => leaves.push((*index, bounds)),
        SplitChildKind::Node(node) => layout_node(bounds, node, gap, leaves),
    }
}

fn along_size(bounds: PixelRect, axis: SplitAxis) -> u32 {
    match axis {
        SplitAxis::Rows => bounds.height,
        SplitAxis::Columns => bounds.width,
    }
}

fn child_rect(bounds: PixelRect, axis: SplitAxis, offset: u32, size: u32) -> PixelRect {
    match axis {
        SplitAxis::Rows => PixelRect::new(
            bounds.x,
            bounds.y.saturating_add(offset as i32),
            bounds.width,
            size,
        ),
        SplitAxis::Columns => PixelRect::new(
            bounds.x.saturating_add(offset as i32),
            bounds.y,
            size,
            bounds.height,
        ),
    }
}

fn split_sizes(node: &SplitNode, available: u32, gap: u32) -> Vec<u32> {
    let count = node.children.len();
    let gap_total = gap.saturating_mul(count.saturating_sub(1) as u32);
    let usable = available.saturating_sub(gap_total);
    let mut sizes = node
        .children
        .iter()
        .map(|child| child.min_px.max(1))
        .collect::<Vec<_>>();
    let min_sum = sizes.iter().copied().sum::<u32>();
    if min_sum >= usable {
        shrink_to_fit(&mut sizes, usable);
        return sizes;
    }

    let extra = usable.saturating_sub(min_sum);
    let weights = node
        .children
        .iter()
        .map(|child| child.weight.max(1))
        .collect::<Vec<_>>();
    let weight_sum = weights.iter().copied().sum::<u32>().max(1);
    let mut distributed = 0u32;
    for (index, weight) in weights.iter().copied().enumerate() {
        if index + 1 == count {
            sizes[index] = sizes[index].saturating_add(extra.saturating_sub(distributed));
            break;
        }
        let share = extra.saturating_mul(weight) / weight_sum;
        sizes[index] = sizes[index].saturating_add(share);
        distributed = distributed.saturating_add(share);
    }
    sizes
}

fn shrink_to_fit(sizes: &mut [u32], usable: u32) {
    let mut used = sizes.iter().copied().sum::<u32>();
    while used > usable {
        let Some((index, _)) = sizes
            .iter()
            .enumerate()
            .filter(|(_, size)| **size > 1)
            .max_by_key(|(_, size)| **size)
        else {
            break;
        };
        sizes[index] = sizes[index].saturating_sub(1);
        used = used.saturating_sub(1);
    }
}

#[cfg(test)]
#[path = "split_layout_tests.rs"]
mod tests;

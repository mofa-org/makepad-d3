//! Partition layout for radial visualizations (sunburst, icicle)
//!
//! Implements D3's partition layout which assigns rectangular coordinates
//! that can be mapped to radial (sunburst) or rectangular (icicle) layouts.
//!
//! For each node, computes:
//! - x0, x1: Angular extent (for sunburst) or horizontal position (for icicle)
//! - y0, y1: Radial extent (for sunburst) or vertical position (for icicle)

use super::HierarchyNode;

/// A positioned node from the partition layout
#[derive(Clone, Debug)]
pub struct PartitionNode<T> {
    /// Original node data
    pub data: T,
    /// Node value (summed from descendants)
    pub value: f64,
    /// Start angle/position (0 to 2π for sunburst)
    pub x0: f64,
    /// End angle/position
    pub x1: f64,
    /// Inner radius/position
    pub y0: f64,
    /// Outer radius/position
    pub y1: f64,
    /// Depth in hierarchy (0 = root)
    pub depth: usize,
    /// Height from node to deepest leaf
    pub height: usize,
    /// Child nodes with partition coordinates
    pub children: Vec<PartitionNode<T>>,
    /// Color index (for sunburst: index of top-level ancestor)
    pub color_index: usize,
    /// Name/label for display
    pub name: String,
}

impl<T: Clone> PartitionNode<T> {
    /// Check if this is a leaf node
    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    /// Iterate over all nodes (pre-order)
    pub fn iter(&self) -> PartitionIter<'_, T> {
        PartitionIter { stack: vec![self] }
    }

    /// Get all nodes as a flat vector (excluding root if specified)
    pub fn descendants(&self, include_root: bool) -> Vec<&PartitionNode<T>> {
        let mut result: Vec<_> = self.iter().collect();
        if !include_root && !result.is_empty() {
            result.remove(0);
        }
        result
    }
}

/// Iterator for partition nodes
pub struct PartitionIter<'a, T> {
    stack: Vec<&'a PartitionNode<T>>,
}

impl<'a, T> Iterator for PartitionIter<'a, T> {
    type Item = &'a PartitionNode<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.stack.pop() {
            for child in node.children.iter().rev() {
                self.stack.push(child);
            }
            Some(node)
        } else {
            None
        }
    }
}

/// Partition layout algorithm
///
/// Computes a partition layout where each node's area is proportional to its value.
/// The layout divides space recursively: the root gets the entire area, and each
/// child gets a portion proportional to its value.
///
/// # Example
///
/// ```ignore
/// use makepad_d3::layout::hierarchy::{HierarchyNode, PartitionLayout};
/// use std::f64::consts::PI;
///
/// let mut root = HierarchyNode::new("root".to_string(), 0.0);
/// root.add_child(HierarchyNode::new("a".to_string(), 10.0));
/// root.add_child(HierarchyNode::new("b".to_string(), 20.0));
///
/// let layout = PartitionLayout::new()
///     .size(2.0 * PI, 400.0);  // angles 0 to 2π, radius 0 to 400
///
/// let result = layout.layout(&root);
/// // result.children[0].x0, x1 = angle range for "a"
/// // result.children[0].y0, y1 = radius range for "a"
/// ```
pub struct PartitionLayout {
    /// Size in x dimension (2π for full circle sunburst)
    pub x_size: f64,
    /// Size in y dimension (radius for sunburst)
    pub y_size: f64,
    /// Padding between siblings (in x units)
    pub padding: f64,
    /// Whether to round coordinates
    pub round: bool,
}

impl Default for PartitionLayout {
    fn default() -> Self {
        Self::new()
    }
}

impl PartitionLayout {
    /// Create a new partition layout with default settings
    pub fn new() -> Self {
        Self {
            x_size: std::f64::consts::PI * 2.0,
            y_size: 1.0,
            padding: 0.0,
            round: false,
        }
    }

    /// Set the layout size
    ///
    /// For sunburst: x_size = 2π (full circle), y_size = radius
    /// For icicle: x_size = width, y_size = height
    pub fn size(mut self, x: f64, y: f64) -> Self {
        self.x_size = x;
        self.y_size = y;
        self
    }

    /// Set padding between siblings
    pub fn padding(mut self, padding: f64) -> Self {
        self.padding = padding;
        self
    }

    /// Enable coordinate rounding
    pub fn round(mut self, round: bool) -> Self {
        self.round = round;
        self
    }

    /// Compute the partition layout
    pub fn layout<T: Clone + ToString>(&self, root: &HierarchyNode<T>) -> PartitionNode<T> {
        // First, sum values and compute depth/height
        let mut tree = root.clone();
        tree.sum();
        tree.each_before();
        tree.sort_by_value();

        let max_depth = self.find_max_depth(&tree);
        let total_value = tree.value;

        // Compute y band for each depth level
        let y_per_depth = if max_depth > 0 {
            self.y_size / (max_depth + 1) as f64
        } else {
            self.y_size
        };

        // Layout recursively
        self.layout_node(&tree, 0.0, self.x_size, 0, y_per_depth, total_value, 0)
    }

    fn find_max_depth<T>(&self, node: &HierarchyNode<T>) -> usize {
        if node.children.is_empty() {
            node.depth
        } else {
            node.children.iter().map(|c| self.find_max_depth(c)).max().unwrap_or(node.depth)
        }
    }

    fn layout_node<T: Clone + ToString>(
        &self,
        node: &HierarchyNode<T>,
        x0: f64,
        x1: f64,
        depth: usize,
        y_per_depth: f64,
        parent_value: f64,
        color_index: usize,
    ) -> PartitionNode<T> {
        let y0 = depth as f64 * y_per_depth;
        let y1 = y0 + y_per_depth;

        // Layout children
        let mut children = Vec::new();
        if !node.children.is_empty() && parent_value > 0.0 {
            let mut child_x = x0;
            let total_child_value = node.value;

            for (i, child) in node.children.iter().enumerate() {
                // Each child gets proportional angular span
                let child_span = if total_child_value > 0.0 {
                    (child.value / total_child_value) * (x1 - x0)
                } else {
                    0.0
                };

                // Determine color index: for depth 1, use child index; otherwise inherit
                let child_color_index = if depth == 0 {
                    i
                } else {
                    color_index
                };

                let child_node = self.layout_node(
                    child,
                    child_x,
                    child_x + child_span,
                    depth + 1,
                    y_per_depth,
                    child.value,
                    child_color_index,
                );

                child_x += child_span;
                children.push(child_node);
            }
        }

        PartitionNode {
            data: node.data.clone(),
            value: node.value,
            x0,
            x1,
            y0,
            y1,
            depth,
            height: node.height,
            children,
            color_index,
            name: node.data.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_partition_simple() {
        let mut root = HierarchyNode::new("root".to_string(), 0.0);
        root.add_child(HierarchyNode::new("a".to_string(), 10.0));
        root.add_child(HierarchyNode::new("b".to_string(), 20.0));

        let layout = PartitionLayout::new().size(2.0 * PI, 100.0);
        let result = layout.layout(&root);

        // Root should span full angle
        assert!((result.x0 - 0.0).abs() < 0.001);
        assert!((result.x1 - 2.0 * PI).abs() < 0.001);

        // Children should split angle proportionally (sorted by value, so b comes first)
        assert_eq!(result.children.len(), 2);
        // b has value 20, a has value 10, total 30
        // b gets 2/3 of 2π, a gets 1/3
        let b = &result.children[0];
        let a = &result.children[1];

        assert!((b.x1 - b.x0 - 2.0 * PI * 2.0 / 3.0).abs() < 0.001);
        assert!((a.x1 - a.x0 - 2.0 * PI * 1.0 / 3.0).abs() < 0.001);
    }

    #[test]
    fn test_partition_depth() {
        let mut root = HierarchyNode::new("root".to_string(), 0.0);
        let mut child = HierarchyNode::new("child".to_string(), 0.0);
        child.add_child(HierarchyNode::new("grandchild".to_string(), 10.0));
        root.add_child(child);

        let layout = PartitionLayout::new().size(2.0 * PI, 300.0);
        let result = layout.layout(&root);

        // Root at depth 0: y0=0, y1=100
        assert!((result.y0 - 0.0).abs() < 0.001);
        assert!((result.y1 - 100.0).abs() < 0.001);

        // Child at depth 1: y0=100, y1=200
        let child = &result.children[0];
        assert!((child.y0 - 100.0).abs() < 0.001);
        assert!((child.y1 - 200.0).abs() < 0.001);

        // Grandchild at depth 2: y0=200, y1=300
        let grandchild = &child.children[0];
        assert!((grandchild.y0 - 200.0).abs() < 0.001);
        assert!((grandchild.y1 - 300.0).abs() < 0.001);
    }
}

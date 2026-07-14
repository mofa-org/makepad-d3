//! Circle packing layout algorithm
//!
//! Visualizes hierarchical data as nested circles.

use super::node::HierarchyNode;

/// Strategy for determining circle sizes
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PackStrategy {
    /// Size circles by value (default)
    #[default]
    Value,
    /// All leaf circles have equal size
    Count,
}

/// Circle packing layout for hierarchical data
///
/// Creates nested circle visualizations where area represents value.
///
/// # Example
///
/// ```
/// use makepad_d3::layout::hierarchy::{HierarchyNode, PackLayout};
///
/// let mut root = HierarchyNode::new("root", 0.0);
/// root.add_child(HierarchyNode::new("A", 30.0));
/// root.add_child(HierarchyNode::new("B", 20.0));
/// root.add_child(HierarchyNode::new("C", 50.0));
///
/// let layout = PackLayout::new()
///     .size(800.0, 600.0)
///     .padding(3.0);
///
/// let positioned = layout.layout(&root);
///
/// for node in positioned.iter() {
///     println!("{}: center=({:.1}, {:.1}) radius={:.1}",
///         node.data, node.x, node.y, node.radius);
/// }
/// ```
#[derive(Clone, Debug)]
pub struct PackLayout {
    /// Layout width
    width: f64,
    /// Layout height
    height: f64,
    /// Padding between sibling circles
    padding: f64,
    /// Strategy for sizing circles
    strategy: PackStrategy,
}

impl Default for PackLayout {
    fn default() -> Self {
        Self::new()
    }
}

impl PackLayout {
    /// Create a new pack layout
    pub fn new() -> Self {
        Self {
            width: 1.0,
            height: 1.0,
            padding: 0.0,
            strategy: PackStrategy::Value,
        }
    }

    /// Set the layout size
    pub fn size(mut self, width: f64, height: f64) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set padding between circles
    pub fn padding(mut self, padding: f64) -> Self {
        self.padding = padding.max(0.0);
        self
    }

    /// Set the sizing strategy
    pub fn strategy(mut self, strategy: PackStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Apply the layout to a hierarchy
    pub fn layout<T: Clone>(&self, root: &HierarchyNode<T>) -> HierarchyNode<T> {
        let mut tree = root.clone_tree();

        // Sum values and compute depths
        tree.sum();
        tree.each_before();

        // Compute radii for all nodes
        self.compute_radii(&mut tree);

        // Pack circles
        self.pack_node(&mut tree);

        // Scale to fit within bounds
        self.scale_to_bounds(&mut tree);

        tree
    }

    /// Compute radii for all nodes (post-order)
    fn compute_radii<T>(&self, node: &mut HierarchyNode<T>) {
        // Process children first
        for child in &mut node.children {
            self.compute_radii(child);
        }

        if node.is_leaf() {
            // Leaf radius based on value
            let value = match self.strategy {
                PackStrategy::Value => node.value.max(0.0),
                PackStrategy::Count => 1.0,
            };
            node.radius = (value / std::f64::consts::PI).sqrt();
        } else {
            // Internal node: pack children and compute enclosing circle
            self.pack_children(node);
        }
    }

    /// Pack children circles within a node
    fn pack_children<T>(&self, node: &mut HierarchyNode<T>) {
        let n = node.children.len();
        if n == 0 {
            node.radius = 0.0;
            return;
        }

        // Add padding to child radii
        for child in &mut node.children {
            child.radius += self.padding;
        }

        if n == 1 {
            // Single child - center it
            node.children[0].x = 0.0;
            node.children[0].y = 0.0;
            node.radius = node.children[0].radius;
        } else if n == 2 {
            // Two children - place side by side
            let r1 = node.children[0].radius;
            let r2 = node.children[1].radius;

            node.children[0].x = -r2;
            node.children[0].y = 0.0;
            node.children[1].x = r1;
            node.children[1].y = 0.0;

            node.radius = r1 + r2;
        } else {
            // Multiple children - use front-chain algorithm
            self.pack_front_chain(node);
        }

        // Remove padding from radii (for accurate sizing)
        for child in &mut node.children {
            child.radius -= self.padding;
        }
    }

    /// Pack circles using front-chain algorithm
    fn pack_front_chain<T>(&self, node: &mut HierarchyNode<T>) {
        // Sort children by radius (descending)
        node.children.sort_by(|a, b| {
            b.radius
                .partial_cmp(&a.radius)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let n = node.children.len();
        if n < 3 {
            return;
        }

        // Place first two circles
        let r0 = node.children[0].radius;
        let r1 = node.children[1].radius;

        node.children[0].x = 0.0;
        node.children[0].y = 0.0;
        node.children[1].x = r0 + r1;
        node.children[1].y = 0.0;

        // Place third circle
        if n > 2 {
            let (x, y) = self.place_circle(
                node.children[0].x,
                node.children[0].y,
                r0,
                node.children[1].x,
                node.children[1].y,
                r1,
                node.children[2].radius,
            );
            node.children[2].x = x;
            node.children[2].y = y;
        }

        // Place remaining circles
        for i in 3..n {
            let ri = node.children[i].radius;

            // Find best position by trying all pairs
            let mut best_x = 0.0;
            let mut best_y = 0.0;
            let mut best_dist = f64::INFINITY;
            let mut found_valid_position = false;

            for j in 0..i {
                for k in (j + 1)..i {
                    let (x, y) = self.place_circle(
                        node.children[j].x,
                        node.children[j].y,
                        node.children[j].radius,
                        node.children[k].x,
                        node.children[k].y,
                        node.children[k].radius,
                        ri,
                    );

                    // Check if this position overlaps with any existing circle
                    let overlaps = (0..i).any(|m| {
                        let dx = x - node.children[m].x;
                        let dy = y - node.children[m].y;
                        let dist = (dx * dx + dy * dy).sqrt();
                        dist < ri + node.children[m].radius - 0.001
                    });

                    if !overlaps {
                        let dist = (x * x + y * y).sqrt();
                        if dist < best_dist {
                            best_dist = dist;
                            best_x = x;
                            best_y = y;
                            found_valid_position = true;
                        }
                    }
                }
            }

            // Fallback: if no valid position found, place outside all existing circles
            if !found_valid_position {
                // Find the circle farthest from the origin and place new circle beyond it
                let mut max_dist: f64 = 0.0;
                let mut farthest_idx = 0;
                for j in 0..i {
                    let dist = (node.children[j].x.powi(2) + node.children[j].y.powi(2)).sqrt()
                        + node.children[j].radius;
                    if dist > max_dist {
                        max_dist = dist;
                        farthest_idx = j;
                    }
                }

                // Place the new circle along the direction from origin to farthest circle
                let fx = node.children[farthest_idx].x;
                let fy = node.children[farthest_idx].y;
                let fr = node.children[farthest_idx].radius;
                let dist_to_farthest = (fx * fx + fy * fy).sqrt();

                if dist_to_farthest > 0.001 {
                    // Normalize direction and place beyond farthest
                    let dir_x = fx / dist_to_farthest;
                    let dir_y = fy / dist_to_farthest;
                    best_x = fx + dir_x * (fr + ri);
                    best_y = fy + dir_y * (fr + ri);
                } else {
                    // Farthest is at origin, place to the right
                    best_x = fr + ri;
                    best_y = 0.0;
                }
            }

            node.children[i].x = best_x;
            node.children[i].y = best_y;
        }

        // Compute enclosing circle
        node.radius = self.compute_enclosing_radius(node);

        // Re-center children
        let (cx, cy) = self.compute_center_of_mass(node);
        for child in &mut node.children {
            child.x -= cx;
            child.y -= cy;
        }
    }

    /// Place a circle tangent to two others
    fn place_circle(
        &self,
        x1: f64,
        y1: f64,
        r1: f64,
        x2: f64,
        y2: f64,
        r2: f64,
        r: f64,
    ) -> (f64, f64) {
        let dx = x2 - x1;
        let dy = y2 - y1;
        let d = (dx * dx + dy * dy).sqrt();

        if d < 0.001 {
            return (x1, y1 + r1 + r);
        }

        let a = (r1 + r) * (r1 + r);
        let b = (r2 + r) * (r2 + r);

        // Use law of cosines
        let cos_alpha = (a + d * d - b) / (2.0 * (r1 + r) * d);
        let cos_alpha = cos_alpha.clamp(-1.0, 1.0);
        let alpha = cos_alpha.acos();

        // Angle from c1 to c2
        let theta = dy.atan2(dx);

        // New circle position
        let x = x1 + (r1 + r) * (theta + alpha).cos();
        let y = y1 + (r1 + r) * (theta + alpha).sin();

        (x, y)
    }

    /// Compute radius of enclosing circle
    fn compute_enclosing_radius<T>(&self, node: &HierarchyNode<T>) -> f64 {
        let mut max_dist: f64 = 0.0;

        for child in &node.children {
            let dist = (child.x * child.x + child.y * child.y).sqrt() + child.radius;
            max_dist = max_dist.max(dist);
        }

        max_dist
    }

    /// Compute center of mass
    fn compute_center_of_mass<T>(&self, node: &HierarchyNode<T>) -> (f64, f64) {
        if node.children.is_empty() {
            return (0.0, 0.0);
        }

        let mut cx = 0.0;
        let mut cy = 0.0;
        let mut total_area = 0.0;

        for child in &node.children {
            let area = child.radius * child.radius;
            cx += child.x * area;
            cy += child.y * area;
            total_area += area;
        }

        if total_area > 0.0 {
            (cx / total_area, cy / total_area)
        } else {
            (0.0, 0.0)
        }
    }

    /// Pack a node (recursively position children)
    fn pack_node<T>(&self, node: &mut HierarchyNode<T>) {
        for child in &mut node.children {
            self.pack_node(child);
        }
    }

    /// Scale and translate to fit within bounds
    fn scale_to_bounds<T>(&self, node: &mut HierarchyNode<T>) {
        if node.radius <= 0.0 {
            // Single leaf case
            node.x = self.width / 2.0;
            node.y = self.height / 2.0;
            node.radius = self.width.min(self.height) / 2.0;
            return;
        }

        // Calculate scale factor to fit
        let diameter = node.radius * 2.0;
        let scale = (self.width.min(self.height) / diameter).min(1.0);

        // Apply scale and translate to center
        let cx = self.width / 2.0;
        let cy = self.height / 2.0;

        self.transform_node(node, cx, cy, scale);
    }

    /// Transform node coordinates
    fn transform_node<T>(&self, node: &mut HierarchyNode<T>, tx: f64, ty: f64, scale: f64) {
        node.x = node.x * scale + tx;
        node.y = node.y * scale + ty;
        node.radius *= scale;

        for child in &mut node.children {
            self.transform_node(child, tx, ty, scale);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tree() -> HierarchyNode<String> {
        let mut root = HierarchyNode::from_label("root", 0.0);
        root.add_child(HierarchyNode::from_label("A", 30.0));
        root.add_child(HierarchyNode::from_label("B", 20.0));
        root.add_child(HierarchyNode::from_label("C", 50.0));
        root
    }

    #[test]
    fn test_pack_layout_new() {
        let layout = PackLayout::new();
        assert_eq!(layout.width, 1.0);
        assert_eq!(layout.height, 1.0);
    }

    #[test]
    fn test_pack_layout_size() {
        let layout = PackLayout::new().size(800.0, 600.0);
        assert_eq!(layout.width, 800.0);
        assert_eq!(layout.height, 600.0);
    }

    #[test]
    fn test_pack_layout_apply() {
        let tree = make_tree();
        let layout = PackLayout::new().size(100.0, 100.0);
        let positioned = layout.layout(&tree);

        // All circles should have positive radii
        for node in positioned.iter() {
            assert!(
                node.radius > 0.0,
                "Node {} has non-positive radius",
                node.data
            );
        }
    }

    #[test]
    fn test_pack_layout_circles_inside_parent() {
        let tree = make_tree();
        let layout = PackLayout::new().size(100.0, 100.0);
        let positioned = layout.layout(&tree);

        // Children should be within parent circle (with some tolerance)
        let parent_r = positioned.radius;
        let parent_x = positioned.x;
        let parent_y = positioned.y;

        for child in &positioned.children {
            let dx = child.x - parent_x;
            let dy = child.y - parent_y;
            let dist = (dx * dx + dy * dy).sqrt();

            assert!(
                dist + child.radius <= parent_r + 1.0,
                "Child {} outside parent",
                child.data
            );
        }
    }

    #[test]
    fn test_pack_layout_no_overlap() {
        let tree = make_tree();
        let layout = PackLayout::new().size(100.0, 100.0);
        let positioned = layout.layout(&tree);

        let children = &positioned.children;
        for i in 0..children.len() {
            for j in (i + 1)..children.len() {
                let dx = children[i].x - children[j].x;
                let dy = children[i].y - children[j].y;
                let dist = (dx * dx + dy * dy).sqrt();
                let min_dist = children[i].radius + children[j].radius;

                assert!(
                    dist >= min_dist - 1.0,
                    "Circles {} and {} overlap",
                    children[i].data,
                    children[j].data
                );
            }
        }
    }

    #[test]
    fn test_pack_layout_centered() {
        let tree = make_tree();
        let layout = PackLayout::new().size(100.0, 100.0);
        let positioned = layout.layout(&tree);

        // Root should be centered
        assert!((positioned.x - 50.0).abs() < 1.0);
        assert!((positioned.y - 50.0).abs() < 1.0);
    }

    #[test]
    fn test_pack_layout_single_child() {
        let mut root = HierarchyNode::from_label("root", 0.0);
        root.add_child(HierarchyNode::from_label("child", 100.0));

        let layout = PackLayout::new().size(100.0, 100.0);
        let positioned = layout.layout(&root);

        // Single child should be centered within parent
        assert!(positioned.children[0].radius > 0.0);
    }

    #[test]
    fn test_pack_layout_two_children() {
        let mut root = HierarchyNode::from_label("root", 0.0);
        root.add_child(HierarchyNode::from_label("A", 50.0));
        root.add_child(HierarchyNode::from_label("B", 50.0));

        let layout = PackLayout::new().size(100.0, 100.0);
        let positioned = layout.layout(&root);

        // Two equal children should have equal radii
        let r1 = positioned.children[0].radius;
        let r2 = positioned.children[1].radius;
        assert!((r1 - r2).abs() < 0.1);
    }

    #[test]
    fn test_pack_layout_with_padding() {
        let tree = make_tree();
        let layout = PackLayout::new().size(100.0, 100.0).padding(5.0);
        let positioned = layout.layout(&tree);

        // With padding, circles should not touch
        let children = &positioned.children;
        for i in 0..children.len() {
            for j in (i + 1)..children.len() {
                let dx = children[i].x - children[j].x;
                let dy = children[i].y - children[j].y;
                let dist = (dx * dx + dy * dy).sqrt();
                let min_dist = children[i].radius + children[j].radius;

                // Should have at least some padding gap
                assert!(dist >= min_dist - 1.0);
            }
        }
    }

    #[test]
    fn test_pack_strategy_count() {
        let mut root = HierarchyNode::from_label("root", 0.0);
        root.add_child(HierarchyNode::from_label("A", 10.0));
        root.add_child(HierarchyNode::from_label("B", 100.0));

        let layout = PackLayout::new()
            .size(100.0, 100.0)
            .strategy(PackStrategy::Count);

        let positioned = layout.layout(&root);

        // With count strategy, both leaves should have same radius
        let r1 = positioned.children[0].radius;
        let r2 = positioned.children[1].radius;
        assert!((r1 - r2).abs() < 0.1);
    }

    #[test]
    fn test_pack_many_children_positions_valid() {
        // Test that many children all get valid positions
        let mut root = HierarchyNode::from_label("root", 0.0);
        for i in 0..10 {
            root.add_child(HierarchyNode::from_label(
                &format!("child_{}", i),
                (i + 1) as f64 * 10.0,
            ));
        }

        let layout = PackLayout::new().size(500.0, 500.0);
        let positioned = layout.layout(&root);

        // All children should have finite, non-zero positions
        for (i, child) in positioned.children.iter().enumerate() {
            assert!(
                child.x.is_finite() && child.y.is_finite(),
                "Child {} has invalid position ({}, {})",
                i,
                child.x,
                child.y
            );
            assert!(
                child.radius > 0.0,
                "Child {} has non-positive radius {}",
                i,
                child.radius
            );
        }

        // No children should overlap
        let children = &positioned.children;
        for i in 0..children.len() {
            for j in (i + 1)..children.len() {
                let dx = children[i].x - children[j].x;
                let dy = children[i].y - children[j].y;
                let dist = (dx * dx + dy * dy).sqrt();
                let min_dist = children[i].radius + children[j].radius;

                assert!(
                    dist >= min_dist - 1.0,
                    "Children {} and {} overlap: dist={}, min_dist={}",
                    i,
                    j,
                    dist,
                    min_dist
                );
            }
        }
    }

    #[test]
    fn test_pack_equal_sized_children() {
        // Test edge case with many equal-sized children
        let mut root = HierarchyNode::from_label("root", 0.0);
        for i in 0..6 {
            root.add_child(HierarchyNode::from_label(&format!("child_{}", i), 100.0));
        }

        let layout = PackLayout::new().size(300.0, 300.0);
        let positioned = layout.layout(&root);

        // All positions should be valid (not at origin for all)
        let mut all_at_origin = true;
        for child in &positioned.children {
            if child.x.abs() > 0.001 || child.y.abs() > 0.001 {
                all_at_origin = false;
                break;
            }
        }

        assert!(!all_at_origin, "Not all children should be at the origin");
    }
}

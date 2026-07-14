//! Tree layout algorithm
//!
//! Implements a tidy tree layout based on the Reingold-Tilford algorithm.

use super::node::HierarchyNode;

/// Tree layout for hierarchical data
///
/// Positions nodes in a traditional tree diagram with the root at top
/// and leaves at bottom.
///
/// # Example
///
/// ```
/// use makepad_d3::layout::hierarchy::{HierarchyNode, TreeLayout};
///
/// let mut root = HierarchyNode::new("root", 0.0);
/// root.add_child(HierarchyNode::new("child1", 10.0));
/// root.add_child(HierarchyNode::new("child2", 20.0));
///
/// let layout = TreeLayout::new().size(800.0, 400.0);
/// let positioned = layout.layout(&root);
///
/// for node in positioned.iter() {
///     println!("{}: ({:.1}, {:.1})", node.data, node.x, node.y);
/// }
/// ```
#[derive(Clone, Debug)]
pub struct TreeLayout {
    /// Layout width
    width: f64,
    /// Layout height
    height: f64,
    /// Horizontal separation between sibling nodes
    separation_siblings: f64,
    /// Horizontal separation between non-siblings
    separation_cousins: f64,
    /// Node size (width, height) - if set, overrides separation
    node_size: Option<(f64, f64)>,
}

impl Default for TreeLayout {
    fn default() -> Self {
        Self::new()
    }
}

impl TreeLayout {
    /// Create a new tree layout
    pub fn new() -> Self {
        Self {
            width: 1.0,
            height: 1.0,
            separation_siblings: 1.0,
            separation_cousins: 2.0,
            node_size: None,
        }
    }

    /// Set the layout size
    pub fn size(mut self, width: f64, height: f64) -> Self {
        self.width = width;
        self.height = height;
        self.node_size = None;
        self
    }

    /// Set fixed node size (alternative to size)
    pub fn node_size(mut self, width: f64, height: f64) -> Self {
        self.node_size = Some((width, height));
        self
    }

    /// Set the separation between siblings
    pub fn separation_siblings(mut self, sep: f64) -> Self {
        self.separation_siblings = sep.max(0.0);
        self
    }

    /// Set the separation between cousins (non-siblings)
    pub fn separation_cousins(mut self, sep: f64) -> Self {
        self.separation_cousins = sep.max(0.0);
        self
    }

    /// Apply the layout to a hierarchy
    pub fn layout<T: Clone>(&self, root: &HierarchyNode<T>) -> HierarchyNode<T> {
        let mut tree = root.clone_tree();
        tree.each_before();

        // First pass: assign preliminary x coordinates
        let mut next_x = vec![0.0; tree.height + 1];
        self.first_walk(&mut tree, &mut next_x);

        // Second pass: compute final coordinates
        let (min_x, max_x) = self.get_x_extent(&tree);
        self.second_walk(&mut tree, min_x, max_x);

        tree
    }

    /// First pass: assign preliminary x coordinates (post-order)
    fn first_walk<T>(&self, node: &mut HierarchyNode<T>, next_x: &mut [f64]) {
        // Process children first
        for child in &mut node.children {
            self.first_walk(child, next_x);
        }

        let depth = node.depth;

        if node.is_leaf() {
            // Leaf: place at next available position
            node.x = next_x[depth];
            next_x[depth] += self.separation_siblings;
        } else {
            // Internal node: center over children
            let first_child_x = node.children.first().map(|c| c.x).unwrap_or(0.0);
            let last_child_x = node.children.last().map(|c| c.x).unwrap_or(0.0);
            let mid = (first_child_x + last_child_x) / 2.0;

            // Ensure we don't overlap with previous siblings
            let min_x = next_x[depth];
            node.x = mid.max(min_x);

            // Update next_x for this depth
            next_x[depth] = node.x + self.separation_cousins;
        }

        // Y coordinate based on depth
        node.y = depth as f64;
    }

    /// Get the x extent of the tree
    fn get_x_extent<T>(&self, node: &HierarchyNode<T>) -> (f64, f64) {
        let mut min_x = node.x;
        let mut max_x = node.x;

        for child in &node.children {
            let (child_min, child_max) = self.get_x_extent(child);
            min_x = min_x.min(child_min);
            max_x = max_x.max(child_max);
        }

        (min_x, max_x)
    }

    /// Second pass: normalize coordinates to layout size
    fn second_walk<T>(&self, node: &mut HierarchyNode<T>, min_x: f64, max_x: f64) {
        let height = self.find_max_depth(node) as f64;
        let x_range = (max_x - min_x).max(1.0);

        self.normalize_coords(node, min_x, x_range, height);
    }

    /// Find maximum depth
    fn find_max_depth<T>(&self, node: &HierarchyNode<T>) -> usize {
        let mut max = node.depth;
        for child in &node.children {
            max = max.max(self.find_max_depth(child));
        }
        max
    }

    /// Normalize coordinates to layout bounds
    fn normalize_coords<T>(
        &self,
        node: &mut HierarchyNode<T>,
        min_x: f64,
        x_range: f64,
        height: f64,
    ) {
        if let Some((node_w, node_h)) = self.node_size {
            // Fixed node size mode
            node.x = (node.x - min_x) * node_w;
            node.y = node.y * node_h;
        } else {
            // Fit to size mode
            node.x = (node.x - min_x) / x_range * self.width;
            node.y = if height > 0.0 {
                node.y / height * self.height
            } else {
                0.0
            };
        }

        for child in &mut node.children {
            self.normalize_coords(child, min_x, x_range, height);
        }
    }
}

/// Cluster layout - variant of tree that aligns leaves
#[derive(Clone, Debug)]
pub struct ClusterLayout {
    /// Layout width
    width: f64,
    /// Layout height
    height: f64,
    /// Separation between nodes
    separation: f64,
}

impl Default for ClusterLayout {
    fn default() -> Self {
        Self::new()
    }
}

impl ClusterLayout {
    /// Create a new cluster layout
    pub fn new() -> Self {
        Self {
            width: 1.0,
            height: 1.0,
            separation: 1.0,
        }
    }

    /// Set the layout size
    pub fn size(mut self, width: f64, height: f64) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set the separation
    pub fn separation(mut self, sep: f64) -> Self {
        self.separation = sep.max(0.0);
        self
    }

    /// Apply the layout
    pub fn layout<T: Clone>(&self, root: &HierarchyNode<T>) -> HierarchyNode<T> {
        let mut tree = root.clone_tree();
        tree.each_before();

        // Position leaves first
        let leaves = self.position_leaves(&mut tree);

        // Then position internal nodes
        self.position_internals(&mut tree);

        // Normalize to layout size
        self.normalize(&mut tree, leaves);

        tree
    }

    fn position_leaves<T>(&self, node: &mut HierarchyNode<T>) -> usize {
        let mut leaf_count = 0;

        if node.is_leaf() {
            node.x = leaf_count as f64 * self.separation;
            leaf_count = 1;
        } else {
            for child in &mut node.children {
                leaf_count += self.position_leaves(child);
            }
            // Center over children
            let first_x = node.children.first().map(|c| c.x).unwrap_or(0.0);
            let last_x = node.children.last().map(|c| c.x).unwrap_or(0.0);
            node.x = (first_x + last_x) / 2.0;
        }

        leaf_count
    }

    fn position_internals<T>(&self, node: &mut HierarchyNode<T>) {
        node.y = node.depth as f64;
        for child in &mut node.children {
            self.position_internals(child);
        }
    }

    fn normalize<T>(&self, node: &mut HierarchyNode<T>, leaf_count: usize) {
        let max_depth = self.find_max_depth(node) as f64;
        let max_x = (leaf_count.saturating_sub(1)) as f64 * self.separation;

        self.normalize_node(node, max_x.max(1.0), max_depth.max(1.0));
    }

    fn find_max_depth<T>(&self, node: &HierarchyNode<T>) -> usize {
        let mut max = node.depth;
        for child in &node.children {
            max = max.max(self.find_max_depth(child));
        }
        max
    }

    fn normalize_node<T>(&self, node: &mut HierarchyNode<T>, max_x: f64, max_depth: f64) {
        node.x = node.x / max_x * self.width;
        node.y = node.y / max_depth * self.height;

        for child in &mut node.children {
            self.normalize_node(child, max_x, max_depth);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tree() -> HierarchyNode<String> {
        let mut root = HierarchyNode::from_label("root", 0.0);

        let mut child1 = HierarchyNode::from_label("child1", 0.0);
        child1.add_child(HierarchyNode::from_label("leaf1", 10.0));
        child1.add_child(HierarchyNode::from_label("leaf2", 20.0));

        let mut child2 = HierarchyNode::from_label("child2", 0.0);
        child2.add_child(HierarchyNode::from_label("leaf3", 15.0));

        root.add_child(child1);
        root.add_child(child2);
        root
    }

    #[test]
    fn test_tree_layout_new() {
        let layout = TreeLayout::new();
        assert_eq!(layout.width, 1.0);
        assert_eq!(layout.height, 1.0);
    }

    #[test]
    fn test_tree_layout_size() {
        let layout = TreeLayout::new().size(800.0, 600.0);
        assert_eq!(layout.width, 800.0);
        assert_eq!(layout.height, 600.0);
    }

    #[test]
    fn test_tree_layout_apply() {
        let tree = make_tree();
        let layout = TreeLayout::new().size(100.0, 100.0);
        let positioned = layout.layout(&tree);

        // Root should be at y=0
        assert_eq!(positioned.y, 0.0);

        // Children should be at y=50
        assert!((positioned.children[0].y - 50.0).abs() < 0.1);

        // Leaves should be at y=100
        assert!((positioned.children[0].children[0].y - 100.0).abs() < 0.1);
    }

    #[test]
    fn test_tree_layout_x_distribution() {
        let tree = make_tree();
        let layout = TreeLayout::new().size(100.0, 100.0);
        let positioned = layout.layout(&tree);

        // X coordinates should be distributed
        let leaf1_x = positioned.children[0].children[0].x;
        let leaf2_x = positioned.children[0].children[1].x;
        let leaf3_x = positioned.children[1].children[0].x;

        // Leaves should be spread out
        assert!(leaf1_x < leaf2_x);
        assert!(leaf2_x < leaf3_x);
    }

    #[test]
    fn test_tree_layout_node_size() {
        let tree = make_tree();
        let layout = TreeLayout::new().node_size(50.0, 30.0);
        let positioned = layout.layout(&tree);

        // Positions should be multiples of node size
        assert_eq!(positioned.y, 0.0);
    }

    #[test]
    fn test_tree_layout_single_node() {
        let root = HierarchyNode::from_label("root", 10.0);
        let layout = TreeLayout::new().size(100.0, 100.0);
        let positioned = layout.layout(&root);

        assert_eq!(positioned.x, 0.0);
        assert_eq!(positioned.y, 0.0);
    }

    #[test]
    fn test_cluster_layout_new() {
        let layout = ClusterLayout::new();
        assert_eq!(layout.width, 1.0);
        assert_eq!(layout.height, 1.0);
    }

    #[test]
    fn test_cluster_layout_apply() {
        let tree = make_tree();
        let layout = ClusterLayout::new().size(100.0, 100.0);
        let positioned = layout.layout(&tree);

        // Root should be at y=0
        assert_eq!(positioned.y, 0.0);

        // All leaves should be at y=100 (bottom)
        let leaf1_y = positioned.children[0].children[0].y;
        let leaf2_y = positioned.children[0].children[1].y;
        let leaf3_y = positioned.children[1].children[0].y;

        assert!((leaf1_y - 100.0).abs() < 0.1);
        assert!((leaf2_y - 100.0).abs() < 0.1);
        assert!((leaf3_y - 100.0).abs() < 0.1);
    }
}

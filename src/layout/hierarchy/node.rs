//! Hierarchy node structure for tree-based layouts

use serde::{Deserialize, Serialize};

/// A node in a hierarchical data structure
///
/// Used as input for tree, treemap, and pack layouts.
///
/// # Example
///
/// ```
/// use makepad_d3::layout::hierarchy::HierarchyNode;
///
/// // Create a hierarchy
/// let mut root = HierarchyNode::new("Company", 0.0);
///
/// let mut sales = HierarchyNode::new("Sales", 0.0);
/// sales.add_child(HierarchyNode::new("Domestic", 100.0));
/// sales.add_child(HierarchyNode::new("International", 150.0));
///
/// let mut engineering = HierarchyNode::new("Engineering", 0.0);
/// engineering.add_child(HierarchyNode::new("Frontend", 80.0));
/// engineering.add_child(HierarchyNode::new("Backend", 120.0));
///
/// root.add_child(sales);
/// root.add_child(engineering);
///
/// // Sum up values from leaves
/// root.sum();
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HierarchyNode<T = String> {
    /// Node data/identifier
    pub data: T,
    /// Node value (for sizing in layouts)
    pub value: f64,
    /// Child nodes
    pub children: Vec<HierarchyNode<T>>,
    /// Depth in the hierarchy (0 = root)
    pub depth: usize,
    /// Height from this node to deepest leaf
    pub height: usize,
    /// Parent index (for flat representations)
    pub parent: Option<usize>,

    // Layout coordinates (filled by layout algorithms)
    /// X position after layout
    pub x: f64,
    /// Y position after layout
    pub y: f64,
    /// Width (for treemap)
    pub width: f64,
    /// Height (for treemap)
    pub rect_height: f64,
    /// Radius (for pack layout)
    pub radius: f64,
}

impl<T> Default for HierarchyNode<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::new(T::default(), 0.0)
    }
}

impl<T> HierarchyNode<T> {
    /// Create a new hierarchy node
    pub fn new(data: T, value: f64) -> Self {
        Self {
            data,
            value,
            children: Vec::new(),
            depth: 0,
            height: 0,
            parent: None,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            rect_height: 0.0,
            radius: 0.0,
        }
    }

    /// Create a leaf node (with value, no children)
    pub fn leaf(data: T, value: f64) -> Self {
        Self::new(data, value)
    }

    /// Create a branch node (no value, will have children)
    pub fn branch(data: T) -> Self {
        Self::new(data, 0.0)
    }

    /// Add a child node
    pub fn add_child(&mut self, child: HierarchyNode<T>) {
        self.children.push(child);
    }

    /// Add multiple children
    pub fn add_children(&mut self, children: Vec<HierarchyNode<T>>) {
        self.children.extend(children);
    }

    /// Set children (replacing existing)
    pub fn with_children(mut self, children: Vec<HierarchyNode<T>>) -> Self {
        self.children = children;
        self
    }

    /// Check if this is a leaf node
    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    /// Check if this is the root node
    pub fn is_root(&self) -> bool {
        self.parent.is_none() && self.depth == 0
    }

    /// Get the number of children
    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    /// Count total descendants (including self)
    pub fn count(&self) -> usize {
        1 + self.children.iter().map(|c| c.count()).sum::<usize>()
    }

    /// Count leaf nodes
    pub fn leaf_count(&self) -> usize {
        if self.is_leaf() {
            1
        } else {
            self.children.iter().map(|c| c.leaf_count()).sum()
        }
    }

    /// Sum values from leaf nodes up the tree
    ///
    /// After calling this, internal nodes will have value = sum of children's values
    pub fn sum(&mut self) -> f64 {
        // Safe recursive implementation - tree depth is typically small
        if self.is_leaf() {
            self.value
        } else {
            self.value = self.children.iter_mut().map(|c| c.sum()).sum();
            self.value
        }
    }

    /// Calculate depth and height for all nodes
    pub fn each_before(&mut self) {
        self.compute_depth_height(0);
    }

    /// Internal: compute depth and height recursively
    fn compute_depth_height(&mut self, depth: usize) -> usize {
        self.depth = depth;
        if self.is_leaf() {
            self.height = 0;
        } else {
            self.height = self
                .children
                .iter_mut()
                .map(|c| c.compute_depth_height(depth + 1) + 1)
                .max()
                .unwrap_or(0);
        }
        self.height
    }

    /// Sort children by value (descending)
    pub fn sort_by_value(&mut self) {
        self.children.sort_by(|a, b| {
            b.value
                .partial_cmp(&a.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for child in &mut self.children {
            child.sort_by_value();
        }
    }

    /// Sort children by height (ascending)
    pub fn sort_by_height(&mut self) {
        self.children.sort_by_key(|c| c.height);
        for child in &mut self.children {
            child.sort_by_height();
        }
    }

    /// Get an iterator over all nodes (pre-order traversal)
    pub fn iter(&self) -> HierarchyIter<'_, T> {
        HierarchyIter { stack: vec![self] }
    }

    /// Get leaf nodes
    pub fn leaves(&self) -> Vec<&HierarchyNode<T>> {
        self.iter().filter(|n| n.is_leaf()).collect()
    }

    /// Get ancestors from this node to root
    pub fn path_to_root<'a>(&self, all_nodes: &'a [HierarchyNode<T>]) -> Vec<&'a HierarchyNode<T>> {
        let mut path = Vec::new();
        let mut current = self.parent;
        while let Some(idx) = current {
            if let Some(node) = all_nodes.get(idx) {
                path.push(node);
                current = node.parent;
            } else {
                break;
            }
        }
        path
    }

    /// Clone the tree structure (deep clone)
    pub fn clone_tree(&self) -> HierarchyNode<T>
    where
        T: Clone,
    {
        // Safe recursive implementation - tree depth is typically small
        HierarchyNode {
            data: self.data.clone(),
            value: self.value,
            children: self.children.iter().map(|c| c.clone_tree()).collect(),
            depth: self.depth,
            height: self.height,
            parent: self.parent,
            x: self.x,
            y: self.y,
            width: self.width,
            rect_height: self.rect_height,
            radius: self.radius,
        }
    }
}

impl HierarchyNode<String> {
    /// Create from a string label
    pub fn from_label(label: &str, value: f64) -> Self {
        Self::new(label.to_string(), value)
    }
}

/// Pre-order iterator over hierarchy nodes
pub struct HierarchyIter<'a, T> {
    stack: Vec<&'a HierarchyNode<T>>,
}

impl<'a, T> Iterator for HierarchyIter<'a, T> {
    type Item = &'a HierarchyNode<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.stack.pop() {
            // Add children in reverse order so they're visited left-to-right
            for child in node.children.iter().rev() {
                self.stack.push(child);
            }
            Some(node)
        } else {
            None
        }
    }
}

/// Positioned node output from layout algorithms
#[derive(Clone, Debug, PartialEq)]
pub struct PositionedNode<T> {
    /// Original node data
    pub data: T,
    /// Node value
    pub value: f64,
    /// X position
    pub x: f64,
    /// Y position
    pub y: f64,
    /// Width (for rectangular layouts)
    pub width: f64,
    /// Height (for rectangular layouts)
    pub height: f64,
    /// Radius (for circular layouts)
    pub radius: f64,
    /// Depth in hierarchy
    pub depth: usize,
    /// Is leaf node
    pub is_leaf: bool,
}

impl<T> PositionedNode<T> {
    /// Create from a hierarchy node
    pub fn from_node(node: &HierarchyNode<T>) -> Self
    where
        T: Clone,
    {
        PositionedNode {
            data: node.data.clone(),
            value: node.value,
            x: node.x,
            y: node.y,
            width: node.width,
            height: node.rect_height,
            radius: node.radius,
            depth: node.depth,
            is_leaf: node.is_leaf(),
        }
    }

    /// Check if point is inside this node's bounds
    pub fn contains(&self, x: f64, y: f64) -> bool {
        if self.radius > 0.0 {
            // Circle check
            let dx = x - self.x;
            let dy = y - self.y;
            dx * dx + dy * dy <= self.radius * self.radius
        } else {
            // Rectangle check
            x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
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
        let child2 = HierarchyNode::from_label("child2", 30.0);
        root.add_child(child1);
        root.add_child(child2);
        root
    }

    #[test]
    fn test_hierarchy_node_new() {
        let node = HierarchyNode::new("test", 42.0);
        assert_eq!(node.data, "test");
        assert_eq!(node.value, 42.0);
        assert!(node.is_leaf());
    }

    #[test]
    fn test_hierarchy_node_add_child() {
        let mut parent = HierarchyNode::new("parent", 0.0);
        parent.add_child(HierarchyNode::new("child", 10.0));

        assert_eq!(parent.child_count(), 1);
        assert!(!parent.is_leaf());
    }

    #[test]
    fn test_hierarchy_node_count() {
        let tree = make_tree();
        assert_eq!(tree.count(), 5); // root + child1 + leaf1 + leaf2 + child2
    }

    #[test]
    fn test_hierarchy_node_leaf_count() {
        let tree = make_tree();
        assert_eq!(tree.leaf_count(), 3); // leaf1, leaf2, child2
    }

    #[test]
    fn test_hierarchy_node_sum() {
        let mut tree = make_tree();
        let total = tree.sum();

        assert_eq!(total, 60.0); // 10 + 20 + 30
        assert_eq!(tree.value, 60.0);
        assert_eq!(tree.children[0].value, 30.0); // child1 = leaf1 + leaf2
    }

    #[test]
    fn test_hierarchy_node_depth_height() {
        let mut tree = make_tree();
        tree.each_before();

        assert_eq!(tree.depth, 0);
        assert_eq!(tree.height, 2);
        assert_eq!(tree.children[0].depth, 1);
        assert_eq!(tree.children[0].height, 1);
        assert_eq!(tree.children[0].children[0].depth, 2);
        assert_eq!(tree.children[0].children[0].height, 0);
    }

    #[test]
    fn test_hierarchy_node_sort_by_value() {
        let mut tree = make_tree();
        tree.sum();
        tree.sort_by_value();

        // child2 (30) should come before child1 (30 = 10 + 20)
        // Actually both are 30, so order may vary
        assert!(tree.children[0].value >= tree.children[1].value);
    }

    #[test]
    fn test_hierarchy_node_iter() {
        let tree = make_tree();
        let nodes: Vec<_> = tree.iter().collect();

        assert_eq!(nodes.len(), 5);
        assert_eq!(nodes[0].data, "root");
    }

    #[test]
    fn test_hierarchy_node_leaves() {
        let tree = make_tree();
        let leaves = tree.leaves();

        assert_eq!(leaves.len(), 3);
    }

    #[test]
    fn test_positioned_node_contains_rect() {
        let node = PositionedNode {
            data: "test",
            value: 10.0,
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
            radius: 0.0,
            depth: 0,
            is_leaf: true,
        };

        assert!(node.contains(50.0, 25.0));
        assert!(!node.contains(150.0, 25.0));
    }

    #[test]
    fn test_positioned_node_contains_circle() {
        let node = PositionedNode {
            data: "test",
            value: 10.0,
            x: 50.0,
            y: 50.0,
            width: 0.0,
            height: 0.0,
            radius: 25.0,
            depth: 0,
            is_leaf: true,
        };

        assert!(node.contains(50.0, 50.0)); // center
        assert!(node.contains(60.0, 50.0)); // within radius
        assert!(!node.contains(100.0, 50.0)); // outside
    }
}

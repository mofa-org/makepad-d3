//! Hierarchical layout algorithms
//!
//! Layouts for tree-structured data including trees, treemaps, and circle packing.
//!
//! # Example
//!
//! ```
//! use makepad_d3::layout::hierarchy::{HierarchyNode, TreeLayout};
//!
//! // Build a hierarchy
//! let mut root = HierarchyNode::new("root", 0.0);
//! root.add_child(HierarchyNode::new("child1", 10.0));
//! root.add_child(HierarchyNode::new("child2", 20.0));
//!
//! // Apply tree layout
//! let layout = TreeLayout::new().size(800.0, 600.0);
//! let positioned = layout.layout(&root);
//! ```

mod node;
mod tree;
mod treemap;
mod pack;
mod partition;

pub use node::HierarchyNode;
pub use tree::TreeLayout;
pub use treemap::{TreemapLayout, TilingMethod};
pub use pack::{PackLayout, PackStrategy};
pub use partition::{PartitionLayout, PartitionNode};

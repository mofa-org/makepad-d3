//! Layout algorithms for data visualization
//!
//! This module provides layout algorithms for positioning nodes in
//! visualizations, including force-directed graphs and hierarchical layouts.
//!
//! # Force-Directed Layouts
//!
//! Force-directed layouts position nodes by simulating physical forces:
//!
//! - [`ForceSimulation`]: Core simulation engine
//! - [`ManyBodyForce`]: Attraction/repulsion between all nodes
//! - [`LinkForce`]: Spring forces between connected nodes
//! - [`CollideForce`]: Collision prevention
//! - [`CenterForce`]: Centering force
//! - [`PositionForce`]: Forces toward target positions
//!
//! # Hierarchical Layouts
//!
//! Hierarchical layouts for tree-structured data:
//!
//! - [`HierarchyNode`]: Node structure for hierarchical data
//! - [`TreeLayout`]: Tidy tree layout (Reingold-Tilford)
//! - [`TreemapLayout`]: Space-filling rectangle layout
//! - [`PackLayout`]: Circle packing layout
//!
//! # Example
//!
//! ```
//! use makepad_d3::layout::force::{ForceSimulation, SimulationNode, ManyBodyForce, LinkForce};
//!
//! // Create nodes
//! let nodes: Vec<SimulationNode> = (0..10)
//!     .map(|i| SimulationNode::new(i))
//!     .collect();
//!
//! // Create links
//! let links = vec![(0, 1), (1, 2), (2, 3), (0, 4), (4, 5)];
//!
//! // Set up simulation
//! let mut sim = ForceSimulation::new(nodes)
//!     .add_force("charge", ManyBodyForce::new().strength(-30.0))
//!     .add_force("links", LinkForce::new(links).distance(50.0));
//!
//! // Run simulation
//! sim.tick_n(300);
//!
//! // Get final positions
//! for node in sim.nodes() {
//!     println!("Node {}: ({:.1}, {:.1})", node.id, node.x, node.y);
//! }
//! ```

pub mod delaunay;
pub mod force;
pub mod hierarchy;

pub use force::{
    CenterForce, CollideForce, Force, ForceSimulation, LinkForce, ManyBodyForce, PositionForce,
    RadialForce, SimulationLink, SimulationNode,
};

pub use hierarchy::{
    HierarchyNode, PackLayout, PackStrategy, TilingMethod, TreeLayout, TreemapLayout,
};

pub use delaunay::{Delaunay, Point, Voronoi};

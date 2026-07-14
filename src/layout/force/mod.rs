//! Force-directed graph layout simulation
//!
//! Implements D3-style force simulation for positioning nodes.
//!
//! # Overview
//!
//! A force simulation iteratively adjusts node positions based on
//! physical forces. Common forces include:
//!
//! - **Charge/ManyBody**: Attraction or repulsion between all nodes
//! - **Link**: Spring forces between connected nodes
//! - **Collision**: Prevents node overlap
//! - **Center**: Pulls nodes toward center
//! - **Position**: Pulls nodes toward target positions
//!
//! # Example
//!
//! ```
//! use makepad_d3::layout::force::{ForceSimulation, SimulationNode, ManyBodyForce};
//!
//! let nodes: Vec<SimulationNode> = (0..5)
//!     .map(|i| SimulationNode::new(i))
//!     .collect();
//!
//! let mut sim = ForceSimulation::new(nodes)
//!     .add_force("charge", ManyBodyForce::new());
//!
//! // Run 100 iterations
//! sim.tick_n(100);
//! ```

mod forces;
mod simulation;

pub use forces::{
    CenterForce, CollideForce, Force, LinkForce, ManyBodyForce, PositionForce, RadialForce,
};
pub use simulation::{ForceSimulation, SimulationLink, SimulationNode};

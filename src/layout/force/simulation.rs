//! Force simulation core implementation
//!
//! The simulation engine that manages nodes and applies forces.

use super::forces::Force;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A node in the force simulation
///
/// Each node has a position, velocity, and can be fixed in place.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SimulationNode {
    /// Unique identifier
    pub id: usize,
    /// X position
    pub x: f64,
    /// Y position
    pub y: f64,
    /// X velocity
    pub vx: f64,
    /// Y velocity
    pub vy: f64,
    /// Fixed X position (if Some, node won't move in X)
    pub fx: Option<f64>,
    /// Fixed Y position (if Some, node won't move in Y)
    pub fy: Option<f64>,
    /// Node radius (for collision detection)
    pub radius: f64,
    /// Node index in the simulation
    pub index: usize,
}

impl SimulationNode {
    /// Create a new node with the given ID
    pub fn new(id: usize) -> Self {
        Self {
            id,
            x: 0.0,
            y: 0.0,
            vx: 0.0,
            vy: 0.0,
            fx: None,
            fy: None,
            radius: 5.0,
            index: 0,
        }
    }

    /// Create a node at a specific position
    pub fn at(id: usize, x: f64, y: f64) -> Self {
        Self {
            id,
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            fx: None,
            fy: None,
            radius: 5.0,
            index: 0,
        }
    }

    /// Set the position
    pub fn with_position(mut self, x: f64, y: f64) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    /// Set the radius
    pub fn with_radius(mut self, radius: f64) -> Self {
        self.radius = radius;
        self
    }

    /// Fix the node at its current position
    pub fn fix(&mut self) {
        self.fx = Some(self.x);
        self.fy = Some(self.y);
    }

    /// Fix the node at a specific position
    pub fn fix_at(&mut self, x: f64, y: f64) {
        self.fx = Some(x);
        self.fy = Some(y);
    }

    /// Unfix the node
    pub fn unfix(&mut self) {
        self.fx = None;
        self.fy = None;
    }

    /// Check if node is fixed
    pub fn is_fixed(&self) -> bool {
        self.fx.is_some() || self.fy.is_some()
    }
}

impl Default for SimulationNode {
    fn default() -> Self {
        Self::new(0)
    }
}

/// A link between two nodes
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SimulationLink {
    /// Source node index
    pub source: usize,
    /// Target node index
    pub target: usize,
    /// Link strength (0.0 to 1.0)
    pub strength: f64,
    /// Target distance
    pub distance: f64,
    /// Link index
    pub index: usize,
}

impl SimulationLink {
    /// Create a new link between two nodes
    pub fn new(source: usize, target: usize) -> Self {
        Self {
            source,
            target,
            strength: 1.0,
            distance: 30.0,
            index: 0,
        }
    }

    /// Set the link strength
    pub fn with_strength(mut self, strength: f64) -> Self {
        self.strength = strength.clamp(0.0, 1.0);
        self
    }

    /// Set the target distance
    pub fn with_distance(mut self, distance: f64) -> Self {
        self.distance = distance.max(0.0);
        self
    }
}

impl From<(usize, usize)> for SimulationLink {
    fn from((source, target): (usize, usize)) -> Self {
        Self::new(source, target)
    }
}

/// Force simulation for positioning nodes
///
/// The simulation runs iteratively, applying forces to nodes and
/// updating their positions based on velocity.
///
/// # Example
///
/// ```
/// use makepad_d3::layout::force::{ForceSimulation, SimulationNode, ManyBodyForce};
///
/// let nodes: Vec<SimulationNode> = (0..10)
///     .map(|i| SimulationNode::new(i))
///     .collect();
///
/// let mut sim = ForceSimulation::new(nodes)
///     .add_force("charge", ManyBodyForce::new());
///
/// // Run until stable
/// while !sim.is_stable() {
///     sim.tick();
/// }
/// ```
pub struct ForceSimulation {
    /// Nodes in the simulation
    nodes: Vec<SimulationNode>,
    /// Named forces
    forces: HashMap<String, Box<dyn Force>>,
    /// Current simulation alpha (energy)
    alpha: f64,
    /// Minimum alpha before stopping
    alpha_min: f64,
    /// Alpha decay rate per tick
    alpha_decay: f64,
    /// Target alpha
    alpha_target: f64,
    /// Velocity decay (friction)
    velocity_decay: f64,
    /// Random seed for initial positions
    random_seed: u64,
}

impl ForceSimulation {
    /// Create a new simulation with the given nodes
    pub fn new(mut nodes: Vec<SimulationNode>) -> Self {
        // Initialize node positions and indices
        let mut rng = SimpleRng::new(12345);
        for (i, node) in nodes.iter_mut().enumerate() {
            node.index = i;
            // Initialize with random positions if at origin
            if node.x == 0.0 && node.y == 0.0 {
                let angle = rng.next_f64() * std::f64::consts::TAU;
                let radius = rng.next_f64() * 10.0 * (i as f64 + 1.0).sqrt();
                node.x = radius * angle.cos();
                node.y = radius * angle.sin();
            }
        }

        Self {
            nodes,
            forces: HashMap::new(),
            alpha: 1.0,
            alpha_min: 0.001,
            alpha_decay: 0.0228, // ~300 iterations to reach alpha_min
            alpha_target: 0.0,
            velocity_decay: 0.4,
            random_seed: 12345,
        }
    }

    /// Add a force to the simulation
    pub fn add_force<F: Force + 'static>(mut self, name: &str, force: F) -> Self {
        self.forces.insert(name.to_string(), Box::new(force));
        self
    }

    /// Remove a force from the simulation
    pub fn remove_force(&mut self, name: &str) -> Option<Box<dyn Force>> {
        self.forces.remove(name)
    }

    /// Get a reference to a force by name
    pub fn force(&self, name: &str) -> Option<&dyn Force> {
        self.forces.get(name).map(|f| f.as_ref())
    }

    /// Set the alpha value (simulation energy)
    pub fn alpha(mut self, alpha: f64) -> Self {
        self.alpha = alpha.clamp(0.0, 1.0);
        self
    }

    /// Set the minimum alpha
    pub fn alpha_min(mut self, min: f64) -> Self {
        self.alpha_min = min.max(0.0);
        self
    }

    /// Set the alpha decay rate
    pub fn alpha_decay(mut self, decay: f64) -> Self {
        self.alpha_decay = decay.clamp(0.0, 1.0);
        self
    }

    /// Set the alpha target
    pub fn alpha_target(mut self, target: f64) -> Self {
        self.alpha_target = target.clamp(0.0, 1.0);
        self
    }

    /// Set the velocity decay (friction)
    pub fn velocity_decay(mut self, decay: f64) -> Self {
        self.velocity_decay = decay.clamp(0.0, 1.0);
        self
    }

    /// Get the current alpha
    pub fn get_alpha(&self) -> f64 {
        self.alpha
    }

    /// Check if simulation has stabilized
    pub fn is_stable(&self) -> bool {
        self.alpha < self.alpha_min
    }

    /// Get the nodes
    pub fn nodes(&self) -> &[SimulationNode] {
        &self.nodes
    }

    /// Get mutable access to nodes
    pub fn nodes_mut(&mut self) -> &mut [SimulationNode] {
        &mut self.nodes
    }

    /// Get a node by index
    pub fn node(&self, index: usize) -> Option<&SimulationNode> {
        self.nodes.get(index)
    }

    /// Get a mutable node by index
    pub fn node_mut(&mut self, index: usize) -> Option<&mut SimulationNode> {
        self.nodes.get_mut(index)
    }

    /// Get number of nodes
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Perform one simulation tick
    pub fn tick(&mut self) {
        // Decay alpha
        self.alpha += (self.alpha_target - self.alpha) * self.alpha_decay;

        // Apply forces
        let force_names: Vec<String> = self.forces.keys().cloned().collect();
        for name in force_names {
            if let Some(force) = self.forces.get(&name) {
                let alpha = self.alpha;
                force.apply(&mut self.nodes, alpha);
            }
        }

        // Update positions
        let decay = 1.0 - self.velocity_decay;
        for node in &mut self.nodes {
            // Apply velocity decay
            node.vx *= decay;
            node.vy *= decay;

            // Update position (respecting fixed positions)
            if let Some(fx) = node.fx {
                node.x = fx;
                node.vx = 0.0;
            } else {
                node.x += node.vx;
            }

            if let Some(fy) = node.fy {
                node.y = fy;
                node.vy = 0.0;
            } else {
                node.y += node.vy;
            }
        }
    }

    /// Run multiple ticks
    pub fn tick_n(&mut self, n: usize) {
        for _ in 0..n {
            self.tick();
        }
    }

    /// Run until stable or max iterations reached
    pub fn run(&mut self, max_iterations: usize) -> usize {
        let mut iterations = 0;
        while !self.is_stable() && iterations < max_iterations {
            self.tick();
            iterations += 1;
        }
        iterations
    }

    /// Restart the simulation
    pub fn restart(&mut self) {
        self.alpha = 1.0;
    }

    /// Stop the simulation
    pub fn stop(&mut self) {
        self.alpha = 0.0;
    }

    /// Find node nearest to a point
    pub fn find(&self, x: f64, y: f64) -> Option<&SimulationNode> {
        self.find_within(x, y, f64::INFINITY)
    }

    /// Find node nearest to a point within a radius
    pub fn find_within(&self, x: f64, y: f64, radius: f64) -> Option<&SimulationNode> {
        let mut closest: Option<&SimulationNode> = None;
        let mut closest_dist = radius * radius;

        for node in &self.nodes {
            let dx = node.x - x;
            let dy = node.y - y;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq < closest_dist {
                closest_dist = dist_sq;
                closest = Some(node);
            }
        }

        closest
    }

    /// Reheat the simulation (useful after adding/removing nodes)
    pub fn reheat(&mut self) {
        self.alpha = 1.0;
    }

    /// Add a node to the simulation
    pub fn add_node(&mut self, mut node: SimulationNode) {
        node.index = self.nodes.len();
        self.nodes.push(node);
    }

    /// Remove a node by index
    pub fn remove_node(&mut self, index: usize) -> Option<SimulationNode> {
        if index < self.nodes.len() {
            let node = self.nodes.remove(index);
            // Update indices
            for (i, n) in self.nodes.iter_mut().enumerate() {
                n.index = i;
            }
            Some(node)
        } else {
            None
        }
    }
}

/// Simple pseudo-random number generator
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.state
    }

    fn next_f64(&mut self) -> f64 {
        (self.next() as f64) / (u64::MAX as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulation_node_new() {
        let node = SimulationNode::new(42);
        assert_eq!(node.id, 42);
        assert_eq!(node.x, 0.0);
        assert_eq!(node.y, 0.0);
    }

    #[test]
    fn test_simulation_node_at() {
        let node = SimulationNode::at(1, 100.0, 200.0);
        assert_eq!(node.id, 1);
        assert_eq!(node.x, 100.0);
        assert_eq!(node.y, 200.0);
    }

    #[test]
    fn test_simulation_node_fix() {
        let mut node = SimulationNode::at(0, 10.0, 20.0);
        assert!(!node.is_fixed());

        node.fix();
        assert!(node.is_fixed());
        assert_eq!(node.fx, Some(10.0));
        assert_eq!(node.fy, Some(20.0));

        node.unfix();
        assert!(!node.is_fixed());
    }

    #[test]
    fn test_simulation_link_new() {
        let link = SimulationLink::new(0, 1);
        assert_eq!(link.source, 0);
        assert_eq!(link.target, 1);
        assert_eq!(link.strength, 1.0);
    }

    #[test]
    fn test_simulation_link_from_tuple() {
        let link: SimulationLink = (3, 5).into();
        assert_eq!(link.source, 3);
        assert_eq!(link.target, 5);
    }

    #[test]
    fn test_force_simulation_new() {
        let nodes: Vec<SimulationNode> = (0..5).map(|i| SimulationNode::new(i)).collect();

        let sim = ForceSimulation::new(nodes);
        assert_eq!(sim.node_count(), 5);
        assert_eq!(sim.get_alpha(), 1.0);
        assert!(!sim.is_stable());
    }

    #[test]
    fn test_force_simulation_tick() {
        let nodes: Vec<SimulationNode> = (0..3).map(|i| SimulationNode::new(i)).collect();

        let mut sim = ForceSimulation::new(nodes);
        let initial_alpha = sim.get_alpha();

        sim.tick();

        // Alpha should decay
        assert!(sim.get_alpha() < initial_alpha);
    }

    #[test]
    fn test_force_simulation_tick_n() {
        let nodes: Vec<SimulationNode> = (0..3).map(|i| SimulationNode::new(i)).collect();

        let mut sim = ForceSimulation::new(nodes);
        sim.tick_n(100);

        // Alpha should have decayed significantly
        assert!(sim.get_alpha() < 0.5);
    }

    #[test]
    fn test_force_simulation_run() {
        let nodes: Vec<SimulationNode> = (0..3).map(|i| SimulationNode::new(i)).collect();

        let mut sim = ForceSimulation::new(nodes);
        let iterations = sim.run(1000);

        assert!(sim.is_stable());
        assert!(iterations < 1000);
    }

    #[test]
    fn test_force_simulation_restart() {
        let nodes: Vec<SimulationNode> = (0..3).map(|i| SimulationNode::new(i)).collect();

        let mut sim = ForceSimulation::new(nodes);
        sim.tick_n(100);
        assert!(sim.get_alpha() < 1.0);

        sim.restart();
        assert_eq!(sim.get_alpha(), 1.0);
    }

    #[test]
    fn test_force_simulation_stop() {
        let nodes: Vec<SimulationNode> = (0..3).map(|i| SimulationNode::new(i)).collect();

        let mut sim = ForceSimulation::new(nodes);
        sim.stop();
        assert_eq!(sim.get_alpha(), 0.0);
        assert!(sim.is_stable());
    }

    #[test]
    fn test_force_simulation_find() {
        let nodes = vec![
            SimulationNode::at(0, 0.0, 0.0),
            SimulationNode::at(1, 100.0, 0.0),
            SimulationNode::at(2, 0.0, 100.0),
        ];

        let sim = ForceSimulation::new(nodes);

        // Find nearest to origin
        let nearest = sim.find(10.0, 10.0).unwrap();
        assert_eq!(nearest.id, 0);

        // Find nearest to (100, 0)
        let nearest = sim.find(90.0, 5.0).unwrap();
        assert_eq!(nearest.id, 1);
    }

    #[test]
    fn test_force_simulation_find_within() {
        let nodes = vec![
            SimulationNode::at(0, 0.0, 0.0),
            SimulationNode::at(1, 100.0, 0.0),
        ];

        let sim = ForceSimulation::new(nodes);

        // Should find node within radius
        let result = sim.find_within(5.0, 5.0, 20.0);
        assert!(result.is_some());

        // Should not find node outside radius
        let result = sim.find_within(50.0, 50.0, 10.0);
        assert!(result.is_none());
    }

    #[test]
    fn test_force_simulation_fixed_nodes() {
        let mut nodes = vec![
            SimulationNode::at(0, 0.0, 0.0),
            SimulationNode::at(1, 100.0, 0.0),
        ];
        nodes[0].fix();

        let mut sim = ForceSimulation::new(nodes);
        sim.tick_n(10);

        // Fixed node should stay at origin
        let node0 = sim.node(0).unwrap();
        assert_eq!(node0.x, 0.0);
        assert_eq!(node0.y, 0.0);
    }

    #[test]
    fn test_force_simulation_add_remove_node() {
        let nodes = vec![SimulationNode::new(0)];
        let mut sim = ForceSimulation::new(nodes);

        assert_eq!(sim.node_count(), 1);

        sim.add_node(SimulationNode::new(1));
        assert_eq!(sim.node_count(), 2);

        sim.remove_node(0);
        assert_eq!(sim.node_count(), 1);
        assert_eq!(sim.nodes()[0].index, 0);
    }

    #[test]
    fn test_simulation_configuration() {
        let nodes = vec![SimulationNode::new(0)];
        let sim = ForceSimulation::new(nodes)
            .alpha(0.5)
            .alpha_min(0.01)
            .alpha_decay(0.05)
            .velocity_decay(0.3);

        assert_eq!(sim.get_alpha(), 0.5);
    }
}

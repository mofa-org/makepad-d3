//! Force implementations for the simulation
//!
//! Various forces that can be applied to nodes in a simulation.

use super::simulation::{SimulationLink, SimulationNode};

/// Trait for forces that can be applied to nodes
pub trait Force: Send + Sync {
    /// Apply the force to nodes
    fn apply(&self, nodes: &mut [SimulationNode], alpha: f64);

    /// Get the force type name
    fn force_type(&self) -> &'static str;
}

/// Many-body force (charge) for attraction or repulsion between all nodes
///
/// Positive strength attracts, negative strength repels.
///
/// # Example
///
/// ```
/// use makepad_d3::layout::force::ManyBodyForce;
///
/// // Repelling force (default)
/// let repel = ManyBodyForce::new().strength(-30.0);
///
/// // Attracting force
/// let attract = ManyBodyForce::new().strength(10.0);
/// ```
#[derive(Clone, Debug)]
pub struct ManyBodyForce {
    /// Force strength (negative = repel, positive = attract)
    strength: f64,
    /// Minimum distance to prevent extreme forces
    distance_min: f64,
    /// Maximum distance for force calculation
    distance_max: f64,
    /// Theta for Barnes-Hut approximation (not used in simple impl)
    theta: f64,
}

impl Default for ManyBodyForce {
    fn default() -> Self {
        Self::new()
    }
}

impl ManyBodyForce {
    /// Create a new many-body force
    pub fn new() -> Self {
        Self {
            strength: -30.0,
            distance_min: 1.0,
            distance_max: f64::INFINITY,
            theta: 0.9,
        }
    }

    /// Set the force strength
    pub fn strength(mut self, strength: f64) -> Self {
        self.strength = strength;
        self
    }

    /// Set the minimum distance
    pub fn distance_min(mut self, min: f64) -> Self {
        self.distance_min = min.max(0.001);
        self
    }

    /// Set the maximum distance
    pub fn distance_max(mut self, max: f64) -> Self {
        self.distance_max = max;
        self
    }

    /// Get the strength
    pub fn get_strength(&self) -> f64 {
        self.strength
    }
}

impl Force for ManyBodyForce {
    fn apply(&self, nodes: &mut [SimulationNode], alpha: f64) {
        let n = nodes.len();
        if n < 2 {
            return;
        }

        // O(n²) implementation - for large graphs, would use Barnes-Hut
        for i in 0..n {
            for j in (i + 1)..n {
                let dx = nodes[j].x - nodes[i].x;
                let dy = nodes[j].y - nodes[i].y;

                let mut dist_sq = dx * dx + dy * dy;

                // Clamp to minimum distance
                if dist_sq < self.distance_min * self.distance_min {
                    dist_sq = self.distance_min * self.distance_min;
                }

                // Skip if beyond maximum distance
                if dist_sq > self.distance_max * self.distance_max {
                    continue;
                }

                let dist = dist_sq.sqrt();
                let force = self.strength * alpha / dist_sq;

                let fx = dx / dist * force;
                let fy = dy / dist * force;

                nodes[i].vx += fx;
                nodes[i].vy += fy;
                nodes[j].vx -= fx;
                nodes[j].vy -= fy;
            }
        }
    }

    fn force_type(&self) -> &'static str {
        "many-body"
    }
}

/// Link force for spring-like connections between nodes
///
/// # Example
///
/// ```
/// use makepad_d3::layout::force::LinkForce;
///
/// let links = vec![(0, 1), (1, 2), (2, 3)];
/// let force = LinkForce::new(links)
///     .distance(50.0)
///     .strength(0.5);
/// ```
#[derive(Clone, Debug)]
pub struct LinkForce {
    /// Links between nodes
    links: Vec<SimulationLink>,
    /// Default link distance
    distance: f64,
    /// Default link strength
    strength: f64,
    /// Number of iterations per tick
    iterations: usize,
}

impl Default for LinkForce {
    fn default() -> Self {
        Self::new(Vec::<(usize, usize)>::new())
    }
}

impl LinkForce {
    /// Create a new link force
    pub fn new<L: Into<SimulationLink>>(links: Vec<L>) -> Self {
        let links: Vec<SimulationLink> = links
            .into_iter()
            .enumerate()
            .map(|(i, l)| {
                let mut link = l.into();
                link.index = i;
                link
            })
            .collect();

        Self {
            links,
            distance: 30.0,
            strength: 1.0,
            iterations: 1,
        }
    }

    /// Set the default link distance
    pub fn distance(mut self, distance: f64) -> Self {
        self.distance = distance.max(0.0);
        // Update all links that don't have custom distance
        for link in &mut self.links {
            link.distance = self.distance;
        }
        self
    }

    /// Set the default link strength
    pub fn strength(mut self, strength: f64) -> Self {
        self.strength = strength.clamp(0.0, 1.0);
        for link in &mut self.links {
            link.strength = self.strength;
        }
        self
    }

    /// Set the number of iterations per tick
    pub fn iterations(mut self, iterations: usize) -> Self {
        self.iterations = iterations.max(1);
        self
    }

    /// Get the links
    pub fn links(&self) -> &[SimulationLink] {
        &self.links
    }

    /// Add a link
    pub fn add_link<L: Into<SimulationLink>>(&mut self, link: L) {
        let mut link = link.into();
        link.index = self.links.len();
        link.distance = self.distance;
        link.strength = self.strength;
        self.links.push(link);
    }
}

impl Force for LinkForce {
    fn apply(&self, nodes: &mut [SimulationNode], alpha: f64) {
        for _ in 0..self.iterations {
            for link in &self.links {
                let source = link.source;
                let target = link.target;

                if source >= nodes.len() || target >= nodes.len() {
                    continue;
                }

                let dx = nodes[target].x - nodes[source].x;
                let dy = nodes[target].y - nodes[source].y;

                let dist = (dx * dx + dy * dy).sqrt().max(0.001);
                let force = (dist - link.distance) / dist * alpha * link.strength;

                let fx = dx * force;
                let fy = dy * force;

                // Bias based on node degree (simplified - use 0.5 for equal)
                let bias = 0.5;

                nodes[target].vx -= fx * bias;
                nodes[target].vy -= fy * bias;
                nodes[source].vx += fx * (1.0 - bias);
                nodes[source].vy += fy * (1.0 - bias);
            }
        }
    }

    fn force_type(&self) -> &'static str {
        "link"
    }
}

/// Collision force to prevent node overlap
///
/// # Example
///
/// ```
/// use makepad_d3::layout::force::CollideForce;
///
/// let force = CollideForce::new()
///     .radius(10.0)
///     .strength(0.7);
/// ```
#[derive(Clone, Debug)]
pub struct CollideForce {
    /// Default collision radius
    radius: f64,
    /// Force strength (0.0 to 1.0)
    strength: f64,
    /// Number of iterations per tick
    iterations: usize,
}

impl Default for CollideForce {
    fn default() -> Self {
        Self::new()
    }
}

impl CollideForce {
    /// Create a new collision force
    pub fn new() -> Self {
        Self {
            radius: 5.0,
            strength: 0.7,
            iterations: 1,
        }
    }

    /// Set the default collision radius
    pub fn radius(mut self, radius: f64) -> Self {
        self.radius = radius.max(0.0);
        self
    }

    /// Set the force strength
    pub fn strength(mut self, strength: f64) -> Self {
        self.strength = strength.clamp(0.0, 1.0);
        self
    }

    /// Set the number of iterations per tick
    pub fn iterations(mut self, iterations: usize) -> Self {
        self.iterations = iterations.max(1);
        self
    }
}

impl Force for CollideForce {
    fn apply(&self, nodes: &mut [SimulationNode], _alpha: f64) {
        let n = nodes.len();
        if n < 2 {
            return;
        }

        for _ in 0..self.iterations {
            // O(n²) collision detection
            for i in 0..n {
                for j in (i + 1)..n {
                    let ri = nodes[i].radius.max(self.radius);
                    let rj = nodes[j].radius.max(self.radius);
                    let r = ri + rj;

                    let dx = nodes[j].x - nodes[i].x;
                    let dy = nodes[j].y - nodes[i].y;

                    let dist_sq = dx * dx + dy * dy;

                    if dist_sq < r * r {
                        let dist = dist_sq.sqrt().max(0.001);
                        let overlap = (r - dist) / dist * self.strength;

                        let mx = dx * overlap * 0.5;
                        let my = dy * overlap * 0.5;

                        nodes[i].vx -= mx;
                        nodes[i].vy -= my;
                        nodes[j].vx += mx;
                        nodes[j].vy += my;
                    }
                }
            }
        }
    }

    fn force_type(&self) -> &'static str {
        "collide"
    }
}

/// Center force to keep nodes centered around a point
///
/// # Example
///
/// ```
/// use makepad_d3::layout::force::CenterForce;
///
/// let force = CenterForce::new()
///     .x(400.0)
///     .y(300.0)
///     .strength(0.1);
/// ```
#[derive(Clone, Debug)]
pub struct CenterForce {
    /// Center X coordinate
    x: f64,
    /// Center Y coordinate
    y: f64,
    /// Force strength
    strength: f64,
}

impl Default for CenterForce {
    fn default() -> Self {
        Self::new()
    }
}

impl CenterForce {
    /// Create a new center force at origin
    pub fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            strength: 1.0,
        }
    }

    /// Set the center X coordinate
    pub fn x(mut self, x: f64) -> Self {
        self.x = x;
        self
    }

    /// Set the center Y coordinate
    pub fn y(mut self, y: f64) -> Self {
        self.y = y;
        self
    }

    /// Set the center position
    pub fn center(mut self, x: f64, y: f64) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    /// Set the force strength
    pub fn strength(mut self, strength: f64) -> Self {
        self.strength = strength.clamp(0.0, 1.0);
        self
    }
}

impl Force for CenterForce {
    fn apply(&self, nodes: &mut [SimulationNode], _alpha: f64) {
        if nodes.is_empty() {
            return;
        }

        // Calculate current center of mass
        let mut cx = 0.0;
        let mut cy = 0.0;
        for node in nodes.iter() {
            cx += node.x;
            cy += node.y;
        }
        cx /= nodes.len() as f64;
        cy /= nodes.len() as f64;

        // Move nodes toward target center
        let dx = (self.x - cx) * self.strength;
        let dy = (self.y - cy) * self.strength;

        for node in nodes.iter_mut() {
            node.x += dx;
            node.y += dy;
        }
    }

    fn force_type(&self) -> &'static str {
        "center"
    }
}

/// Position force to pull nodes toward target positions
///
/// # Example
///
/// ```
/// use makepad_d3::layout::force::PositionForce;
///
/// // Pull toward X axis
/// let force_x = PositionForce::x(0.0).strength(0.1);
///
/// // Pull toward Y axis
/// let force_y = PositionForce::y(0.0).strength(0.1);
///
/// // Pull toward specific point
/// let force_xy = PositionForce::xy(100.0, 100.0).strength(0.05);
/// ```
#[derive(Clone, Debug)]
pub struct PositionForce {
    /// Target X (None = don't affect X)
    target_x: Option<f64>,
    /// Target Y (None = don't affect Y)
    target_y: Option<f64>,
    /// Force strength
    strength: f64,
}

impl Default for PositionForce {
    fn default() -> Self {
        Self::new()
    }
}

impl PositionForce {
    /// Create a new position force (no target set)
    pub fn new() -> Self {
        Self {
            target_x: None,
            target_y: None,
            strength: 0.1,
        }
    }

    /// Create an X-axis position force
    pub fn x(target: f64) -> Self {
        Self {
            target_x: Some(target),
            target_y: None,
            strength: 0.1,
        }
    }

    /// Create a Y-axis position force
    pub fn y(target: f64) -> Self {
        Self {
            target_x: None,
            target_y: Some(target),
            strength: 0.1,
        }
    }

    /// Create a 2D position force
    pub fn xy(x: f64, y: f64) -> Self {
        Self {
            target_x: Some(x),
            target_y: Some(y),
            strength: 0.1,
        }
    }

    /// Set the X target
    pub fn with_x(mut self, x: f64) -> Self {
        self.target_x = Some(x);
        self
    }

    /// Set the Y target
    pub fn with_y(mut self, y: f64) -> Self {
        self.target_y = Some(y);
        self
    }

    /// Set the force strength
    pub fn strength(mut self, strength: f64) -> Self {
        self.strength = strength.clamp(0.0, 1.0);
        self
    }
}

impl Force for PositionForce {
    fn apply(&self, nodes: &mut [SimulationNode], alpha: f64) {
        let strength = self.strength * alpha;

        for node in nodes.iter_mut() {
            if let Some(tx) = self.target_x {
                node.vx += (tx - node.x) * strength;
            }
            if let Some(ty) = self.target_y {
                node.vy += (ty - node.y) * strength;
            }
        }
    }

    fn force_type(&self) -> &'static str {
        "position"
    }
}

/// Radial force to pull nodes toward a circle
///
/// # Example
///
/// ```
/// use makepad_d3::layout::force::RadialForce;
///
/// let force = RadialForce::new(100.0)  // radius 100
///     .center(200.0, 200.0)
///     .strength(0.1);
/// ```
#[derive(Clone, Debug)]
pub struct RadialForce {
    /// Target radius
    radius: f64,
    /// Center X
    x: f64,
    /// Center Y
    y: f64,
    /// Force strength
    strength: f64,
}

impl RadialForce {
    /// Create a new radial force
    pub fn new(radius: f64) -> Self {
        Self {
            radius: radius.max(0.0),
            x: 0.0,
            y: 0.0,
            strength: 0.1,
        }
    }

    /// Set the target radius
    pub fn radius(mut self, radius: f64) -> Self {
        self.radius = radius.max(0.0);
        self
    }

    /// Set the center position
    pub fn center(mut self, x: f64, y: f64) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    /// Set the force strength
    pub fn strength(mut self, strength: f64) -> Self {
        self.strength = strength.clamp(0.0, 1.0);
        self
    }
}

impl Force for RadialForce {
    fn apply(&self, nodes: &mut [SimulationNode], alpha: f64) {
        let strength = self.strength * alpha;

        for node in nodes.iter_mut() {
            let dx = node.x - self.x;
            let dy = node.y - self.y;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist > 0.001 {
                let delta = (self.radius - dist) * strength / dist;
                node.vx += dx * delta;
                node.vy += dy * delta;
            }
        }
    }

    fn force_type(&self) -> &'static str {
        "radial"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_nodes(n: usize) -> Vec<SimulationNode> {
        (0..n)
            .map(|i| SimulationNode::at(i, i as f64 * 10.0, 0.0))
            .collect()
    }

    #[test]
    fn test_many_body_force_new() {
        let force = ManyBodyForce::new();
        assert_eq!(force.get_strength(), -30.0);
    }

    #[test]
    fn test_many_body_force_repel() {
        let force = ManyBodyForce::new().strength(-100.0);
        let mut nodes = vec![
            SimulationNode::at(0, 0.0, 0.0),
            SimulationNode::at(1, 10.0, 0.0),
        ];

        force.apply(&mut nodes, 1.0);

        // Nodes should be pushed apart
        assert!(nodes[0].vx < 0.0); // First node pushed left
        assert!(nodes[1].vx > 0.0); // Second node pushed right
    }

    #[test]
    fn test_many_body_force_attract() {
        let force = ManyBodyForce::new().strength(100.0);
        let mut nodes = vec![
            SimulationNode::at(0, 0.0, 0.0),
            SimulationNode::at(1, 100.0, 0.0),
        ];

        force.apply(&mut nodes, 1.0);

        // Nodes should be pulled together
        assert!(nodes[0].vx > 0.0); // First node pulled right
        assert!(nodes[1].vx < 0.0); // Second node pulled left
    }

    #[test]
    fn test_link_force_new() {
        let links = vec![(0, 1), (1, 2)];
        let force = LinkForce::new(links);
        assert_eq!(force.links().len(), 2);
    }

    #[test]
    fn test_link_force_apply() {
        let links = vec![(0, 1)];
        let force = LinkForce::new(links).distance(50.0);
        let mut nodes = vec![
            SimulationNode::at(0, 0.0, 0.0),
            SimulationNode::at(1, 100.0, 0.0), // 100 units apart, target is 50
        ];

        force.apply(&mut nodes, 1.0);

        // Nodes should be pulled together
        assert!(nodes[0].vx > 0.0);
        assert!(nodes[1].vx < 0.0);
    }

    #[test]
    fn test_collide_force_new() {
        let force = CollideForce::new();
        assert_eq!(force.radius, 5.0);
    }

    #[test]
    fn test_collide_force_apply() {
        let force = CollideForce::new().radius(20.0);
        let mut nodes = vec![
            SimulationNode::at(0, 0.0, 0.0),
            SimulationNode::at(1, 10.0, 0.0), // Overlapping at radius 20
        ];

        force.apply(&mut nodes, 1.0);

        // Nodes should be pushed apart
        assert!(nodes[0].vx < 0.0);
        assert!(nodes[1].vx > 0.0);
    }

    #[test]
    fn test_center_force_new() {
        let force = CenterForce::new();
        assert_eq!(force.x, 0.0);
        assert_eq!(force.y, 0.0);
    }

    #[test]
    fn test_center_force_apply() {
        let force = CenterForce::new().center(100.0, 100.0);
        let mut nodes = vec![
            SimulationNode::at(0, 0.0, 0.0),
            SimulationNode::at(1, 0.0, 0.0),
        ];

        force.apply(&mut nodes, 1.0);

        // Nodes should be moved to center at (100, 100)
        assert_eq!(nodes[0].x, 100.0);
        assert_eq!(nodes[0].y, 100.0);
        assert_eq!(nodes[1].x, 100.0);
        assert_eq!(nodes[1].y, 100.0);
    }

    #[test]
    fn test_position_force_x() {
        let force = PositionForce::x(100.0).strength(1.0);
        let mut nodes = vec![SimulationNode::at(0, 0.0, 50.0)];

        force.apply(&mut nodes, 1.0);

        // Should pull toward X=100
        assert!(nodes[0].vx > 0.0);
        // Y should be unaffected
        assert_eq!(nodes[0].vy, 0.0);
    }

    #[test]
    fn test_position_force_y() {
        let force = PositionForce::y(100.0).strength(1.0);
        let mut nodes = vec![SimulationNode::at(0, 50.0, 0.0)];

        force.apply(&mut nodes, 1.0);

        // X should be unaffected
        assert_eq!(nodes[0].vx, 0.0);
        // Should pull toward Y=100
        assert!(nodes[0].vy > 0.0);
    }

    #[test]
    fn test_position_force_xy() {
        let force = PositionForce::xy(100.0, 100.0).strength(1.0);
        let mut nodes = vec![SimulationNode::at(0, 0.0, 0.0)];

        force.apply(&mut nodes, 1.0);

        assert!(nodes[0].vx > 0.0);
        assert!(nodes[0].vy > 0.0);
    }

    #[test]
    fn test_radial_force_new() {
        let force = RadialForce::new(100.0);
        assert_eq!(force.radius, 100.0);
    }

    #[test]
    fn test_radial_force_apply() {
        let force = RadialForce::new(100.0).strength(1.0);
        let mut nodes = vec![SimulationNode::at(0, 50.0, 0.0)]; // Inside radius

        force.apply(&mut nodes, 1.0);

        // Should be pushed outward
        assert!(nodes[0].vx > 0.0);
    }

    #[test]
    fn test_force_types() {
        assert_eq!(ManyBodyForce::new().force_type(), "many-body");
        assert_eq!(
            LinkForce::new(Vec::<(usize, usize)>::new()).force_type(),
            "link"
        );
        assert_eq!(CollideForce::new().force_type(), "collide");
        assert_eq!(CenterForce::new().force_type(), "center");
        assert_eq!(PositionForce::new().force_type(), "position");
        assert_eq!(RadialForce::new(10.0).force_type(), "radial");
    }
}

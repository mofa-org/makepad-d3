//! Packed Bubble Chart - Non-hierarchical packed circles
//!
//! Displays leaf nodes of a hierarchy as non-overlapping circles,
//! where area is proportional to value and color represents the parent group.
//! Based on D3's bubble chart example.

use makepad_widgets::*;
use makepad_d3::layout::hierarchy::{HierarchyNode, PackLayout};
use super::draw_primitives::DrawPoint;
use super::animation::{ChartAnimator, EasingType};

live_design! {
    link widgets;
    use link::shaders::*;
    use super::draw_primitives::DrawPoint;

    BUBBLE_FONT = {
        font_family: {
            latin = font("crate://self/resources/Manrope-Regular.ttf", 0.0, 0.0),
        }
    }

    pub PackedBubbleChart = {{PackedBubbleChart}} {
        width: Fill,
        height: Fill,

        draw_text: {
            text_style: <BUBBLE_FONT> {
                font_size: 9.0
            },
            color: #000
        }
    }
}

#[derive(Clone, Debug)]
pub struct BubbleItem {
    pub id: String,
    pub group: String,
    pub value: f64,
    pub x: f64,
    pub y: f64,
    pub radius: f64,
    pub color: Vec4,
}

#[derive(Clone)]
struct PackNode {
    id: String,
    value: f64,
}

#[derive(Live, LiveHook, Widget)]
pub struct PackedBubbleChart {
    #[redraw]
    #[live]
    draw_point: DrawPoint,

    #[redraw]
    #[live]
    draw_text: DrawText,

    #[walk]
    walk: Walk,

    #[layout]
    layout: Layout,

    #[rust]
    bubbles: Vec<BubbleItem>,

    #[rust]
    initialized: bool,

    #[rust]
    area: Area,

    #[rust]
    animator: ChartAnimator,

    #[rust]
    animation_started: bool,

    #[rust]
    was_visible: bool,

    #[rust]
    hovered_index: Option<usize>,

    #[rust]
    last_rect: Rect,

    #[rust]
    chart_size: f64,

    #[rust]
    offset_x: f64,

    #[rust]
    offset_y: f64,

    // Ripple animation state (spring physics)
    #[rust]
    ripple_progress: f64,  // 0.0 = no ripple, 1.0 = full ripple

    #[rust]
    ripple_velocity: f64,  // Current velocity for spring physics

    #[rust]
    ripple_target: f64,    // Target ripple state (0.0 or 1.0)

    #[rust]
    ripple_center: Option<(f64, f64, f64)>,  // (x, y, radius) of hovered bubble for ripple

    // Drag state
    #[rust]
    dragging_index: Option<usize>,  // Which bubble is being dragged

    #[rust]
    drag_offset: DVec2,  // Offset from mouse to bubble center when drag started

    #[rust]
    bubble_velocities: Vec<DVec2>,  // Velocities for collision physics
}

// Ripple effect constants
const RIPPLE_STRENGTH: f64 = 0.03;   // How much bubbles are pushed (as fraction of chart size)
const RIPPLE_RADIUS: f64 = 0.3;      // How far the ripple extends (as fraction of chart size)

// Spring physics constants
const SPRING_STIFFNESS: f64 = 180.0; // How snappy the spring is (higher = faster response)
const SPRING_DAMPING: f64 = 12.0;    // How much the spring oscillates (lower = more bouncy)

// Drag physics constants
const COLLISION_STRENGTH: f64 = 0.8;  // How strongly bubbles push each other apart
const VELOCITY_DAMPING: f64 = 0.85;   // How quickly bubbles slow down (lower = more damping)
const BOUNDARY_PADDING: f64 = 0.05;   // Padding from edge of chart area

impl Widget for PackedBubbleChart {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        match event {
            Event::NextFrame(nf) => {
                let mut needs_redraw = false;

                // Start entry animation if not yet started
                if self.initialized && !self.animation_started {
                    self.animator = ChartAnimator::new(1200.0)
                        .with_easing(EasingType::EaseOutElastic);
                    self.animator.start(nf.time);
                    self.animation_started = true;
                    needs_redraw = true;
                }

                // Continue entry animation if running
                if self.animation_started && self.animator.update(nf.time) {
                    needs_redraw = true;
                }

                // Animate ripple using spring physics
                let dt = 0.016; // ~60fps frame time
                let displacement = self.ripple_target - self.ripple_progress;

                // Spring physics: F = -kx - cv (spring force - damping force)
                let spring_force = SPRING_STIFFNESS * displacement;
                let damping_force = SPRING_DAMPING * self.ripple_velocity;
                let acceleration = spring_force - damping_force;

                self.ripple_velocity += acceleration * dt;
                self.ripple_progress += self.ripple_velocity * dt;

                // Check if spring has settled (both position and velocity are small)
                let is_settled = displacement.abs() < 0.001 && self.ripple_velocity.abs() < 0.01;

                if !is_settled {
                    needs_redraw = true;
                } else {
                    // Snap to target when settled
                    self.ripple_progress = self.ripple_target;
                    self.ripple_velocity = 0.0;
                }

                // Run collision physics for all bubbles
                if self.run_collision_physics() {
                    needs_redraw = true;
                }

                if needs_redraw {
                    self.redraw(cx);
                    cx.new_next_frame();
                }
            }
            Event::MouseDown(e) => {
                if let Some(idx) = self.find_bubble_at(e.abs) {
                    // Start dragging this bubble
                    self.dragging_index = Some(idx);

                    // Calculate offset from mouse to bubble center
                    let bubble = &self.bubbles[idx];
                    let center_x = self.offset_x + bubble.x * self.chart_size;
                    let center_y = self.offset_y + bubble.y * self.chart_size;
                    self.drag_offset = dvec2(center_x - e.abs.x, center_y - e.abs.y);

                    // Initialize velocities if needed
                    if self.bubble_velocities.len() != self.bubbles.len() {
                        self.bubble_velocities = vec![dvec2(0.0, 0.0); self.bubbles.len()];
                    }

                    // Reset velocity for dragged bubble
                    self.bubble_velocities[idx] = dvec2(0.0, 0.0);

                    self.redraw(cx);
                    cx.new_next_frame();
                }
            }
            Event::MouseMove(e) => {
                // Handle dragging
                if let Some(drag_idx) = self.dragging_index {
                    // Move the dragged bubble to follow the mouse
                    let new_center_x = e.abs.x + self.drag_offset.x;
                    let new_center_y = e.abs.y + self.drag_offset.y;

                    // Convert to normalized coordinates
                    if self.chart_size > 0.0 {
                        let new_x = (new_center_x - self.offset_x) / self.chart_size;
                        let new_y = (new_center_y - self.offset_y) / self.chart_size;

                        // Clamp to boundaries
                        let r = self.bubbles[drag_idx].radius;
                        self.bubbles[drag_idx].x = new_x.clamp(r + BOUNDARY_PADDING, 1.0 - r - BOUNDARY_PADDING);
                        self.bubbles[drag_idx].y = new_y.clamp(r + BOUNDARY_PADDING, 1.0 - r - BOUNDARY_PADDING);
                    }

                    self.redraw(cx);
                    cx.new_next_frame();
                } else {
                    // Normal hover behavior
                    let old_hovered = self.hovered_index;
                    self.hovered_index = self.find_bubble_at(e.abs);

                    if old_hovered != self.hovered_index {
                        // Update ripple target and center
                        if let Some(idx) = self.hovered_index {
                            let hb = &self.bubbles[idx];
                            self.ripple_center = Some((hb.x, hb.y, hb.radius));
                            self.ripple_target = 1.0;
                        } else {
                            self.ripple_target = 0.0;
                            // Keep ripple_center so it animates out from the last position
                        }
                        self.redraw(cx);
                        cx.new_next_frame();
                    }
                }
            }
            Event::MouseUp(_) => {
                if self.dragging_index.is_some() {
                    self.dragging_index = None;
                    self.redraw(cx);
                    cx.new_next_frame();
                }
            }
            Event::MouseLeave(_) => {
                // Stop dragging and clear hover
                self.dragging_index = None;
                if self.hovered_index.is_some() {
                    self.hovered_index = None;
                    self.ripple_target = 0.0;
                    self.redraw(cx);
                    cx.new_next_frame();
                }
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        if !self.initialized {
            self.initialize_data();
        }

        let rect = cx.walk_turtle_with_area(&mut self.area, walk);
        let width = rect.size.x as f64;
        let height = rect.size.y as f64;

        // Check if we just became visible (rect changed from zero to non-zero)
        let is_visible = width > 0.0 && height > 0.0;
        if is_visible && !self.was_visible {
            // Reset animation to replay
            self.animation_started = false;
            self.animator.reset();
            self.ripple_progress = 0.0;
            self.ripple_velocity = 0.0;
            self.ripple_target = 0.0;
            self.ripple_center = None;
        }
        self.was_visible = is_visible;

        if !is_visible {
            return DrawStep::done();
        }

        // Request next frame to start animation
        if !self.animation_started {
            cx.new_next_frame();
        }

        // Calculate chart size and centering offset
        self.chart_size = width.min(height);
        self.offset_x = rect.pos.x + (width - self.chart_size) / 2.0;
        self.offset_y = rect.pos.y + (height - self.chart_size) / 2.0;
        self.last_rect = rect;

        // Get animation progress
        let base_progress = self.animator.get_progress();

        // Get ripple center for effect (use stored center for smooth animation)
        let ripple_center = self.ripple_center;
        // Spring physics naturally handles easing, can overshoot > 1.0 for bounce effect
        let ripple_amount = self.ripple_progress.max(0.0);

        // Draw all bubbles with staggered animation
        let bubble_count = self.bubbles.len();
        for (i, bubble) in self.bubbles.iter().enumerate() {
            // Stagger animation: each bubble starts slightly later
            let stagger_delay = (i as f64) / (bubble_count as f64) * 0.3;
            let bubble_progress = ((base_progress - stagger_delay) / (1.0 - stagger_delay * 0.5))
                .clamp(0.0, 1.0);

            // Base position (normalized 0-1)
            let mut bx = bubble.x;
            let mut by = bubble.y;

            // Check if this bubble is hovered or dragged
            let is_hovered = self.hovered_index == Some(i);
            let is_dragged = self.dragging_index == Some(i);

            // Apply ripple effect: push non-hovered bubbles away from ripple center
            if !is_hovered && ripple_amount > 0.001 {
                if let Some((hx, hy, hr)) = ripple_center {
                    let dx = bx - hx;
                    let dy = by - hy;
                    let dist = (dx * dx + dy * dy).sqrt();

                    if dist > 0.001 && dist < RIPPLE_RADIUS {
                        // Calculate push strength (stronger when closer)
                        let falloff = 1.0 - (dist / RIPPLE_RADIUS);
                        let falloff = falloff * falloff; // Quadratic falloff for smoother effect
                        let push = RIPPLE_STRENGTH * falloff * (1.0 + hr * 3.0) * ripple_amount;

                        // Normalize direction and apply push
                        let nx = dx / dist;
                        let ny = dy / dist;
                        bx += nx * push;
                        by += ny * push;
                    }
                }
            }

            // Convert to screen coordinates
            let center_x = self.offset_x + bx * self.chart_size;
            let center_y = self.offset_y + by * self.chart_size;
            let target_radius = bubble.radius * self.chart_size;

            // Animate radius from 0 to target
            let mut radius = target_radius * bubble_progress;

            // Apply hover/drag effects
            let mut color = bubble.color;
            if is_dragged {
                // Dragged bubble: larger scale, slightly transparent, brighter
                radius *= 1.12;
                color = vec4(
                    (color.x + 0.2).min(1.0),
                    (color.y + 0.2).min(1.0),
                    (color.z + 0.2).min(1.0),
                    0.9,
                );
            } else if is_hovered && bubble_progress > 0.5 {
                // Hover effect: scale up and brighten (also animated)
                let hover_scale = 1.0 + 0.08 * ripple_amount; // Animate scale
                radius *= hover_scale;
                let brighten = 0.15 * ripple_amount as f32;
                color = vec4(
                    (color.x + brighten).min(1.0),
                    (color.y + brighten).min(1.0),
                    (color.z + brighten).min(1.0),
                    color.w,
                );
            }

            // Skip if radius is too small
            if radius < 0.5 {
                continue;
            }

            // Draw circle
            self.draw_point.color = color;
            self.draw_point.draw_abs(
                cx,
                Rect {
                    pos: dvec2(center_x - radius, center_y - radius),
                    size: dvec2(radius * 2.0, radius * 2.0),
                },
            );

            // Draw label (name split by CamelCase) - only when animation mostly complete
            // Use stricter radius check to ensure text fits inside circle
            if radius > 25.0 && bubble_progress > 0.7 {
                let text_alpha = ((bubble_progress - 0.7) / 0.3).clamp(0.0, 1.0);
                let name = bubble.id.split('.').last().unwrap_or(&bubble.id);
                let words = split_camel_case(name);

                let line_height = 12.0;
                let total_height = words.len() as f64 * line_height;

                // Only draw text if it fits vertically within the circle
                if total_height < radius * 1.6 {
                    let mut y_offset = center_y - total_height / 2.0;

                    // Set text color with fade-in alpha
                    self.draw_text.color = vec4(0.0, 0.0, 0.0, text_alpha as f32);

                    for word in &words {
                        let text_width = word.len() as f64 * 6.5; // Slightly more conservative estimate
                        // Check text fits horizontally with some margin
                        if text_width < radius * 1.8 {
                            self.draw_text.draw_abs(
                                cx,
                                dvec2(center_x - text_width / 2.0, y_offset),
                                word,
                            );
                        }
                        y_offset += line_height;
                    }

                    // Draw value below name (only if there's room)
                    if radius > 35.0 && y_offset < center_y + radius - 10.0 {
                        let value_str = format!("{}", bubble.value as i64);
                        let value_width = value_str.len() as f64 * 6.5;
                        if value_width < radius * 1.8 {
                            self.draw_text.draw_abs(
                                cx,
                                dvec2(center_x - value_width / 2.0, y_offset),
                                &value_str,
                            );
                        }
                    }
                }
            }
        }

        DrawStep::done()
    }
}

impl PackedBubbleChart {
    fn initialize_data(&mut self) {
        if self.initialized {
            return;
        }
        self.initialized = true;

        // Sample hierarchical data (similar to flare dataset structure)
        let data = vec![
            ("analytics.cluster.AgglomerativeCluster", "analytics", 3938.0),
            ("analytics.cluster.CommunityStructure", "analytics", 3812.0),
            ("analytics.cluster.HierarchicalCluster", "analytics", 6714.0),
            ("analytics.cluster.MergeEdge", "analytics", 743.0),
            ("analytics.graph.BetweennessCentrality", "analytics", 3534.0),
            ("analytics.graph.LinkDistance", "analytics", 5731.0),
            ("analytics.graph.MaxFlowMinCut", "analytics", 7840.0),
            ("analytics.graph.ShortestPaths", "analytics", 5914.0),
            ("animate.Easing", "animate", 17010.0),
            ("animate.FunctionSequence", "animate", 5842.0),
            ("animate.Parallel", "animate", 5176.0),
            ("animate.Sequence", "animate", 5534.0),
            ("animate.Transition", "animate", 9201.0),
            ("animate.Transitioner", "animate", 19975.0),
            ("data.converters.DelimitedTextConverter", "data", 4294.0),
            ("data.converters.GraphMLConverter", "data", 9800.0),
            ("data.converters.JSONConverter", "data", 2220.0),
            ("data.DataField", "data", 1759.0),
            ("data.DataSchema", "data", 2165.0),
            ("data.DataSource", "data", 3331.0),
            ("display.DirtySprite", "display", 8833.0),
            ("display.LineSprite", "display", 1732.0),
            ("display.RectSprite", "display", 3623.0),
            ("display.TextSprite", "display", 10066.0),
            ("flex.FlareVis", "flex", 4116.0),
            ("physics.DragForce", "physics", 1082.0),
            ("physics.GravityForce", "physics", 1336.0),
            ("physics.IForce", "physics", 319.0),
            ("physics.NBodyForce", "physics", 10498.0),
            ("physics.Particle", "physics", 2822.0),
            ("physics.Simulation", "physics", 9983.0),
            ("physics.Spring", "physics", 2213.0),
            ("query.AggregateExpression", "query", 1616.0),
            ("query.And", "query", 1027.0),
            ("query.Arithmetic", "query", 3891.0),
            ("query.Average", "query", 891.0),
            ("query.BinaryExpression", "query", 2893.0),
            ("query.Comparison", "query", 5103.0),
            ("scale.LinearScale", "scale", 1316.0),
            ("scale.LogScale", "scale", 3151.0),
            ("scale.OrdinalScale", "scale", 3770.0),
            ("scale.QuantileScale", "scale", 2435.0),
            ("scale.QuantitativeScale", "scale", 4839.0),
            ("scale.RootScale", "scale", 1756.0),
            ("scale.Scale", "scale", 4268.0),
            ("scale.TimeScale", "scale", 5833.0),
            ("util.Colors", "util", 10001.0),
            ("util.Dates", "util", 8217.0),
            ("util.Displays", "util", 12555.0),
            ("util.Filter", "util", 2324.0),
            ("util.Geometry", "util", 10993.0),
            ("util.heap.FibonacciHeap", "util", 9354.0),
            ("util.heap.HeapNode", "util", 1233.0),
            ("util.math.DenseMatrix", "util", 3165.0),
            ("util.math.IMatrix", "util", 2815.0),
            ("util.math.SparseMatrix", "util", 3366.0),
            ("vis.Visualization", "vis", 16540.0),
        ];

        // Build hierarchy
        let mut root = HierarchyNode::new(PackNode {
            id: "root".to_string(),
            value: 0.0,
        }, 0.0);

        for (id, group, value) in data {
            root.add_child(HierarchyNode::new(
                PackNode {
                    id: id.to_string(),
                    value,
                },
                value,
            ));
        }

        // Apply pack layout
        let layout = PackLayout::new()
            .size(928.0, 928.0)
            .padding(3.0);

        let positioned = layout.layout(&root);

        // Color scheme (Tableau10 colors)
        let color_map = create_color_map();

        // Extract leaf nodes
        self.bubbles.clear();
        for node in positioned.iter() {
            if node.children.is_empty() {
                // This is a leaf node
                let group = node.data.id.split('.').nth(0).unwrap_or("other");
                let color = color_map.get(group).copied().unwrap_or(vec4(0.5, 0.5, 0.5, 1.0));

                self.bubbles.push(BubbleItem {
                    id: node.data.id.clone(),
                    group: group.to_string(),
                    value: node.data.value,
                    x: node.x / 928.0,
                    y: node.y / 928.0,
                    radius: node.radius / 928.0,
                    color,
                });
            }
        }
    }

    fn find_bubble_at(&self, pos: DVec2) -> Option<usize> {
        if self.chart_size <= 0.0 {
            return None;
        }

        // Check bubbles in reverse order (top-most first, which are drawn last)
        for (i, bubble) in self.bubbles.iter().enumerate().rev() {
            let center_x = self.offset_x + bubble.x * self.chart_size;
            let center_y = self.offset_y + bubble.y * self.chart_size;
            let radius = bubble.radius * self.chart_size;

            let dx = pos.x - center_x;
            let dy = pos.y - center_y;
            let dist_sq = dx * dx + dy * dy;

            if dist_sq <= radius * radius {
                return Some(i);
            }
        }

        None
    }

    /// Run collision physics to push overlapping bubbles apart
    /// Returns true if any bubble moved significantly
    fn run_collision_physics(&mut self) -> bool {
        if self.bubbles.is_empty() {
            return false;
        }

        // Initialize velocities if needed
        if self.bubble_velocities.len() != self.bubbles.len() {
            self.bubble_velocities = vec![dvec2(0.0, 0.0); self.bubbles.len()];
        }

        let n = self.bubbles.len();
        let mut has_motion = false;

        // Calculate collision forces between all pairs
        for i in 0..n {
            for j in (i + 1)..n {
                let dx = self.bubbles[j].x - self.bubbles[i].x;
                let dy = self.bubbles[j].y - self.bubbles[i].y;
                let dist = (dx * dx + dy * dy).sqrt();
                let min_dist = self.bubbles[i].radius + self.bubbles[j].radius;

                // Check for overlap
                if dist < min_dist && dist > 0.0001 {
                    let overlap = min_dist - dist;
                    let nx = dx / dist;
                    let ny = dy / dist;

                    // Push strength based on overlap
                    let push = overlap * COLLISION_STRENGTH * 0.5;

                    // Skip the dragged bubble - don't apply forces to it
                    if self.dragging_index != Some(i) {
                        self.bubble_velocities[i].x -= nx * push;
                        self.bubble_velocities[i].y -= ny * push;
                    }
                    if self.dragging_index != Some(j) {
                        self.bubble_velocities[j].x += nx * push;
                        self.bubble_velocities[j].y += ny * push;
                    }
                }
            }
        }

        // Apply velocities and damping
        for i in 0..n {
            // Skip dragged bubble
            if self.dragging_index == Some(i) {
                continue;
            }

            let v = &self.bubble_velocities[i];
            let speed = (v.x * v.x + v.y * v.y).sqrt();

            if speed > 0.0001 {
                // Apply velocity to position
                self.bubbles[i].x += v.x;
                self.bubbles[i].y += v.y;

                // Clamp to boundaries
                let r = self.bubbles[i].radius;
                self.bubbles[i].x = self.bubbles[i].x.clamp(r + BOUNDARY_PADDING, 1.0 - r - BOUNDARY_PADDING);
                self.bubbles[i].y = self.bubbles[i].y.clamp(r + BOUNDARY_PADDING, 1.0 - r - BOUNDARY_PADDING);

                // Apply damping
                self.bubble_velocities[i].x *= VELOCITY_DAMPING;
                self.bubble_velocities[i].y *= VELOCITY_DAMPING;

                has_motion = true;
            }
        }

        has_motion
    }
}

fn split_camel_case(s: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();

    for ch in s.chars() {
        if ch.is_uppercase() && !current.is_empty() {
            result.push(current.clone());
            current.clear();
        }
        current.push(ch);
    }

    if !current.is_empty() {
        result.push(current);
    }

    if result.is_empty() {
        vec![s.to_string()]
    } else {
        result
    }
}

fn create_color_map() -> std::collections::HashMap<&'static str, Vec4> {
    let mut map = std::collections::HashMap::new();

    // Tableau10 color scheme
    map.insert("analytics", vec4(0.26, 0.45, 0.71, 1.0));  // Blue
    map.insert("animate", vec4(1.0, 0.50, 0.05, 1.0));     // Orange
    map.insert("data", vec4(0.17, 0.63, 0.17, 1.0));       // Green
    map.insert("display", vec4(0.84, 0.15, 0.16, 1.0));    // Red
    map.insert("flex", vec4(0.58, 0.40, 0.74, 1.0));       // Purple
    map.insert("physics", vec4(0.55, 0.34, 0.29, 1.0));    // Brown
    map.insert("query", vec4(0.89, 0.47, 0.76, 1.0));      // Pink
    map.insert("scale", vec4(0.50, 0.50, 0.50, 1.0));      // Gray
    map.insert("util", vec4(0.74, 0.74, 0.13, 1.0));       // Yellow
    map.insert("vis", vec4(0.09, 0.75, 0.81, 1.0));        // Cyan

    map
}

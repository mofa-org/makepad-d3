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
}

impl Widget for PackedBubbleChart {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        match event {
            Event::NextFrame(nf) => {
                // Start animation if not yet started
                if self.initialized && !self.animation_started {
                    self.animator = ChartAnimator::new(1200.0)
                        .with_easing(EasingType::EaseOutElastic);
                    self.animator.start(nf.time);
                    self.animation_started = true;
                    self.redraw(cx);
                    cx.new_next_frame();
                }

                // Continue animation if running
                if self.animation_started && self.animator.update(nf.time) {
                    self.redraw(cx);
                    cx.new_next_frame();
                }
            }
            Event::MouseMove(e) => {
                let old_hovered = self.hovered_index;
                self.hovered_index = self.find_bubble_at(e.abs);

                if old_hovered != self.hovered_index {
                    self.redraw(cx);
                }
            }
            Event::MouseLeave(_) => {
                if self.hovered_index.is_some() {
                    self.hovered_index = None;
                    self.redraw(cx);
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

        // Draw all bubbles with staggered animation
        let bubble_count = self.bubbles.len();
        for (i, bubble) in self.bubbles.iter().enumerate() {
            // Stagger animation: each bubble starts slightly later
            let stagger_delay = (i as f64) / (bubble_count as f64) * 0.3;
            let bubble_progress = ((base_progress - stagger_delay) / (1.0 - stagger_delay * 0.5))
                .clamp(0.0, 1.0);

            let center_x = self.offset_x + bubble.x * self.chart_size;
            let center_y = self.offset_y + bubble.y * self.chart_size;
            let target_radius = bubble.radius * self.chart_size;

            // Animate radius from 0 to target
            let mut radius = target_radius * bubble_progress;

            // Check if this bubble is hovered
            let is_hovered = self.hovered_index == Some(i);

            // Apply hover effect: scale up and brighten
            let mut color = bubble.color;
            if is_hovered && bubble_progress > 0.5 {
                radius *= 1.08; // Scale up 8%
                color = vec4(
                    (color.x + 0.15).min(1.0),
                    (color.y + 0.15).min(1.0),
                    (color.z + 0.15).min(1.0),
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
            if radius > 15.0 && bubble_progress > 0.7 {
                let text_alpha = ((bubble_progress - 0.7) / 0.3).clamp(0.0, 1.0);
                let name = bubble.id.split('.').last().unwrap_or(&bubble.id);
                let words = split_camel_case(name);

                let line_height = 12.0;
                let total_height = words.len() as f64 * line_height;
                let mut y_offset = center_y - total_height / 2.0;

                // Set text color with fade-in alpha (darker on hover)
                let text_brightness = if is_hovered { 0.0 } else { 0.0 };
                self.draw_text.color = vec4(text_brightness, text_brightness, text_brightness, text_alpha as f32);

                for word in &words {
                    let text_width = word.len() as f64 * 6.0; // Rough estimate
                    if text_width < radius * 2.0 {
                        self.draw_text.draw_abs(
                            cx,
                            dvec2(center_x - text_width / 2.0, y_offset),
                            word,
                        );
                    }
                    y_offset += line_height;
                }

                // Draw value below name
                if radius > 25.0 {
                    let value_str = format!("{}", bubble.value as i64);
                    let value_width = value_str.len() as f64 * 6.0;
                    self.draw_text.draw_abs(
                        cx,
                        dvec2(center_x - value_width / 2.0, y_offset),
                        &value_str,
                    );
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

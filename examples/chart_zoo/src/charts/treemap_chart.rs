//! Treemap Widget
//!
//! GPU-accelerated hierarchical visualization with animated reveal.
//! Features labels inside cells, multiple tiling methods, and staggered depth animation.

use makepad_widgets::*;
use makepad_d3::layout::hierarchy::{HierarchyNode, TreemapLayout, TilingMethod};
use super::draw_primitives::DrawTriangle;
use super::animation::{ChartAnimator, EasingType};

live_design! {
    link widgets;
    use link::shaders::*;
    use super::draw_primitives::DrawTriangle;

    pub TreemapWidget = {{TreemapWidget}} {
        width: Fill,
        height: Fill,
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct TreemapWidget {
    #[redraw]
    #[live]
    draw_rect: DrawTriangle,

    #[walk]
    walk: Walk,

    #[rust]
    tree: Option<HierarchyNode<TreeNode>>,

    #[rust]
    positioned_tree: Option<HierarchyNode<TreeNode>>,

    #[rust]
    colors: Vec<Vec4>,

    #[rust]
    animator: ChartAnimator,

    #[rust]
    initialized: bool,

    #[rust]
    area: Area,

    #[rust]
    chart_rect: Rect,

    #[rust]
    offset_x: f64,

    #[rust]
    offset_y: f64,

    #[rust]
    hovered_node: Option<(f64, f64, f64, f64)>,

    #[rust(true)]
    show_borders: bool,

    #[rust(true)]
    rounded_corners: bool,

    #[rust]
    node_count: usize,

    /// Show labels inside cells
    #[rust(true)]
    show_labels: bool,

    /// Fill opacity for cells (D3 default is 0.6)
    #[rust(0.6)]
    fill_opacity: f32,

    /// Tiling method to use
    #[rust]
    tiling_method: TilingMethod,
}

#[derive(Clone)]
struct TreeNode {
    name: String,
    color_index: usize,
    depth: usize,
    leaf_index: usize,
    value: f64,
}

impl Widget for TreemapWidget {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        match event {
            Event::NextFrame(_) => {
                if self.animator.is_running() {
                    let time = cx.seconds_since_app_start();
                    if self.animator.update(time) {
                        self.redraw(cx);
                    }
                    cx.new_next_frame();
                }
            }
            Event::MouseMove(e) => {
                self.handle_mouse_move(cx, e.abs);
            }
            Event::WindowGeomChange(_) => {
                self.redraw(cx);
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let rect = cx.walk_turtle_with_area(&mut self.area, walk);

        if rect.size.x > 0.0 && rect.size.y > 0.0 {
            self.chart_rect = rect;

            // Only draw if initialized - initialization must be done externally
            // to avoid stack overflow when many treemap widgets are used
            if self.initialized && self.tree.is_some() {
                self.draw_treemap(cx, rect);
            }
        }

        DrawStep::done()
    }
}

impl TreemapWidget {
    fn initialize_data(&mut self) {
        // Create a hierarchical dataset (file system-like structure)
        let mut root = HierarchyNode::new(
            TreeNode { name: "root".to_string(), color_index: 0, depth: 0, leaf_index: 0, value: 0.0 },
            0.0,
        );

        // Category 1: Technology
        let mut tech = HierarchyNode::new(
            TreeNode { name: "Technology".to_string(), color_index: 0, depth: 1, leaf_index: 0, value: 0.0 },
            0.0,
        );
        tech.add_child(HierarchyNode::new(
            TreeNode { name: "Software".to_string(), color_index: 0, depth: 2, leaf_index: 0, value: 120.0 },
            120.0,
        ));
        tech.add_child(HierarchyNode::new(
            TreeNode { name: "Hardware".to_string(), color_index: 0, depth: 2, leaf_index: 1, value: 80.0 },
            80.0,
        ));
        tech.add_child(HierarchyNode::new(
            TreeNode { name: "Cloud".to_string(), color_index: 0, depth: 2, leaf_index: 2, value: 95.0 },
            95.0,
        ));
        tech.add_child(HierarchyNode::new(
            TreeNode { name: "AI/ML".to_string(), color_index: 0, depth: 2, leaf_index: 3, value: 150.0 },
            150.0,
        ));

        // Category 2: Healthcare
        let mut health = HierarchyNode::new(
            TreeNode { name: "Healthcare".to_string(), color_index: 1, depth: 1, leaf_index: 0, value: 0.0 },
            0.0,
        );
        health.add_child(HierarchyNode::new(
            TreeNode { name: "Pharma".to_string(), color_index: 1, depth: 2, leaf_index: 4, value: 90.0 },
            90.0,
        ));
        health.add_child(HierarchyNode::new(
            TreeNode { name: "Biotech".to_string(), color_index: 1, depth: 2, leaf_index: 5, value: 70.0 },
            70.0,
        ));
        health.add_child(HierarchyNode::new(
            TreeNode { name: "Devices".to_string(), color_index: 1, depth: 2, leaf_index: 6, value: 45.0 },
            45.0,
        ));

        // Category 3: Finance
        let mut finance = HierarchyNode::new(
            TreeNode { name: "Finance".to_string(), color_index: 2, depth: 1, leaf_index: 0, value: 0.0 },
            0.0,
        );
        finance.add_child(HierarchyNode::new(
            TreeNode { name: "Banking".to_string(), color_index: 2, depth: 2, leaf_index: 7, value: 100.0 },
            100.0,
        ));
        finance.add_child(HierarchyNode::new(
            TreeNode { name: "Insurance".to_string(), color_index: 2, depth: 2, leaf_index: 8, value: 60.0 },
            60.0,
        ));
        finance.add_child(HierarchyNode::new(
            TreeNode { name: "Fintech".to_string(), color_index: 2, depth: 2, leaf_index: 9, value: 85.0 },
            85.0,
        ));

        // Category 4: Energy
        let mut energy = HierarchyNode::new(
            TreeNode { name: "Energy".to_string(), color_index: 3, depth: 1, leaf_index: 0, value: 0.0 },
            0.0,
        );
        energy.add_child(HierarchyNode::new(
            TreeNode { name: "Solar".to_string(), color_index: 3, depth: 2, leaf_index: 10, value: 55.0 },
            55.0,
        ));
        energy.add_child(HierarchyNode::new(
            TreeNode { name: "Wind".to_string(), color_index: 3, depth: 2, leaf_index: 11, value: 40.0 },
            40.0,
        ));
        energy.add_child(HierarchyNode::new(
            TreeNode { name: "Oil & Gas".to_string(), color_index: 3, depth: 2, leaf_index: 12, value: 75.0 },
            75.0,
        ));

        // Category 5: Consumer
        let mut consumer = HierarchyNode::new(
            TreeNode { name: "Consumer".to_string(), color_index: 4, depth: 1, leaf_index: 0, value: 0.0 },
            0.0,
        );
        consumer.add_child(HierarchyNode::new(
            TreeNode { name: "Retail".to_string(), color_index: 4, depth: 2, leaf_index: 13, value: 65.0 },
            65.0,
        ));
        consumer.add_child(HierarchyNode::new(
            TreeNode { name: "Media".to_string(), color_index: 4, depth: 2, leaf_index: 14, value: 50.0 },
            50.0,
        ));

        root.add_child(tech);
        root.add_child(health);
        root.add_child(finance);
        root.add_child(energy);
        root.add_child(consumer);

        self.tree = Some(root);
        self.node_count = 15; // Total leaf nodes

        // Vibrant color palette
        self.colors = vec![
            vec4(0.26, 0.52, 0.96, 1.0), // Blue (Tech)
            vec4(0.20, 0.78, 0.50, 1.0), // Green (Health)
            vec4(1.0, 0.76, 0.03, 1.0),  // Gold (Finance)
            vec4(0.92, 0.36, 0.32, 1.0), // Red (Energy)
            vec4(0.61, 0.35, 0.80, 1.0), // Purple (Consumer)
        ];
    }

    fn start_animation(&mut self, cx: &mut Cx) {
        let time = cx.seconds_since_app_start();
        self.animator = ChartAnimator::new(1400.0)
            .with_easing(EasingType::EaseOutCubic);
        self.animator.start(time);
        cx.new_next_frame();
    }

    pub fn replay_animation(&mut self, cx: &mut Cx) {
        self.animator.reset();
        self.start_animation(cx);
        self.redraw(cx);
    }

    fn draw_treemap(&mut self, cx: &mut Cx2d, rect: Rect) {
        let tree = match &self.tree {
            Some(t) => t,
            None => return,
        };

        let padding = 10.0;
        self.offset_x = rect.pos.x + padding;
        self.offset_y = rect.pos.y + padding;

        let layout = TreemapLayout::new()
            .size(
                (rect.size.x - padding * 2.0) as f64,
                (rect.size.y - padding * 2.0) as f64,
            )
            .padding(1.0)
            .padding_top(0.0)
            .tiling(self.tiling_method.clone());

        let positioned = layout.layout(tree);
        self.positioned_tree = Some(positioned.clone());

        let progress = self.animator.get_progress();

        // Draw background
        self.draw_rect.color = vec4(0.15, 0.15, 0.18, 1.0);
        self.draw_rect.disable_gradient();
        let bg_p1 = dvec2(self.offset_x - 5.0, self.offset_y - 5.0);
        let bg_p2 = dvec2(self.offset_x + rect.size.x - padding * 2.0 + 5.0, self.offset_y - 5.0);
        let bg_p3 = dvec2(self.offset_x + rect.size.x - padding * 2.0 + 5.0, self.offset_y + rect.size.y - padding * 2.0 + 5.0);
        let bg_p4 = dvec2(self.offset_x - 5.0, self.offset_y + rect.size.y - padding * 2.0 + 5.0);
        self.draw_rect.draw_triangle(cx, bg_p1, bg_p2, bg_p3);
        self.draw_rect.draw_triangle(cx, bg_p1, bg_p3, bg_p4);

        // Draw all leaf nodes with animation
        self.draw_node(cx, &positioned, self.offset_x, self.offset_y, progress);
    }

    fn draw_node(&mut self, cx: &mut Cx2d, root: &HierarchyNode<TreeNode>, offset_x: f64, offset_y: f64, progress: f64) {
        // Use iterative approach with explicit stack to avoid stack overflow
        let mut stack: Vec<&HierarchyNode<TreeNode>> = vec![root];

        while let Some(node) = stack.pop() {
            if node.is_leaf() && node.width > 0.0 && node.rect_height > 0.0 {
                // Stagger animation by leaf index
                let leaf_idx = node.data.leaf_index;
                let node_delay = leaf_idx as f64 * 0.04;
                let node_progress = ((progress - node_delay) / 0.5).clamp(0.0, 1.0);

                if node_progress > 0.0 {
                    let color_idx = node.data.color_index % self.colors.len();
                    let mut base_color = self.colors[color_idx];
                    // Apply fill opacity
                    base_color.w = self.fill_opacity;

                    // Calculate node rectangle
                    let full_x = offset_x + node.x;
                    let full_y = offset_y + node.y;
                    let full_w = node.width;
                    let full_h = node.rect_height;

                    // Animate from center
                    let w = full_w * node_progress;
                    let h = full_h * node_progress;
                    let x = full_x + (full_w - w) / 2.0;
                    let y = full_y + (full_h - h) / 2.0;

                    // Check if this node is hovered
                    let is_hovered = self.hovered_node.map_or(false, |hover| {
                        (hover.0 - full_x).abs() < 0.1 &&
                        (hover.1 - full_y).abs() < 0.1 &&
                        (hover.2 - full_w).abs() < 0.1 &&
                        (hover.3 - full_h).abs() < 0.1
                    });

                    // Brighten hovered node
                    if is_hovered {
                        base_color = vec4(
                            (base_color.x + 0.18).min(1.0),
                            (base_color.y + 0.18).min(1.0),
                            (base_color.z + 0.18).min(1.0),
                            base_color.w,
                        );
                    }

                    // Draw the rectangle
                    self.draw_cell(cx, x, y, w, h, base_color, node_progress, is_hovered);
                }
            }

            // Add children to stack (in reverse order for correct traversal)
            for child in node.children.iter().rev() {
                stack.push(child);
            }
        }
    }

    fn draw_cell_label(&mut self, _cx: &mut Cx2d, _x: f64, _y: f64, _w: f64, _h: f64, _name: &str, _value: f64) {
        // Labels disabled - DrawText causes stack overflow
    }

    fn draw_cell(
        &mut self,
        cx: &mut Cx2d,
        x: f64, y: f64,
        w: f64, h: f64,
        color: Vec4,
        _progress: f64,
        is_hovered: bool,
    ) {
        // Flat solid color
        self.draw_rect.color = color;
        self.draw_rect.disable_gradient();

        let p1 = dvec2(x, y);
        let p2 = dvec2(x + w, y);
        let p3 = dvec2(x + w, y + h);
        let p4 = dvec2(x, y + h);

        self.draw_rect.draw_triangle(cx, p1, p2, p3);
        self.draw_rect.draw_triangle(cx, p1, p3, p4);

        // Draw borders using thin rectangles
        if self.show_borders {
            let border_color = if is_hovered {
                vec4(1.0, 1.0, 1.0, 0.8)
            } else {
                vec4(0.1, 0.1, 0.12, 0.8)
            };
            let bw = if is_hovered { 2.0 } else { 1.0 };

            self.draw_rect.color = border_color;
            // Top border
            self.draw_rect.draw_triangle(cx, p1, p2, dvec2(p2.x, p2.y + bw));
            self.draw_rect.draw_triangle(cx, p1, dvec2(p2.x, p2.y + bw), dvec2(p1.x, p1.y + bw));
            // Right border
            self.draw_rect.draw_triangle(cx, p2, p3, dvec2(p3.x - bw, p3.y));
            self.draw_rect.draw_triangle(cx, p2, dvec2(p3.x - bw, p3.y), dvec2(p2.x - bw, p2.y));
            // Bottom border
            self.draw_rect.draw_triangle(cx, p3, p4, dvec2(p4.x, p4.y - bw));
            self.draw_rect.draw_triangle(cx, p3, dvec2(p4.x, p4.y - bw), dvec2(p3.x, p3.y - bw));
            // Left border
            self.draw_rect.draw_triangle(cx, p4, p1, dvec2(p1.x + bw, p1.y));
            self.draw_rect.draw_triangle(cx, p4, dvec2(p1.x + bw, p1.y), dvec2(p4.x + bw, p4.y));
        }
    }

    fn handle_mouse_move(&mut self, cx: &mut Cx, pos: DVec2) {
        let old_hovered = self.hovered_node;
        self.hovered_node = self.find_node_at(pos);

        if old_hovered != self.hovered_node {
            self.redraw(cx);
        }
    }

    fn find_node_at(&self, pos: DVec2) -> Option<(f64, f64, f64, f64)> {
        let tree = self.positioned_tree.as_ref()?;
        self.find_leaf_at(tree, pos, self.offset_x, self.offset_y)
    }

    fn find_leaf_at(&self, root: &HierarchyNode<TreeNode>, pos: DVec2, offset_x: f64, offset_y: f64) -> Option<(f64, f64, f64, f64)> {
        // Use iterative approach to avoid stack overflow
        let mut stack: Vec<&HierarchyNode<TreeNode>> = vec![root];

        while let Some(node) = stack.pop() {
            if node.is_leaf() && node.width > 0.0 && node.rect_height > 0.0 {
                let x = offset_x + node.x;
                let y = offset_y + node.y;
                let w = node.width;
                let h = node.rect_height;

                if pos.x >= x && pos.x <= x + w && pos.y >= y && pos.y <= y + h {
                    return Some((x, y, w, h));
                }
            }

            for child in &node.children {
                stack.push(child);
            }
        }
        None
    }

    /// Initialize with D3 flare-style software package hierarchy data
    pub fn initialize_flare_data(&mut self) {
        self.initialize_flare_with_tiling(TilingMethod::Binary);
    }

    /// Initialize with flare data and specific tiling method
    pub fn initialize_flare_with_tiling(&mut self, tiling: TilingMethod) {
        let mut root = HierarchyNode::new(
            TreeNode { name: "flare".to_string(), color_index: 0, depth: 0, leaf_index: 0, value: 0.0 },
            0.0,
        );

        let mut leaf_idx = 0usize;

        // Helper to create leaf node
        let leaf = |name: &str, color: usize, value: f64, idx: &mut usize| {
            *idx += 1;
            HierarchyNode::new(
                TreeNode { name: name.to_string(), color_index: color, depth: 2, leaf_index: *idx - 1, value },
                value,
            )
        };

        // analytics
        let mut analytics = HierarchyNode::new(
            TreeNode { name: "analytics".to_string(), color_index: 0, depth: 1, leaf_index: 0, value: 0.0 },
            0.0,
        );
        analytics.add_child(leaf("AgglomerativeCluster", 0, 3938.0, &mut leaf_idx));
        analytics.add_child(leaf("CommunityStructure", 0, 3812.0, &mut leaf_idx));
        analytics.add_child(leaf("HierarchicalCluster", 0, 6714.0, &mut leaf_idx));
        analytics.add_child(leaf("MergeEdge", 0, 743.0, &mut leaf_idx));
        root.add_child(analytics);

        // animate
        let mut animate = HierarchyNode::new(
            TreeNode { name: "animate".to_string(), color_index: 1, depth: 1, leaf_index: 0, value: 0.0 },
            0.0,
        );
        animate.add_child(leaf("Easing", 1, 17010.0, &mut leaf_idx));
        animate.add_child(leaf("FunctionSequence", 1, 5842.0, &mut leaf_idx));
        animate.add_child(leaf("Tween", 1, 6006.0, &mut leaf_idx));
        animate.add_child(leaf("Transitioner", 1, 19975.0, &mut leaf_idx));
        root.add_child(animate);

        // data
        let mut data = HierarchyNode::new(
            TreeNode { name: "data".to_string(), color_index: 2, depth: 1, leaf_index: 0, value: 0.0 },
            0.0,
        );
        data.add_child(leaf("DataField", 2, 1759.0, &mut leaf_idx));
        data.add_child(leaf("DataSchema", 2, 2165.0, &mut leaf_idx));
        data.add_child(leaf("DataSet", 2, 586.0, &mut leaf_idx));
        data.add_child(leaf("DataSource", 2, 3331.0, &mut leaf_idx));
        data.add_child(leaf("DataUtil", 2, 3322.0, &mut leaf_idx));
        root.add_child(data);

        // display
        let mut display = HierarchyNode::new(
            TreeNode { name: "display".to_string(), color_index: 3, depth: 1, leaf_index: 0, value: 0.0 },
            0.0,
        );
        display.add_child(leaf("DirtySprite", 3, 8833.0, &mut leaf_idx));
        display.add_child(leaf("LineSprite", 3, 1732.0, &mut leaf_idx));
        display.add_child(leaf("RectSprite", 3, 3623.0, &mut leaf_idx));
        display.add_child(leaf("TextSprite", 3, 10066.0, &mut leaf_idx));
        root.add_child(display);

        // scale
        let mut scale = HierarchyNode::new(
            TreeNode { name: "scale".to_string(), color_index: 4, depth: 1, leaf_index: 0, value: 0.0 },
            0.0,
        );
        scale.add_child(leaf("LinearScale", 4, 1316.0, &mut leaf_idx));
        scale.add_child(leaf("LogScale", 4, 3151.0, &mut leaf_idx));
        scale.add_child(leaf("OrdinalScale", 4, 3770.0, &mut leaf_idx));
        scale.add_child(leaf("TimeScale", 4, 5833.0, &mut leaf_idx));
        root.add_child(scale);

        // vis
        let mut vis = HierarchyNode::new(
            TreeNode { name: "vis".to_string(), color_index: 5, depth: 1, leaf_index: 0, value: 0.0 },
            0.0,
        );
        vis.add_child(leaf("ScaleBinding", 5, 11275.0, &mut leaf_idx));
        vis.add_child(leaf("Tree", 5, 7147.0, &mut leaf_idx));
        vis.add_child(leaf("TreeBuilder", 5, 9930.0, &mut leaf_idx));
        root.add_child(vis);

        self.tree = Some(root);
        self.node_count = leaf_idx;
        self.tiling_method = tiling;

        // D3 schemeTableau10 colors
        self.colors = vec![
            vec4(0.122, 0.467, 0.706, 1.0), // Blue - analytics
            vec4(1.0, 0.498, 0.055, 1.0),   // Orange - animate
            vec4(0.173, 0.627, 0.173, 1.0), // Green - data
            vec4(0.839, 0.153, 0.157, 1.0), // Red - display
            vec4(0.580, 0.404, 0.741, 1.0), // Purple - scale
            vec4(0.549, 0.337, 0.294, 1.0), // Brown - vis
        ];
        self.initialized = true;
    }

    /// Set tiling method
    pub fn set_tiling_method(&mut self, method: TilingMethod) {
        self.tiling_method = method;
    }

    /// Set whether to show labels
    pub fn set_show_labels(&mut self, show: bool) {
        self.show_labels = show;
    }
}

/// Split a camelCase string into parts (like D3's split on /(?=[A-Z][a-z])|\s+/g)
fn split_camel_case(s: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let chars: Vec<char> = s.chars().collect();

    for i in 1..chars.len() {
        // Split before uppercase letter followed by lowercase
        if chars[i].is_uppercase() && i + 1 < chars.len() && chars[i + 1].is_lowercase() {
            if start < i {
                parts.push(&s[start..i]);
            }
            start = i;
        }
        // Also split on whitespace
        if chars[i].is_whitespace() {
            if start < i {
                parts.push(&s[start..i]);
            }
            start = i + 1;
        }
    }

    if start < s.len() {
        parts.push(&s[start..]);
    }

    if parts.is_empty() {
        vec![s]
    } else {
        parts
    }
}

/// Format a number with comma separators (like D3's d3.format(",d"))
fn format_number(n: f64) -> String {
    let n = n as i64;
    let s = n.to_string();
    let chars: Vec<char> = s.chars().collect();
    let mut result = String::new();

    for (i, c) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(*c);
    }

    result
}

/// Widget reference implementation for external initialization
impl TreemapWidgetRef {
    /// Initialize with default demo data (Technology, Healthcare, Finance, Energy, Consumer)
    pub fn initialize_default(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.initialize_data();
            inner.start_animation(cx);
            inner.initialized = true;
        }
    }

    pub fn initialize_flare_data(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.initialize_flare_data();
            inner.replay_animation(cx);
        }
    }

    pub fn initialize_with_tiling(&self, cx: &mut Cx, tiling: TilingMethod) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.initialize_flare_with_tiling(tiling);
            inner.replay_animation(cx);
        }
    }
}

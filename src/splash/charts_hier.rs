//! Hierarchy chart widgets: `d3.Treemap`, `d3.Sunburst`, `d3.CirclePack`,
//! `d3.TreeChart`.
//!
//! All four consume the same declarative `data:` shape — a nested node
//! object mirroring d3's hierarchy convention:
//!
//! ```splash,ignore
//! data: {name: "root" children: [
//!     {name: "a" value: 8},
//!     {name: "b" children: [{name: "b1" value: 3}, {name: "b2" value: 5}]}
//! ]}
//! ```
//!
//! A flat array of numbers is accepted as a single-level shorthand.
//! Leaves are colored by their top-level branch (d3 Category10).

// The `script_mod!` and `derive(Script, Widget)` macros generate public
// registration items that cannot carry doc comments.
#![allow(missing_docs)]

use crate::color::CategoricalScale;
use crate::layout::hierarchy::{PartitionLayout, PartitionNode};
use crate::layout::{HierarchyNode, TreeLayout};

use super::charts::{begin_plot, compute_plot_rect, end_plot, fire_index_callback, rgba_to_vec4f};
use super::vm_data;
use makepad_widgets::makepad_script::ScriptFnRef;
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*

    mod.d3.TreemapBase = #(D3Treemap::register_widget(vm))
    mod.d3.Treemap = set_type_default() do mod.d3.TreemapBase{
        width: Fill
        height: 300
        plot_margin: Inset{left: 12.0, top: 12.0, right: 12.0, bottom: 12.0}
        draw_bg +: { draw_depth: 0.0 color: #x00000000 }
        draw_grid +: { draw_depth: 0.1 }
        draw_vector +: { draw_depth: 2.0 }
        draw_text +: { draw_depth: 3.0 }
    }

    mod.d3.SunburstBase = #(D3Sunburst::register_widget(vm))
    mod.d3.Sunburst = set_type_default() do mod.d3.SunburstBase{
        width: Fill
        height: 300
        plot_margin: Inset{left: 12.0, top: 12.0, right: 12.0, bottom: 12.0}
        draw_bg +: { draw_depth: 0.0 color: #x00000000 }
        draw_grid +: { draw_depth: 0.1 }
        draw_vector +: { draw_depth: 2.0 }
        draw_text +: { draw_depth: 3.0 }
    }

    mod.d3.CirclePackBase = #(D3CirclePack::register_widget(vm))
    mod.d3.CirclePack = set_type_default() do mod.d3.CirclePackBase{
        width: Fill
        height: 300
        plot_margin: Inset{left: 12.0, top: 12.0, right: 12.0, bottom: 12.0}
        branch_color: #x3a3f52
        draw_bg +: { draw_depth: 0.0 color: #x00000000 }
        draw_grid +: { draw_depth: 0.1 }
        draw_vector +: { draw_depth: 2.0 }
        draw_text +: { draw_depth: 3.0 }
    }

    mod.d3.TreeChartBase = #(D3TreeChart::register_widget(vm))
    mod.d3.TreeChart = set_type_default() do mod.d3.TreeChartBase{
        width: Fill
        height: 300
        plot_margin: Inset{left: 24.0, top: 16.0, right: 24.0, bottom: 16.0}
        link_color: #x5a6075
        node_color: #x1f77b4
        leaf_color: #x2ca02c
        node_radius: 4.0
        draw_bg +: { draw_depth: 0.0 color: #x00000000 }
        draw_grid +: { draw_depth: 0.1 }
        draw_vector +: { draw_depth: 2.0 }
        draw_text +: { draw_depth: 3.0 }
    }
}

// ---- Shared hierarchy parsing ----

/// Parse a script value into a hierarchy (see module docs for the shape).
pub(crate) fn parse_hierarchy(
    vm: &mut ScriptVm,
    value: ScriptValue,
) -> Option<HierarchyNode<String>> {
    fn parse_node(
        vm: &mut ScriptVm,
        value: ScriptValue,
        index: usize,
    ) -> Option<HierarchyNode<String>> {
        if let Some(v) = value.as_number() {
            return Some(HierarchyNode::leaf(format!("{index}"), v));
        }
        if value.as_object().is_some() {
            let name_v = vm_data::field(vm, value, live_id!(name));
            let name = if name_v.is_nil() {
                format!("{index}")
            } else {
                vm_data::to_string_cast(vm, name_v)
            };
            let children_v = vm_data::field(vm, value, live_id!(children));
            if !children_v.is_nil() {
                if let Some(items) = vm_data::elements(vm, children_v) {
                    let mut node = HierarchyNode::branch(name);
                    for (i, item) in items.iter().enumerate() {
                        if let Some(child) = parse_node(vm, *item, i) {
                            node.add_child(child);
                        }
                    }
                    return Some(node);
                }
            }
            let v = vm_data::field(vm, value, live_id!(value))
                .as_number()
                .unwrap_or(1.0);
            return Some(HierarchyNode::leaf(name, v));
        }
        None
    }

    if value.is_nil() {
        return None;
    }
    // Flat array of numbers => single-level hierarchy
    if let Some(items) = vm_data::elements(vm, value) {
        // If it's an object with children, parse_node handles it below;
        // a plain sequence becomes the root's children.
        if value.as_array().is_some() {
            let mut root = HierarchyNode::branch("root".to_string());
            for (i, item) in items.iter().enumerate() {
                if let Some(child) = parse_node(vm, *item, i) {
                    root.add_child(child);
                }
            }
            if root.child_count() == 0 {
                return None;
            }
            return Some(root);
        }
    }
    parse_node(vm, value, 0)
}

pub(crate) fn demo_hierarchy() -> HierarchyNode<String> {
    let mut root = HierarchyNode::branch("root".to_string());
    let groups: [(&str, &[f64]); 4] = [
        ("alpha", &[8.0, 5.0, 3.0]),
        ("beta", &[12.0, 4.0]),
        ("gamma", &[6.0, 6.0, 2.0, 2.0]),
        ("delta", &[10.0]),
    ];
    for (name, values) in groups {
        let mut branch = HierarchyNode::branch(name.to_string());
        for (i, &v) in values.iter().enumerate() {
            branch.add_child(HierarchyNode::leaf(format!("{name}-{i}"), v));
        }
        root.add_child(branch);
    }
    root
}

/// Collect leaves depth-first with their top-level branch index.
fn collect_leaves<'a>(
    node: &'a HierarchyNode<String>,
    top: usize,
    depth: usize,
    out: &mut Vec<(&'a HierarchyNode<String>, usize)>,
) {
    if node.is_leaf() {
        out.push((node, top));
        return;
    }
    for (i, child) in node.children.iter().enumerate() {
        let t = if depth == 0 { i } else { top };
        collect_leaves(child, t, depth + 1, out);
    }
}

fn branch_color(i: usize) -> Vec4f {
    rgba_to_vec4f(CategoricalScale::category10().get(i))
}

/// Classic squarified treemap tiling of `values` into `bounds`,
/// preserving input order in the output.
fn squarify(values: &[f64], bounds: Rect) -> Vec<Rect> {
    let total: f64 = values.iter().map(|v| v.max(0.0)).sum();
    let mut out = vec![Rect::default(); values.len()];
    if total <= 0.0 || values.is_empty() {
        return out;
    }
    // Areas in pixel² with original indices, largest first.
    let scale = bounds.size.x * bounds.size.y / total;
    let mut items: Vec<(usize, f64)> = values
        .iter()
        .enumerate()
        .map(|(i, v)| (i, v.max(0.0) * scale))
        .collect();
    items.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let worst = |row_sum: f64, min: f64, max: f64, side: f64| -> f64 {
        let s2 = row_sum * row_sum;
        let side2 = side * side;
        (side2 * max / s2).max(s2 / (side2 * min))
    };

    let (mut x, mut y) = (bounds.pos.x, bounds.pos.y);
    let (mut w, mut h) = (bounds.size.x, bounds.size.y);
    let mut i = 0;
    while i < items.len() {
        let side = w.min(h).max(1.0);
        // Grow the row while the worst aspect ratio improves.
        let mut row_end = i + 1;
        let mut row_sum = items[i].1;
        let mut row_min = items[i].1;
        let mut row_max = items[i].1;
        let mut best = worst(row_sum, row_min, row_max, side);
        while row_end < items.len() {
            let a = items[row_end].1;
            let cand = worst(row_sum + a, row_min.min(a), row_max.max(a), side);
            if cand > best {
                break;
            }
            row_sum += a;
            row_min = row_min.min(a);
            row_max = row_max.max(a);
            best = cand;
            row_end += 1;
        }
        // Lay the row along the shorter side.
        let thickness = (row_sum / side).min(w.max(h));
        let mut cursor = 0.0;
        for &(idx, area) in &items[i..row_end] {
            let len = if row_sum > 0.0 {
                area / row_sum * side
            } else {
                0.0
            };
            out[idx] = if w >= h {
                Rect {
                    pos: DVec2 { x, y: y + cursor },
                    size: DVec2 {
                        x: thickness.max(1.0),
                        y: len.max(1.0),
                    },
                }
            } else {
                Rect {
                    pos: DVec2 { x: x + cursor, y },
                    size: DVec2 {
                        x: len.max(1.0),
                        y: thickness.max(1.0),
                    },
                }
            };
            cursor += len;
        }
        if w >= h {
            x += thickness;
            w -= thickness;
        } else {
            y += thickness;
            h -= thickness;
        }
        i = row_end;
    }
    out
}

/// Greedy tangent bubble packing: circles sorted by size spiral outward
/// until collision-free. Returns (x, y) centers relative to (0, 0) plus the
/// radius actually used per input.
fn pack_bubbles(radii: &[f64]) -> Vec<(f64, f64)> {
    let mut placed: Vec<(f64, f64, f64)> = Vec::with_capacity(radii.len());
    let mut order: Vec<usize> = (0..radii.len()).collect();
    order.sort_by(|&a, &b| {
        radii[b]
            .partial_cmp(&radii[a])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut out = vec![(0.0, 0.0); radii.len()];
    for &idx in &order {
        let r = radii[idx];
        let mut best: Option<(f64, f64)> = None;
        if placed.is_empty() {
            best = Some((0.0, 0.0));
        } else {
            // Walk an Archimedean spiral until a free spot appears.
            let mut theta: f64 = 0.0;
            while best.is_none() && theta < 400.0 {
                let rad = theta * 0.55;
                let (cx, cy) = (rad * theta.cos(), rad * theta.sin());
                let ok = placed.iter().all(|&(px, py, pr)| {
                    ((px - cx).powi(2) + (py - cy).powi(2)).sqrt() >= pr + r - 0.5
                });
                if ok {
                    best = Some((cx, cy));
                }
                theta += 0.1;
            }
        }
        let (cx, cy) = best.unwrap_or((0.0, 0.0));
        placed.push((cx, cy, r));
        out[idx] = (cx, cy);
    }
    out
}

// ================= Treemap =================

/// Squarified treemap, exposed as `d3.Treemap`.
#[derive(Script, Widget)]
pub struct D3Treemap {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,

    #[redraw]
    #[live]
    draw_bg: DrawColor,
    #[live]
    draw_grid: DrawColor,
    #[live]
    draw_vector: DrawVector,
    #[live]
    draw_text: DrawText,

    /// Declarative data: nested hierarchy object.
    #[live]
    data: ScriptValue,
    /// Margins around the plot area.
    #[live]
    pub plot_margin: Inset,

    /// Fired with the leaf index (depth-first order) on click.
    #[live]
    on_click: ScriptFnRef,

    #[rust]
    root: Option<HierarchyNode<String>>,
    #[rust]
    rect: Rect,
    #[rust]
    plot: Rect,
    #[rust]
    leaf_rects: Vec<Rect>,
    #[rust]
    hovered: Option<usize>,
}

impl D3Treemap {
    /// Replace the hierarchy from Rust.
    pub fn set_root(&mut self, root: HierarchyNode<String>) {
        self.root = Some(root);
        self.hovered = None;
    }

    fn leaf_at(&self, abs: DVec2) -> Option<usize> {
        self.leaf_rects.iter().position(|r| r.contains(abs))
    }
}

impl ScriptHook for D3Treemap {
    fn on_after_apply(
        &mut self,
        vm: &mut ScriptVm,
        _apply: &Apply,
        _scope: &mut Scope,
        _value: ScriptValue,
    ) {
        if let Some(root) = parse_hierarchy(vm, self.data) {
            self.root = Some(root);
        }
    }
}

impl Widget for D3Treemap {
    fn script_call(
        &mut self,
        vm: &mut ScriptVm,
        method: LiveId,
        args: ScriptValue,
    ) -> ScriptAsyncResult {
        if method == live_id!(set_data) {
            let value = vm_data::arg(vm, args, 0);
            if let Some(root) = parse_hierarchy(vm, value) {
                self.set_root(root);
                vm.with_cx_mut(|cx| self.redraw(cx));
            }
            return ScriptAsyncResult::Return(NIL);
        }
        ScriptAsyncResult::MethodNotFound
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        match event.hits(cx, self.draw_bg.area()) {
            Hit::FingerHoverIn(fe) | Hit::FingerHoverOver(fe) => {
                let hit = self.leaf_at(fe.abs);
                if hit != self.hovered {
                    self.hovered = hit;
                    self.redraw(cx);
                }
            }
            Hit::FingerHoverOut(_) if self.hovered.is_some() => {
                self.hovered = None;
                self.redraw(cx);
            }
            Hit::FingerUp(fe) if fe.is_over && fe.is_primary_hit() => {
                if let Some(i) = self.leaf_at(fe.abs) {
                    fire_index_callback(cx, self.uid, &self.source, &self.on_click, i);
                }
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.root.is_none() {
            self.root = Some(demo_hierarchy());
        }
        self.rect = cx.walk_turtle(walk);
        self.plot = compute_plot_rect(&self.rect, &self.plot_margin);
        self.draw_bg.draw_abs(cx, self.rect);

        begin_plot(cx, &self.rect, &self.plot_margin, &mut self.draw_vector);

        let mut root = self.root.clone().unwrap();
        root.sum();
        // Flatten to leaf-level tiling (branch identity kept for color) and
        // tile with an inline squarify — the classic d3 treemap look.
        let mut src_leaves = Vec::new();
        collect_leaves(&root, 0, 0, &mut src_leaves);
        let tops: Vec<usize> = src_leaves.iter().map(|(_, top)| *top).collect();
        let values: Vec<f64> = src_leaves.iter().map(|(l, _)| l.value.max(0.0)).collect();
        let rects = squarify(&values, self.plot);
        self.leaf_rects.clear();

        for (i, r) in rects.iter().enumerate() {
            let r = *r;
            self.leaf_rects.push(r);
            let mut color = branch_color(tops[i]);
            if self.hovered == Some(i) {
                color.x = (color.x * 1.25).min(1.0);
                color.y = (color.y * 1.25).min(1.0);
                color.z = (color.z * 1.25).min(1.0);
            }
            self.draw_vector
                .set_color(color.x, color.y, color.z, color.w);
            self.draw_vector.rect(
                r.pos.x as f32,
                r.pos.y as f32,
                r.size.x as f32,
                r.size.y as f32,
            );
            self.draw_vector.fill();
        }

        end_plot(cx, &mut self.draw_vector);
        DrawStep::done()
    }
}

// ================= Sunburst =================

/// Radial partition (sunburst), exposed as `d3.Sunburst`.
#[derive(Script, Widget)]
pub struct D3Sunburst {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,

    #[redraw]
    #[live]
    draw_bg: DrawColor,
    #[live]
    draw_grid: DrawColor,
    #[live]
    draw_vector: DrawVector,
    #[live]
    draw_text: DrawText,

    /// Declarative data: nested hierarchy object.
    #[live]
    data: ScriptValue,
    /// Margins around the plot area.
    #[live]
    pub plot_margin: Inset,

    #[rust]
    root: Option<HierarchyNode<String>>,
    #[rust]
    rect: Rect,
    #[rust]
    plot: Rect,
}

impl D3Sunburst {
    /// Replace the hierarchy from Rust.
    pub fn set_root(&mut self, root: HierarchyNode<String>) {
        self.root = Some(root);
    }
}

impl ScriptHook for D3Sunburst {
    fn on_after_apply(
        &mut self,
        vm: &mut ScriptVm,
        _apply: &Apply,
        _scope: &mut Scope,
        _value: ScriptValue,
    ) {
        if let Some(root) = parse_hierarchy(vm, self.data) {
            self.root = Some(root);
        }
    }
}

impl Widget for D3Sunburst {
    fn script_call(
        &mut self,
        vm: &mut ScriptVm,
        method: LiveId,
        args: ScriptValue,
    ) -> ScriptAsyncResult {
        if method == live_id!(set_data) {
            let value = vm_data::arg(vm, args, 0);
            if let Some(root) = parse_hierarchy(vm, value) {
                self.set_root(root);
                vm.with_cx_mut(|cx| self.redraw(cx));
            }
            return ScriptAsyncResult::Return(NIL);
        }
        ScriptAsyncResult::MethodNotFound
    }

    fn handle_event(&mut self, _cx: &mut Cx, _event: &Event, _scope: &mut Scope) {}

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.root.is_none() {
            self.root = Some(demo_hierarchy());
        }
        self.rect = cx.walk_turtle(walk);
        self.plot = compute_plot_rect(&self.rect, &self.plot_margin);
        self.draw_bg.draw_abs(cx, self.rect);

        begin_plot(cx, &self.rect, &self.plot_margin, &mut self.draw_vector);

        let center = DVec2 {
            x: self.plot.pos.x + self.plot.size.x * 0.5,
            y: self.plot.pos.y + self.plot.size.y * 0.5,
        };
        let radius = (self.plot.size.x.min(self.plot.size.y) * 0.5 - 4.0).max(4.0);

        let mut root = self.root.clone().unwrap();
        root.sum();
        let part = PartitionLayout::new()
            .size(std::f64::consts::TAU, radius)
            .layout(&root);

        let max_y = part
            .descendants(true)
            .iter()
            .map(|n| n.y1)
            .fold(1.0_f64, f64::max);

        // Draw ring segments; nodes carry (angle0, angle1, r0, r1)
        fn draw_node(
            dv: &mut DrawVector,
            node: &PartitionNode<String>,
            center: DVec2,
            radius: f64,
            max_y: f64,
            top: usize,
            depth: usize,
        ) {
            if depth > 0 {
                let a0 = node.x0;
                let a1 = node.x1;
                let r0 = node.y0 / max_y * radius;
                let r1 = node.y1 / max_y * radius;
                let sweep = (a1 - a0).max(0.0);
                if sweep > 0.002 && r1 > r0 {
                    let mut color = branch_color(top);
                    // fade deeper rings
                    let fade = 1.0 - 0.15 * (depth.saturating_sub(1) as f32);
                    color.x *= fade.max(0.4);
                    color.y *= fade.max(0.4);
                    color.z *= fade.max(0.4);
                    dv.set_color(color.x, color.y, color.z, color.w);
                    let segs = ((sweep / 0.05).ceil() as usize).clamp(2, 256);
                    let step = sweep / segs as f64;
                    let pt = |angle: f64, r: f64| -> (f32, f32) {
                        (
                            (center.x + r * angle.sin()) as f32,
                            (center.y - r * angle.cos()) as f32,
                        )
                    };
                    let (sx, sy) = pt(a0, r1);
                    dv.move_to(sx, sy);
                    for k in 1..=segs {
                        let (px, py) = pt(a0 + step * k as f64, r1);
                        dv.line_to(px, py);
                    }
                    for k in (0..=segs).rev() {
                        let (px, py) = pt(a0 + step * k as f64, r0 + 1.0);
                        dv.line_to(px, py);
                    }
                    dv.close();
                    dv.fill();
                }
            }
            for (i, child) in node.children.iter().enumerate() {
                let t = if depth == 0 { i } else { top };
                draw_node(dv, child, center, radius, max_y, t, depth + 1);
            }
        }
        draw_node(&mut self.draw_vector, &part, center, radius, max_y, 0, 0);

        end_plot(cx, &mut self.draw_vector);
        DrawStep::done()
    }
}

// ================= Circle pack =================

/// Nested circle packing, exposed as `d3.CirclePack`.
#[derive(Script, Widget)]
pub struct D3CirclePack {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,

    #[redraw]
    #[live]
    draw_bg: DrawColor,
    #[live]
    draw_grid: DrawColor,
    #[live]
    draw_vector: DrawVector,
    #[live]
    draw_text: DrawText,

    /// Declarative data: nested hierarchy object.
    #[live]
    data: ScriptValue,
    /// Stroke color for branch (non-leaf) circles.
    #[live]
    pub branch_color: Vec4f,
    /// Margins around the plot area.
    #[live]
    pub plot_margin: Inset,

    #[rust]
    root: Option<HierarchyNode<String>>,
    #[rust]
    rect: Rect,
    #[rust]
    plot: Rect,
}

impl D3CirclePack {
    /// Replace the hierarchy from Rust.
    pub fn set_root(&mut self, root: HierarchyNode<String>) {
        self.root = Some(root);
    }
}

impl ScriptHook for D3CirclePack {
    fn on_after_apply(
        &mut self,
        vm: &mut ScriptVm,
        _apply: &Apply,
        _scope: &mut Scope,
        _value: ScriptValue,
    ) {
        if let Some(root) = parse_hierarchy(vm, self.data) {
            self.root = Some(root);
        }
    }
}

impl Widget for D3CirclePack {
    fn script_call(
        &mut self,
        vm: &mut ScriptVm,
        method: LiveId,
        args: ScriptValue,
    ) -> ScriptAsyncResult {
        if method == live_id!(set_data) {
            let value = vm_data::arg(vm, args, 0);
            if let Some(root) = parse_hierarchy(vm, value) {
                self.set_root(root);
                vm.with_cx_mut(|cx| self.redraw(cx));
            }
            return ScriptAsyncResult::Return(NIL);
        }
        ScriptAsyncResult::MethodNotFound
    }

    fn handle_event(&mut self, _cx: &mut Cx, _event: &Event, _scope: &mut Scope) {}

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.root.is_none() {
            self.root = Some(demo_hierarchy());
        }
        self.rect = cx.walk_turtle(walk);
        self.plot = compute_plot_rect(&self.rect, &self.plot_margin);
        self.draw_bg.draw_abs(cx, self.rect);

        begin_plot(cx, &self.rect, &self.plot_margin, &mut self.draw_vector);

        let mut root = self.root.clone().unwrap();
        root.sum();
        let mut leaves = Vec::new();
        collect_leaves(&root, 0, 0, &mut leaves);

        // Bubble-pack the leaves (radius ~ sqrt(value)), then map the
        // bounding circle onto the plot.
        let radii: Vec<f64> = leaves
            .iter()
            .map(|(l, _)| l.value.max(0.01).sqrt())
            .collect();
        let centers = pack_bubbles(&radii);
        let extent = centers
            .iter()
            .zip(&radii)
            .map(|(&(x, y), &r)| (x * x + y * y).sqrt() + r)
            .fold(1.0_f64, f64::max);
        let scale = (self.plot.size.x.min(self.plot.size.y) * 0.5 - 2.0).max(2.0) / extent;
        let cx0 = self.plot.pos.x + self.plot.size.x * 0.5;
        let cy0 = self.plot.pos.y + self.plot.size.y * 0.5;

        // Enclosing circle outline
        let b = self.branch_color;
        self.draw_vector.set_color(b.x, b.y, b.z, b.w);
        self.draw_vector
            .circle(cx0 as f32, cy0 as f32, (extent * scale) as f32);
        self.draw_vector.stroke(1.0);

        for (i, ((_, top), &(x, y))) in leaves.iter().zip(&centers).enumerate() {
            let c = branch_color(*top);
            self.draw_vector.set_color(c.x, c.y, c.z, c.w);
            self.draw_vector.circle(
                (cx0 + x * scale) as f32,
                (cy0 + y * scale) as f32,
                (radii[i] * scale).max(1.0) as f32,
            );
            self.draw_vector.fill();
        }

        end_plot(cx, &mut self.draw_vector);
        DrawStep::done()
    }
}

// ================= Tree =================

/// Tidy node-link tree, exposed as `d3.TreeChart`.
#[derive(Script, Widget)]
pub struct D3TreeChart {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,

    #[redraw]
    #[live]
    draw_bg: DrawColor,
    #[live]
    draw_grid: DrawColor,
    #[live]
    draw_vector: DrawVector,
    #[live]
    draw_text: DrawText,

    /// Declarative data: nested hierarchy object.
    #[live]
    data: ScriptValue,
    /// Link (edge) color.
    #[live]
    pub link_color: Vec4f,
    /// Branch node fill color.
    #[live]
    pub node_color: Vec4f,
    /// Leaf node fill color.
    #[live]
    pub leaf_color: Vec4f,
    /// Node circle radius.
    #[live(4.0)]
    pub node_radius: f32,
    /// Margins around the plot area.
    #[live]
    pub plot_margin: Inset,

    #[rust]
    root: Option<HierarchyNode<String>>,
    #[rust]
    rect: Rect,
    #[rust]
    plot: Rect,
}

impl D3TreeChart {
    /// Replace the hierarchy from Rust.
    pub fn set_root(&mut self, root: HierarchyNode<String>) {
        self.root = Some(root);
    }
}

impl ScriptHook for D3TreeChart {
    fn on_after_apply(
        &mut self,
        vm: &mut ScriptVm,
        _apply: &Apply,
        _scope: &mut Scope,
        _value: ScriptValue,
    ) {
        if let Some(root) = parse_hierarchy(vm, self.data) {
            self.root = Some(root);
        }
    }
}

impl Widget for D3TreeChart {
    fn script_call(
        &mut self,
        vm: &mut ScriptVm,
        method: LiveId,
        args: ScriptValue,
    ) -> ScriptAsyncResult {
        if method == live_id!(set_data) {
            let value = vm_data::arg(vm, args, 0);
            if let Some(root) = parse_hierarchy(vm, value) {
                self.set_root(root);
                vm.with_cx_mut(|cx| self.redraw(cx));
            }
            return ScriptAsyncResult::Return(NIL);
        }
        ScriptAsyncResult::MethodNotFound
    }

    fn handle_event(&mut self, _cx: &mut Cx, _event: &Event, _scope: &mut Scope) {}

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.root.is_none() {
            self.root = Some(demo_hierarchy());
        }
        self.rect = cx.walk_turtle(walk);
        self.plot = compute_plot_rect(&self.rect, &self.plot_margin);
        self.draw_bg.draw_abs(cx, self.rect);

        begin_plot(cx, &self.rect, &self.plot_margin, &mut self.draw_vector);

        let mut root = self.root.clone().unwrap();
        root.sum();
        // Horizontal tree: layout in (height, width) space then swap axes.
        let laid = TreeLayout::new()
            .size(self.plot.size.y, self.plot.size.x)
            .layout(&root);

        let ox = self.plot.pos.x;
        let oy = self.plot.pos.y;

        // Links (cubic horizontal beziers), then nodes on top.
        fn draw_links(
            dv: &mut DrawVector,
            node: &HierarchyNode<String>,
            ox: f64,
            oy: f64,
            c: Vec4f,
        ) {
            for child in &node.children {
                let (x0, y0) = (ox + node.y, oy + node.x);
                let (x1, y1) = (ox + child.y, oy + child.x);
                let mx = (x0 + x1) * 0.5;
                dv.set_color(c.x, c.y, c.z, c.w);
                dv.move_to(x0 as f32, y0 as f32);
                dv.bezier_to(
                    mx as f32, y0 as f32, mx as f32, y1 as f32, x1 as f32, y1 as f32,
                );
                dv.stroke(1.5);
                draw_links(dv, child, ox, oy, c);
            }
        }
        fn draw_nodes(
            dv: &mut DrawVector,
            node: &HierarchyNode<String>,
            ox: f64,
            oy: f64,
            branch: Vec4f,
            leaf: Vec4f,
            r: f32,
        ) {
            let c = if node.is_leaf() { leaf } else { branch };
            dv.set_color(c.x, c.y, c.z, c.w);
            dv.circle((ox + node.y) as f32, (oy + node.x) as f32, r);
            dv.fill();
            for child in &node.children {
                draw_nodes(dv, child, ox, oy, branch, leaf, r);
            }
        }
        draw_links(&mut self.draw_vector, &laid, ox, oy, self.link_color);
        draw_nodes(
            &mut self.draw_vector,
            &laid,
            ox,
            oy,
            self.node_color,
            self.leaf_color,
            self.node_radius,
        );

        end_plot(cx, &mut self.draw_vector);
        DrawStep::done()
    }
}

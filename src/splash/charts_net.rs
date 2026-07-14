//! Network, density and geographic widgets: `d3.ForceGraph`, `d3.Hexbin`,
//! `d3.Ridgeline`, `d3.Horizon`, `d3.Contour`, `d3.Globe`.

// The `script_mod!` and `derive(Script, Widget)` macros generate public
// registration items that cannot carry doc comments.
#![allow(missing_docs)]
// Marching-squares segment tables read clearer as plain nested tuples.
#![allow(clippy::type_complexity)]

use crate::layout::{
    CenterForce, ForceSimulation, LinkForce, ManyBodyForce, SimulationLink, SimulationNode,
};

use super::charts::{begin_plot, compute_plot_rect, end_plot, fire_index_callback, min_max};
use super::charts_flow::{demo_graph, parse_graph, GraphData};
use super::charts_stat::{colormap_from_name, colormap_vec4f};
use super::vm_data;
use makepad_widgets::makepad_script::ScriptFnRef;
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*

    mod.d3.ForceGraphBase = #(D3ForceGraph::register_widget(vm))
    mod.d3.ForceGraph = set_type_default() do mod.d3.ForceGraphBase{
        width: Fill
        height: 300
        plot_margin: Inset{left: 16.0, top: 16.0, right: 16.0, bottom: 16.0}
        link_color: #x5a6075
        node_radius: 6.0
        draw_bg +: { draw_depth: 0.0 color: #x00000000 }
        draw_grid +: { draw_depth: 0.1 }
        draw_vector +: { draw_depth: 2.0 }
        draw_text +: { draw_depth: 3.0 }
    }

    mod.d3.HexbinBase = #(D3Hexbin::register_widget(vm))
    mod.d3.Hexbin = set_type_default() do mod.d3.HexbinBase{
        width: Fill
        height: 300
        plot_margin: Inset{left: 16.0, top: 16.0, right: 16.0, bottom: 16.0}
        hex_radius: 12.0
        colormap: "viridis"
        draw_bg +: { draw_depth: 0.0 color: #x00000000 }
        draw_grid +: { draw_depth: 0.1 }
        draw_vector +: { draw_depth: 2.0 }
        draw_text +: { draw_depth: 3.0 }
    }

    mod.d3.RidgelineBase = #(D3Ridgeline::register_widget(vm))
    mod.d3.Ridgeline = set_type_default() do mod.d3.RidgelineBase{
        width: Fill
        height: 300
        plot_margin: Inset{left: 16.0, top: 16.0, right: 16.0, bottom: 16.0}
        overlap: 2.2
        draw_bg +: { draw_depth: 0.0 color: #x00000000 }
        draw_grid +: { draw_depth: 0.1 }
        draw_vector +: { draw_depth: 2.0 }
        draw_text +: { draw_depth: 3.0 }
    }

    mod.d3.HorizonBase = #(D3Horizon::register_widget(vm))
    mod.d3.Horizon = set_type_default() do mod.d3.HorizonBase{
        width: Fill
        height: 300
        plot_margin: Inset{left: 16.0, top: 16.0, right: 16.0, bottom: 16.0}
        bands: 3
        band_color: #x1f77b4
        draw_bg +: { draw_depth: 0.0 color: #x00000000 }
        draw_grid +: { draw_depth: 0.1 }
        draw_vector +: { draw_depth: 2.0 }
        draw_text +: { draw_depth: 3.0 }
    }

    mod.d3.ContourBase = #(D3Contour::register_widget(vm))
    mod.d3.Contour = set_type_default() do mod.d3.ContourBase{
        width: Fill
        height: 300
        plot_margin: Inset{left: 16.0, top: 16.0, right: 16.0, bottom: 16.0}
        thresholds: 9
        colormap: "viridis"
        line_width: 1.5
        draw_bg +: { draw_depth: 0.0 color: #x00000000 }
        draw_grid +: { draw_depth: 0.1 }
        draw_vector +: { draw_depth: 2.0 }
        draw_text +: { draw_depth: 3.0 }
    }

    mod.d3.GlobeBase = #(D3Globe::register_widget(vm))
    mod.d3.Globe = set_type_default() do mod.d3.GlobeBase{
        width: Fill
        height: 300
        plot_margin: Inset{left: 12.0, top: 12.0, right: 12.0, bottom: 12.0}
        sphere_color: #x1a2030
        graticule_color: #x3a4258
        point_color: #xff7f0e
        rotation_lon: -30.0
        rotation_lat: 20.0
        draw_bg +: { draw_depth: 0.0 color: #x00000000 }
        draw_grid +: { draw_depth: 0.1 }
        draw_vector +: { draw_depth: 2.0 }
        draw_text +: { draw_depth: 3.0 }
    }
}

fn cat_color(i: usize) -> Vec4f {
    super::charts::rgba_to_vec4f(crate::color::CategoricalScale::category10().get(i))
}

// ================= Force-directed graph =================

/// Force-directed node/link graph, exposed as `d3.ForceGraph`.
///
/// The simulation runs to (bounded) convergence when data arrives; the
/// result is drawn statically.
#[derive(Script, Widget)]
pub struct D3ForceGraph {
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

    /// Declarative data: `{nodes: [...] links: [...]}`.
    #[live]
    data: ScriptValue,
    /// Link stroke color.
    #[live]
    pub link_color: Vec4f,
    /// Node circle radius.
    #[live(6.0)]
    pub node_radius: f32,
    /// Margins around the plot area.
    #[live]
    pub plot_margin: Inset,

    /// Fired with the node index on click.
    #[live]
    on_click: ScriptFnRef,
    /// Fired with the node index on hover.
    #[live]
    on_hover: ScriptFnRef,

    #[rust]
    graph: Option<GraphData>,
    #[rust]
    positions: Vec<(f64, f64)>,
    #[rust]
    sim_done: bool,
    #[rust]
    rect: Rect,
    #[rust]
    plot: Rect,
    #[rust]
    hovered: Option<usize>,
}

impl D3ForceGraph {
    /// Replace the graph from Rust.
    pub fn set_graph(&mut self, graph: GraphData) {
        self.graph = Some(graph);
        self.sim_done = false;
        self.hovered = None;
    }

    fn run_simulation(&mut self) {
        let Some(graph) = &self.graph else {
            return;
        };
        let n = graph.names.len();
        let mut nodes = Vec::with_capacity(n);
        for i in 0..n {
            let mut node = SimulationNode::new(i);
            let angle = std::f64::consts::TAU * i as f64 / n.max(1) as f64;
            node.x = 100.0 * angle.cos();
            node.y = 100.0 * angle.sin();
            nodes.push(node);
        }
        let links: Vec<SimulationLink> = graph
            .links
            .iter()
            .map(|&(s, t, _)| SimulationLink::new(s, t))
            .collect();
        let mut sim = ForceSimulation::new(nodes)
            .add_force("charge", ManyBodyForce::new().strength(-60.0))
            .add_force("link", LinkForce::new(links).distance(60.0))
            .add_force("center", CenterForce::new());
        sim.tick_n(300);
        self.positions = sim.nodes().iter().map(|node| (node.x, node.y)).collect();
        self.sim_done = true;
    }

    /// Simulation positions mapped into the plot rect.
    fn screen_positions(&self) -> Vec<(f64, f64)> {
        if self.positions.is_empty() {
            return Vec::new();
        }
        let (x_min, x_max) = min_max(self.positions.iter().map(|p| p.0));
        let (y_min, y_max) = min_max(self.positions.iter().map(|p| p.1));
        let pad = self.node_radius as f64 + 4.0;
        let sx = (self.plot.size.x - pad * 2.0) / (x_max - x_min).max(1.0);
        let sy = (self.plot.size.y - pad * 2.0) / (y_max - y_min).max(1.0);
        let s = sx.min(sy);
        let cx = self.plot.pos.x + self.plot.size.x * 0.5;
        let cy = self.plot.pos.y + self.plot.size.y * 0.5;
        let mx = (x_min + x_max) * 0.5;
        let my = (y_min + y_max) * 0.5;
        self.positions
            .iter()
            .map(|&(x, y)| (cx + (x - mx) * s, cy + (y - my) * s))
            .collect()
    }

    fn node_at(&self, abs: DVec2) -> Option<usize> {
        let rr = (self.node_radius as f64 + 5.0).powi(2);
        self.screen_positions()
            .iter()
            .position(|&(x, y)| (x - abs.x).powi(2) + (y - abs.y).powi(2) <= rr)
    }
}

impl ScriptHook for D3ForceGraph {
    fn on_after_apply(
        &mut self,
        vm: &mut ScriptVm,
        _apply: &Apply,
        _scope: &mut Scope,
        _value: ScriptValue,
    ) {
        if let Some(graph) = parse_graph(vm, self.data) {
            self.set_graph(graph);
        }
    }
}

impl Widget for D3ForceGraph {
    fn script_call(
        &mut self,
        vm: &mut ScriptVm,
        method: LiveId,
        args: ScriptValue,
    ) -> ScriptAsyncResult {
        if method == live_id!(set_data) {
            let value = vm_data::arg(vm, args, 0);
            if let Some(graph) = parse_graph(vm, value) {
                self.set_graph(graph);
                vm.with_cx_mut(|cx| self.redraw(cx));
            }
            return ScriptAsyncResult::Return(NIL);
        }
        ScriptAsyncResult::MethodNotFound
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        match event.hits(cx, self.draw_bg.area()) {
            Hit::FingerHoverIn(fe) | Hit::FingerHoverOver(fe) => {
                let hit = self.node_at(fe.abs);
                if hit != self.hovered {
                    self.hovered = hit;
                    if let Some(i) = hit {
                        fire_index_callback(cx, self.uid, &self.source, &self.on_hover, i);
                    }
                    self.redraw(cx);
                }
            }
            Hit::FingerHoverOut(_) if self.hovered.is_some() => {
                self.hovered = None;
                self.redraw(cx);
            }
            Hit::FingerUp(fe) if fe.is_over && fe.is_primary_hit() => {
                if let Some(i) = self.node_at(fe.abs) {
                    fire_index_callback(cx, self.uid, &self.source, &self.on_click, i);
                }
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.graph.is_none() {
            self.graph = Some(demo_graph());
            self.sim_done = false;
        }
        if !self.sim_done {
            self.run_simulation();
        }
        self.rect = cx.walk_turtle(walk);
        self.plot = compute_plot_rect(&self.rect, &self.plot_margin);
        self.draw_bg.draw_abs(cx, self.rect);

        begin_plot(cx, &self.rect, &self.plot_margin, &mut self.draw_vector);

        let pos = self.screen_positions();
        if let Some(graph) = &self.graph {
            let lc = self.link_color;
            for &(s, t, v) in &graph.links {
                if s < pos.len() && t < pos.len() {
                    self.draw_vector.set_color(lc.x, lc.y, lc.z, lc.w);
                    self.draw_vector.move_to(pos[s].0 as f32, pos[s].1 as f32);
                    self.draw_vector.line_to(pos[t].0 as f32, pos[t].1 as f32);
                    self.draw_vector.stroke((0.8 + v * 0.15).min(4.0) as f32);
                }
            }
            for (i, &(x, y)) in pos.iter().enumerate() {
                let mut c = cat_color(i % 10);
                if self.hovered == Some(i) {
                    c.x = (c.x * 1.3).min(1.0);
                    c.y = (c.y * 1.3).min(1.0);
                    c.z = (c.z * 1.3).min(1.0);
                }
                self.draw_vector.set_color(c.x, c.y, c.z, c.w);
                self.draw_vector
                    .circle(x as f32, y as f32, self.node_radius);
                self.draw_vector.fill();
            }
        }

        end_plot(cx, &mut self.draw_vector);
        DrawStep::done()
    }
}

// ================= Hexbin =================

/// Hexagonal density binning of x/y points, exposed as `d3.Hexbin`.
#[derive(Script, Widget)]
pub struct D3Hexbin {
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

    /// Declarative data: x/y points (pairs or objects).
    #[live]
    data: ScriptValue,
    /// Hexagon radius in pixels.
    #[live(12.0)]
    pub hex_radius: f32,
    /// Colormap name (see `d3.Heatmap`).
    #[live]
    pub colormap: String,
    /// Margins around the plot area.
    #[live]
    pub plot_margin: Inset,

    #[rust]
    points: Vec<(f64, f64)>,
    #[rust]
    rect: Rect,
    #[rust]
    plot: Rect,
}

impl D3Hexbin {
    /// Replace the points from Rust.
    pub fn set_points(&mut self, points: Vec<(f64, f64)>) {
        self.points = points;
    }

    fn ensure_demo_data(&mut self) {
        if self.points.is_empty() {
            let mut seed = 99u64;
            let mut r = || {
                seed ^= seed << 13;
                seed ^= seed >> 7;
                seed ^= seed << 17;
                seed as f64 / u64::MAX as f64
            };
            self.points = (0..900)
                .map(|_| {
                    let cluster = if r() > 0.5 { (0.32, 0.4) } else { (0.68, 0.62) };
                    (
                        cluster.0 + (r() + r() - 1.0) * 0.22,
                        cluster.1 + (r() + r() - 1.0) * 0.22,
                    )
                })
                .collect();
        }
    }
}

impl ScriptHook for D3Hexbin {
    fn on_after_apply(
        &mut self,
        vm: &mut ScriptVm,
        _apply: &Apply,
        _scope: &mut Scope,
        _value: ScriptValue,
    ) {
        if let Some(points) = vm_data::to_xy_vec(vm, self.data) {
            if !points.is_empty() {
                self.points = points;
            }
        }
    }
}

impl Widget for D3Hexbin {
    fn script_call(
        &mut self,
        vm: &mut ScriptVm,
        method: LiveId,
        args: ScriptValue,
    ) -> ScriptAsyncResult {
        if method == live_id!(set_data) {
            let value = vm_data::arg(vm, args, 0);
            if let Some(points) = vm_data::to_xy_vec(vm, value) {
                self.set_points(points);
                vm.with_cx_mut(|cx| self.redraw(cx));
            }
            return ScriptAsyncResult::Return(NIL);
        }
        ScriptAsyncResult::MethodNotFound
    }

    fn handle_event(&mut self, _cx: &mut Cx, _event: &Event, _scope: &mut Scope) {}

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        self.ensure_demo_data();
        self.rect = cx.walk_turtle(walk);
        self.plot = compute_plot_rect(&self.rect, &self.plot_margin);
        self.draw_bg.draw_abs(cx, self.rect);

        begin_plot(cx, &self.rect, &self.plot_margin, &mut self.draw_vector);

        // Scale data into the plot, then bin in pixel space (pointy-top hexes).
        let (x_min, x_max) = min_max(self.points.iter().map(|p| p.0));
        let (y_min, y_max) = min_max(self.points.iter().map(|p| p.1));
        let xr = (x_max - x_min).max(f64::EPSILON);
        let yr = (y_max - y_min).max(f64::EPSILON);
        let r = self.hex_radius.max(3.0) as f64;

        use std::collections::HashMap;
        let mut bins: HashMap<(i64, i64), usize> = HashMap::new();
        for &(x, y) in &self.points {
            let px = self.plot.pos.x + (x - x_min) / xr * self.plot.size.x;
            let py = self.plot.pos.y + (1.0 - (y - y_min) / yr) * self.plot.size.y;
            // axial coords with cube rounding
            let q = (3f64.sqrt() / 3.0 * (px) - 1.0 / 3.0 * py) / r;
            let rr = 2.0 / 3.0 * py / r;
            let (mut rq, mut rr2) = (q.round(), rr.round());
            let (dq, dr) = (q - rq, rr - rr2);
            let ds = -dq - dr - ((-q - rr) - (-rq - rr2));
            if dq.abs() >= rr2.abs().max(ds.abs()) && dq.abs() >= dr.abs() {
                rq = -rr2 - (-q - rr).round();
            } else if dr.abs() > dq.abs() && dr.abs() >= ds.abs() {
                rr2 = -rq - (-q - rr).round();
            }
            *bins.entry((rq as i64, rr2 as i64)).or_insert(0) += 1;
        }
        let max_count = bins.values().copied().max().unwrap_or(1) as f64;
        let cm = colormap_from_name(&self.colormap);

        for (&(q, rr2), &count) in &bins {
            let px = r * 3f64.sqrt() * (q as f64 + rr2 as f64 / 2.0);
            let py = r * 1.5 * rr2 as f64;
            if px < self.plot.pos.x - r
                || px > self.plot.pos.x + self.plot.size.x + r
                || py < self.plot.pos.y - r
                || py > self.plot.pos.y + self.plot.size.y + r
            {
                continue;
            }
            let t = (count as f64 / max_count).clamp(0.05, 1.0);
            let c = colormap_vec4f(cm, t);
            self.draw_vector.set_color(c.x, c.y, c.z, c.w);
            for k in 0..6 {
                let a = std::f64::consts::TAU * (k as f64 + 0.5) / 6.0;
                let (hx, hy) = (px + (r - 0.5) * a.sin(), py - (r - 0.5) * a.cos());
                if k == 0 {
                    self.draw_vector.move_to(hx as f32, hy as f32);
                } else {
                    self.draw_vector.line_to(hx as f32, hy as f32);
                }
            }
            self.draw_vector.close();
            self.draw_vector.fill();
        }

        end_plot(cx, &mut self.draw_vector);
        DrawStep::done()
    }
}

// ================= Ridgeline =================

/// Overlapping series ridgeline (joyplot), exposed as `d3.Ridgeline`.
#[derive(Script, Widget)]
pub struct D3Ridgeline {
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

    /// Declarative data: array of series (arrays of numbers).
    #[live]
    data: ScriptValue,
    /// Peak height as a multiple of the lane height.
    #[live(2.2)]
    pub overlap: f32,
    /// Margins around the plot area.
    #[live]
    pub plot_margin: Inset,

    #[rust]
    series: Vec<Vec<f64>>,
    #[rust]
    rect: Rect,
    #[rust]
    plot: Rect,
}

impl D3Ridgeline {
    /// Replace the series from Rust.
    pub fn set_series(&mut self, series: Vec<Vec<f64>>) {
        self.series = series;
    }

    fn ensure_demo_data(&mut self) {
        if self.series.is_empty() {
            self.series = (0..6)
                .map(|s| {
                    (0..80)
                        .map(|i| {
                            let x = i as f64 / 80.0;
                            let peak = 0.2 + s as f64 * 0.1;
                            (-(x - peak).powi(2) * 60.0).exp()
                                + 0.6 * (-(x - peak - 0.25).powi(2) * 90.0).exp()
                        })
                        .collect()
                })
                .collect();
        }
    }
}

impl ScriptHook for D3Ridgeline {
    fn on_after_apply(
        &mut self,
        vm: &mut ScriptVm,
        _apply: &Apply,
        _scope: &mut Scope,
        _value: ScriptValue,
    ) {
        if let Some(rows) = vm_data::to_rows(vm, self.data) {
            if !rows.is_empty() {
                self.series = rows;
            }
        }
    }
}

impl Widget for D3Ridgeline {
    fn script_call(
        &mut self,
        vm: &mut ScriptVm,
        method: LiveId,
        args: ScriptValue,
    ) -> ScriptAsyncResult {
        if method == live_id!(set_data) {
            let value = vm_data::arg(vm, args, 0);
            if let Some(rows) = vm_data::to_rows(vm, value) {
                if !rows.is_empty() {
                    self.set_series(rows);
                    vm.with_cx_mut(|cx| self.redraw(cx));
                }
            }
            return ScriptAsyncResult::Return(NIL);
        }
        ScriptAsyncResult::MethodNotFound
    }

    fn handle_event(&mut self, _cx: &mut Cx, _event: &Event, _scope: &mut Scope) {}

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        self.ensure_demo_data();
        self.rect = cx.walk_turtle(walk);
        self.plot = compute_plot_rect(&self.rect, &self.plot_margin);
        self.draw_bg.draw_abs(cx, self.rect);

        begin_plot(cx, &self.rect, &self.plot_margin, &mut self.draw_vector);

        let n = self.series.len();
        if n > 0 {
            let global_max = self
                .series
                .iter()
                .flatten()
                .copied()
                .fold(f64::EPSILON, f64::max);
            let lane = self.plot.size.y / n as f64;
            let amp = lane * self.overlap.max(0.5) as f64;

            for (i, row) in self.series.iter().enumerate() {
                if row.len() < 2 {
                    continue;
                }
                let baseline = self.plot.pos.y + (i as f64 + 1.0) * lane;
                let dx = self.plot.size.x / (row.len() - 1) as f64;
                let mut c = cat_color(i % 10);
                c.w = 0.85;

                self.draw_vector.set_color(c.x, c.y, c.z, c.w);
                self.draw_vector
                    .move_to(self.plot.pos.x as f32, baseline as f32);
                for (j, &v) in row.iter().enumerate() {
                    let px = self.plot.pos.x + j as f64 * dx;
                    let py = baseline - (v / global_max) * amp;
                    self.draw_vector.line_to(px as f32, py as f32);
                }
                self.draw_vector
                    .line_to((self.plot.pos.x + self.plot.size.x) as f32, baseline as f32);
                self.draw_vector.close();
                self.draw_vector.fill();

                // Crest line
                self.draw_vector.set_color(1.0, 1.0, 1.0, 0.5);
                let mut first = true;
                for (j, &v) in row.iter().enumerate() {
                    let px = self.plot.pos.x + j as f64 * dx;
                    let py = baseline - (v / global_max) * amp;
                    if first {
                        self.draw_vector.move_to(px as f32, py as f32);
                        first = false;
                    } else {
                        self.draw_vector.line_to(px as f32, py as f32);
                    }
                }
                self.draw_vector.stroke(1.0);
            }
        }

        end_plot(cx, &mut self.draw_vector);
        DrawStep::done()
    }
}

// ================= Horizon =================

/// Banded horizon chart rows, exposed as `d3.Horizon`.
#[derive(Script, Widget)]
pub struct D3Horizon {
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

    /// Declarative data: array of series (arrays of non-negative numbers).
    #[live]
    data: ScriptValue,
    /// Number of horizon bands.
    #[live(3.0)]
    pub bands: f64,
    /// Band base color (opacity stacks per band).
    #[live]
    pub band_color: Vec4f,
    /// Margins around the plot area.
    #[live]
    pub plot_margin: Inset,

    #[rust]
    series: Vec<Vec<f64>>,
    #[rust]
    rect: Rect,
    #[rust]
    plot: Rect,
}

impl D3Horizon {
    /// Replace the series from Rust.
    pub fn set_series(&mut self, series: Vec<Vec<f64>>) {
        self.series = series;
    }

    fn ensure_demo_data(&mut self) {
        if self.series.is_empty() {
            self.series = (0..5)
                .map(|s| {
                    (0..120)
                        .map(|i| {
                            let x = i as f64 * 0.12 + s as f64;
                            (x.sin() + 1.0) * 0.5 + 0.35 * ((x * 2.3).cos() + 1.0) * 0.5
                        })
                        .collect()
                })
                .collect();
        }
    }
}

impl ScriptHook for D3Horizon {
    fn on_after_apply(
        &mut self,
        vm: &mut ScriptVm,
        _apply: &Apply,
        _scope: &mut Scope,
        _value: ScriptValue,
    ) {
        if let Some(rows) = vm_data::to_rows(vm, self.data) {
            if !rows.is_empty() {
                self.series = rows;
            }
        }
    }
}

impl Widget for D3Horizon {
    fn script_call(
        &mut self,
        vm: &mut ScriptVm,
        method: LiveId,
        args: ScriptValue,
    ) -> ScriptAsyncResult {
        if method == live_id!(set_data) {
            let value = vm_data::arg(vm, args, 0);
            if let Some(rows) = vm_data::to_rows(vm, value) {
                if !rows.is_empty() {
                    self.set_series(rows);
                    vm.with_cx_mut(|cx| self.redraw(cx));
                }
            }
            return ScriptAsyncResult::Return(NIL);
        }
        ScriptAsyncResult::MethodNotFound
    }

    fn handle_event(&mut self, _cx: &mut Cx, _event: &Event, _scope: &mut Scope) {}

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        self.ensure_demo_data();
        self.rect = cx.walk_turtle(walk);
        self.plot = compute_plot_rect(&self.rect, &self.plot_margin);
        self.draw_bg.draw_abs(cx, self.rect);

        begin_plot(cx, &self.rect, &self.plot_margin, &mut self.draw_vector);

        let n = self.series.len();
        if n > 0 {
            let bands = (self.bands.max(1.0) as usize).min(6);
            let global_max = self
                .series
                .iter()
                .flatten()
                .copied()
                .fold(f64::EPSILON, f64::max);
            let lane = self.plot.size.y / n as f64;
            let gap = (lane * 0.12).min(4.0);

            for (i, row) in self.series.iter().enumerate() {
                if row.len() < 2 {
                    continue;
                }
                let base = self.plot.pos.y + (i as f64 + 1.0) * lane - gap;
                let h = lane - gap;
                let dx = self.plot.size.x / (row.len() - 1) as f64;

                for band in 0..bands {
                    let mut c = self.band_color;
                    c.w = ((band + 1) as f32 / bands as f32) * 0.85 + 0.1;
                    self.draw_vector.set_color(c.x, c.y, c.z, c.w);
                    self.draw_vector
                        .move_to(self.plot.pos.x as f32, base as f32);
                    for (j, &v) in row.iter().enumerate() {
                        let vn = (v / global_max * bands as f64 - band as f64).clamp(0.0, 1.0);
                        let px = self.plot.pos.x + j as f64 * dx;
                        let py = base - vn * h;
                        self.draw_vector.line_to(px as f32, py as f32);
                    }
                    self.draw_vector
                        .line_to((self.plot.pos.x + self.plot.size.x) as f32, base as f32);
                    self.draw_vector.close();
                    self.draw_vector.fill();
                }
            }
        }

        end_plot(cx, &mut self.draw_vector);
        DrawStep::done()
    }
}

// ================= Contour =================

/// Iso-line contour plot over a value grid (marching squares), exposed as
/// `d3.Contour`. Data shape matches `d3.Heatmap`: array of rows.
#[derive(Script, Widget)]
pub struct D3Contour {
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

    /// Declarative data: array of rows (value grid).
    #[live]
    data: ScriptValue,
    /// Number of iso levels.
    #[live(9.0)]
    pub thresholds: f64,
    /// Colormap name (see `d3.Heatmap`).
    #[live]
    pub colormap: String,
    /// Iso-line stroke width.
    #[live(1.5)]
    pub line_width: f32,
    /// Margins around the plot area.
    #[live]
    pub plot_margin: Inset,

    #[rust]
    grid: Vec<Vec<f64>>,
    #[rust]
    rect: Rect,
    #[rust]
    plot: Rect,
}

impl D3Contour {
    /// Replace the value grid from Rust.
    pub fn set_grid(&mut self, grid: Vec<Vec<f64>>) {
        self.grid = grid;
    }

    fn ensure_demo_data(&mut self) {
        if self.grid.is_empty() {
            self.grid = (0..36)
                .map(|r| {
                    (0..56)
                        .map(|c| {
                            let x = c as f64 * 0.18 - 5.0;
                            let y = r as f64 * 0.18 - 3.2;
                            (x * x + y * y).sqrt().sin() + (x * 0.6).cos() * 0.5
                        })
                        .collect()
                })
                .collect();
        }
    }
}

impl ScriptHook for D3Contour {
    fn on_after_apply(
        &mut self,
        vm: &mut ScriptVm,
        _apply: &Apply,
        _scope: &mut Scope,
        _value: ScriptValue,
    ) {
        if let Some(rows) = vm_data::to_rows(vm, self.data) {
            if !rows.is_empty() {
                self.grid = rows;
            }
        }
    }
}

impl Widget for D3Contour {
    fn script_call(
        &mut self,
        vm: &mut ScriptVm,
        method: LiveId,
        args: ScriptValue,
    ) -> ScriptAsyncResult {
        if method == live_id!(set_data) {
            let value = vm_data::arg(vm, args, 0);
            if let Some(rows) = vm_data::to_rows(vm, value) {
                if !rows.is_empty() {
                    self.set_grid(rows);
                    vm.with_cx_mut(|cx| self.redraw(cx));
                }
            }
            return ScriptAsyncResult::Return(NIL);
        }
        ScriptAsyncResult::MethodNotFound
    }

    fn handle_event(&mut self, _cx: &mut Cx, _event: &Event, _scope: &mut Scope) {}

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        self.ensure_demo_data();
        self.rect = cx.walk_turtle(walk);
        self.plot = compute_plot_rect(&self.rect, &self.plot_margin);
        self.draw_bg.draw_abs(cx, self.rect);

        begin_plot(cx, &self.rect, &self.plot_margin, &mut self.draw_vector);

        let rows = self.grid.len();
        let cols = self.grid.iter().map(|r| r.len()).min().unwrap_or(0);
        if rows >= 2 && cols >= 2 {
            let (min, max) = min_max(self.grid.iter().flatten().copied());
            let span = (max - min).max(f64::EPSILON);
            let levels = (self.thresholds.max(1.0) as usize).min(24);
            let cm = colormap_from_name(&self.colormap);
            let cw = self.plot.size.x / (cols - 1) as f64;
            let ch = self.plot.size.y / (rows - 1) as f64;
            let ox = self.plot.pos.x;
            let oy = self.plot.pos.y;

            for level in 1..=levels {
                let t = level as f64 / (levels + 1) as f64;
                let iso = min + t * span;
                let c = colormap_vec4f(cm, t);
                self.draw_vector.set_color(c.x, c.y, c.z, c.w);

                for r in 0..rows - 1 {
                    for q in 0..cols - 1 {
                        let tl = self.grid[r][q];
                        let tr = self.grid[r][q + 1];
                        let br = self.grid[r + 1][q + 1];
                        let bl = self.grid[r + 1][q];
                        let case = ((tl > iso) as u8) << 3
                            | ((tr > iso) as u8) << 2
                            | ((br > iso) as u8) << 1
                            | ((bl > iso) as u8);
                        if case == 0 || case == 15 {
                            continue;
                        }
                        let lerp = |a: f64, b: f64| -> f64 {
                            if (b - a).abs() < f64::EPSILON {
                                0.5
                            } else {
                                ((iso - a) / (b - a)).clamp(0.0, 1.0)
                            }
                        };
                        let x0 = ox + q as f64 * cw;
                        let y0 = oy + r as f64 * ch;
                        // Edge midpoints with interpolation
                        let top = (x0 + lerp(tl, tr) * cw, y0);
                        let right = (x0 + cw, y0 + lerp(tr, br) * ch);
                        let bottom = (x0 + lerp(bl, br) * cw, y0 + ch);
                        let left = (x0, y0 + lerp(tl, bl) * ch);

                        let segments: &[((f64, f64), (f64, f64))] = match case {
                            1 => &[(left, bottom)],
                            2 => &[(bottom, right)],
                            3 => &[(left, right)],
                            4 => &[(top, right)],
                            5 => &[(top, left), (bottom, right)],
                            6 => &[(top, bottom)],
                            7 => &[(top, left)],
                            8 => &[(top, left)],
                            9 => &[(top, bottom)],
                            10 => &[(top, right), (left, bottom)],
                            11 => &[(top, right)],
                            12 => &[(left, right)],
                            13 => &[(bottom, right)],
                            14 => &[(left, bottom)],
                            _ => &[],
                        };
                        for &((ax, ay), (bx, by)) in segments {
                            self.draw_vector.move_to(ax as f32, ay as f32);
                            self.draw_vector.line_to(bx as f32, by as f32);
                            self.draw_vector.stroke(self.line_width);
                        }
                    }
                }
            }
        }

        end_plot(cx, &mut self.draw_vector);
        DrawStep::done()
    }
}

// ================= Globe =================

/// Draggable orthographic globe with graticule and lon/lat points,
/// exposed as `d3.Globe`.
///
/// Points: `data: [{lon: 116.4 lat: 39.9}, {lon: -74.0 lat: 40.7}, ...]`.
#[derive(Script, Widget)]
pub struct D3Globe {
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

    /// Declarative data: array of `{lon lat}` points.
    #[live]
    data: ScriptValue,
    /// Sphere fill color.
    #[live]
    pub sphere_color: Vec4f,
    /// Graticule line color.
    #[live]
    pub graticule_color: Vec4f,
    /// Marker color.
    #[live]
    pub point_color: Vec4f,
    /// Initial longitude rotation (degrees).
    #[live(-30.0)]
    pub rotation_lon: f32,
    /// Initial latitude tilt (degrees).
    #[live(20.0)]
    pub rotation_lat: f32,
    /// Margins around the plot area.
    #[live]
    pub plot_margin: Inset,

    /// Fired with the point index on click.
    #[live]
    on_click: ScriptFnRef,

    #[rust]
    points: Vec<(f64, f64)>,
    #[rust]
    rect: Rect,
    #[rust]
    plot: Rect,
    #[rust]
    drag_start: Option<(DVec2, f32, f32)>,
}

/// Orthographic projection with backside culling.
fn globe_project(
    lon_deg: f64,
    lat_deg: f64,
    rot_lon_deg: f64,
    rot_lat_deg: f64,
    center: DVec2,
    radius: f64,
) -> Option<(f64, f64)> {
    let lon = (lon_deg + rot_lon_deg).to_radians();
    let lat = lat_deg.to_radians();
    let tilt = rot_lat_deg.to_radians();
    // Sphere point
    let x = lat.cos() * lon.sin();
    let mut y = lat.sin();
    let mut z = lat.cos() * lon.cos();
    // Tilt around the x-axis
    let (y2, z2) = (
        y * tilt.cos() - z * tilt.sin(),
        y * tilt.sin() + z * tilt.cos(),
    );
    y = y2;
    z = z2;
    if z < 0.0 {
        return None;
    }
    Some((center.x + x * radius, center.y - y * radius))
}

impl D3Globe {
    /// Replace the marker points from Rust ((lon, lat) degrees).
    pub fn set_points(&mut self, points: Vec<(f64, f64)>) {
        self.points = points;
    }

    fn ensure_demo_data(&mut self) {
        if self.points.is_empty() {
            self.points = vec![
                (116.4, 39.9),  // Beijing
                (-74.0, 40.7),  // New York
                (2.35, 48.86),  // Paris
                (139.7, 35.7),  // Tokyo
                (-122.4, 37.8), // San Francisco
                (151.2, -33.9), // Sydney
            ];
        }
    }

    fn geometry(&self) -> (DVec2, f64) {
        let center = DVec2 {
            x: self.plot.pos.x + self.plot.size.x * 0.5,
            y: self.plot.pos.y + self.plot.size.y * 0.5,
        };
        let radius = (self.plot.size.x.min(self.plot.size.y) * 0.5 - 4.0).max(4.0);
        (center, radius)
    }

    fn draw_arc(&mut self, samples: &[(f64, f64)], center: DVec2, radius: f64) {
        let mut pen_down = false;
        for &(lon, lat) in samples {
            match globe_project(
                lon,
                lat,
                self.rotation_lon as f64,
                self.rotation_lat as f64,
                center,
                radius,
            ) {
                Some((px, py)) => {
                    if pen_down {
                        self.draw_vector.line_to(px as f32, py as f32);
                    } else {
                        self.draw_vector.move_to(px as f32, py as f32);
                        pen_down = true;
                    }
                }
                None => {
                    if pen_down {
                        self.draw_vector.stroke(1.0);
                    }
                    pen_down = false;
                }
            }
        }
        if pen_down {
            self.draw_vector.stroke(1.0);
        }
    }
}

impl ScriptHook for D3Globe {
    fn on_after_apply(
        &mut self,
        vm: &mut ScriptVm,
        _apply: &Apply,
        _scope: &mut Scope,
        _value: ScriptValue,
    ) {
        if let Some(items) = vm_data::elements(vm, self.data) {
            let mut points = Vec::new();
            for item in items {
                let lon = vm_data::field(vm, item, live_id!(lon)).as_number();
                let lat = vm_data::field(vm, item, live_id!(lat)).as_number();
                if let (Some(lon), Some(lat)) = (lon, lat) {
                    points.push((lon, lat));
                }
            }
            if !points.is_empty() {
                self.points = points;
            }
        }
    }
}

impl Widget for D3Globe {
    fn script_call(
        &mut self,
        vm: &mut ScriptVm,
        method: LiveId,
        args: ScriptValue,
    ) -> ScriptAsyncResult {
        if method == live_id!(set_rotation) {
            let lon = vm_data::arg(vm, args, 0).as_number();
            let lat = vm_data::arg(vm, args, 1).as_number();
            if let Some(lon) = lon {
                self.rotation_lon = lon as f32;
            }
            if let Some(lat) = lat {
                self.rotation_lat = (lat as f32).clamp(-89.0, 89.0);
            }
            vm.with_cx_mut(|cx| self.redraw(cx));
            return ScriptAsyncResult::Return(NIL);
        }
        ScriptAsyncResult::MethodNotFound
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        match event.hits(cx, self.draw_bg.area()) {
            Hit::FingerDown(fe) if fe.is_primary_hit() => {
                self.drag_start = Some((fe.abs, self.rotation_lon, self.rotation_lat));
                cx.set_cursor(MouseCursor::Grabbing);
            }
            Hit::FingerMove(fe) => {
                if let Some((start, lon0, lat0)) = self.drag_start {
                    let dx = (fe.abs.x - start.x) as f32;
                    let dy = (fe.abs.y - start.y) as f32;
                    self.rotation_lon = lon0 + dx * 0.4;
                    self.rotation_lat = (lat0 + dy * 0.4).clamp(-89.0, 89.0);
                    self.redraw(cx);
                }
            }
            Hit::FingerUp(fe) => {
                let was_drag = self
                    .drag_start
                    .take()
                    .map(|(start, _, _)| {
                        (fe.abs.x - start.x).abs() + (fe.abs.y - start.y).abs() > 4.0
                    })
                    .unwrap_or(false);
                cx.set_cursor(MouseCursor::Grab);
                if !was_drag && fe.is_over && fe.is_primary_hit() {
                    // Click: nearest visible marker
                    let (center, radius) = self.geometry();
                    let mut best = None;
                    let mut best_d = 14.0_f64.powi(2);
                    for (i, &(lon, lat)) in self.points.iter().enumerate() {
                        if let Some((px, py)) = globe_project(
                            lon,
                            lat,
                            self.rotation_lon as f64,
                            self.rotation_lat as f64,
                            center,
                            radius,
                        ) {
                            let d = (px - fe.abs.x).powi(2) + (py - fe.abs.y).powi(2);
                            if d < best_d {
                                best_d = d;
                                best = Some(i);
                            }
                        }
                    }
                    if let Some(i) = best {
                        fire_index_callback(cx, self.uid, &self.source, &self.on_click, i);
                    }
                }
            }
            Hit::FingerHoverIn(_) => {
                cx.set_cursor(MouseCursor::Grab);
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        self.ensure_demo_data();
        self.rect = cx.walk_turtle(walk);
        self.plot = compute_plot_rect(&self.rect, &self.plot_margin);
        self.draw_bg.draw_abs(cx, self.rect);

        begin_plot(cx, &self.rect, &self.plot_margin, &mut self.draw_vector);

        let (center, radius) = self.geometry();

        // Sphere disc
        let s = self.sphere_color;
        self.draw_vector.set_color(s.x, s.y, s.z, s.w);
        self.draw_vector
            .circle(center.x as f32, center.y as f32, radius as f32);
        self.draw_vector.fill();

        // Graticule
        let g = self.graticule_color;
        self.draw_vector.set_color(g.x, g.y, g.z, g.w);
        for m in 0..12 {
            let lon = m as f64 * 30.0;
            let samples: Vec<(f64, f64)> =
                (-88..=88).step_by(4).map(|lat| (lon, lat as f64)).collect();
            self.draw_arc(&samples, center, radius);
        }
        for p in -2..=2 {
            let lat = p as f64 * 30.0;
            let samples: Vec<(f64, f64)> =
                (0..=360).step_by(4).map(|lon| (lon as f64, lat)).collect();
            self.draw_arc(&samples, center, radius);
        }

        // Outline
        self.draw_vector.set_color(g.x, g.y, g.z, 1.0);
        self.draw_vector
            .circle(center.x as f32, center.y as f32, radius as f32);
        self.draw_vector.stroke(1.5);

        // Markers
        let p = self.point_color;
        for &(lon, lat) in &self.points {
            if let Some((px, py)) = globe_project(
                lon,
                lat,
                self.rotation_lon as f64,
                self.rotation_lat as f64,
                center,
                radius,
            ) {
                self.draw_vector.set_color(p.x, p.y, p.z, p.w);
                self.draw_vector.circle(px as f32, py as f32, 4.0);
                self.draw_vector.fill();
            }
        }

        end_plot(cx, &mut self.draw_vector);
        DrawStep::done()
    }
}

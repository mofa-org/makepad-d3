//! Flow chart widgets: `d3.Sankey`, `d3.ChordDiagram`, `d3.ArcDiagram`.
//!
//! `d3.Sankey` and `d3.ArcDiagram` consume a node/link graph:
//!
//! ```splash,ignore
//! data: {
//!     nodes: [{name: "Coal"}, {name: "Gas"}, {name: "Power"}, {name: "Home"}]
//!     links: [
//!         {source: 0 target: 2 value: 10},
//!         {source: 1 target: 2 value: 6},
//!         {source: 2 target: 3 value: 14}
//!     ]
//! }
//! ```
//!
//! `d3.ChordDiagram` consumes a square flow matrix: `data: [[..], [..], ..]`.

// The `script_mod!` and `derive(Script, Widget)` macros generate public
// registration items that cannot carry doc comments.
#![allow(missing_docs)]

use crate::color::CategoricalScale;

use super::charts::{begin_plot, compute_plot_rect, end_plot, fire_index_callback, rgba_to_vec4f};
use super::vm_data;
use makepad_widgets::makepad_script::ScriptFnRef;
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*

    mod.d3.SankeyBase = #(D3Sankey::register_widget(vm))
    mod.d3.Sankey = set_type_default() do mod.d3.SankeyBase{
        width: Fill
        height: 300
        plot_margin: Inset{left: 12.0, top: 12.0, right: 12.0, bottom: 12.0}
        node_width: 14.0
        node_padding: 10.0
        link_alpha: 0.35
        draw_bg +: { draw_depth: 0.0 color: #x00000000 }
        draw_grid +: { draw_depth: 0.1 }
        draw_vector +: { draw_depth: 2.0 }
        draw_text +: { draw_depth: 3.0 }
    }

    mod.d3.ChordDiagramBase = #(D3ChordDiagram::register_widget(vm))
    mod.d3.ChordDiagram = set_type_default() do mod.d3.ChordDiagramBase{
        width: Fill
        height: 300
        plot_margin: Inset{left: 12.0, top: 12.0, right: 12.0, bottom: 12.0}
        ribbon_alpha: 0.55
        draw_bg +: { draw_depth: 0.0 color: #x00000000 }
        draw_grid +: { draw_depth: 0.1 }
        draw_vector +: { draw_depth: 2.0 }
        draw_text +: { draw_depth: 3.0 }
    }

    mod.d3.ArcDiagramBase = #(D3ArcDiagram::register_widget(vm))
    mod.d3.ArcDiagram = set_type_default() do mod.d3.ArcDiagramBase{
        width: Fill
        height: 300
        plot_margin: Inset{left: 24.0, top: 12.0, right: 24.0, bottom: 30.0}
        node_radius: 5.0
        arc_alpha: 0.5
        draw_bg +: { draw_depth: 0.0 color: #x00000000 }
        draw_grid +: { draw_depth: 0.1 }
        draw_vector +: { draw_depth: 2.0 }
        draw_text +: { draw_depth: 3.0 }
    }
}

// ---- Shared graph parsing ----

/// Parsed node/link graph shared by the flow and network widgets.
pub struct GraphData {
    pub names: Vec<String>,
    /// (source, target, value)
    pub links: Vec<(usize, usize, f64)>,
}

/// Parse `{nodes: [...] links: [...]}`. Links accept `{source target value?}`
/// objects or `[s t v?]` arrays; node indices out of range are dropped.
pub(crate) fn parse_graph(vm: &mut ScriptVm, value: ScriptValue) -> Option<GraphData> {
    if value.is_nil() || value.as_object().is_none() {
        return None;
    }
    let nodes_v = vm_data::field(vm, value, live_id!(nodes));
    let links_v = vm_data::field(vm, value, live_id!(links));
    let node_items = vm_data::elements(vm, nodes_v)?;
    let mut names = Vec::with_capacity(node_items.len());
    for (i, item) in node_items.iter().enumerate() {
        let name_v = vm_data::field(vm, *item, live_id!(name));
        if name_v.is_nil() {
            names.push(vm_data::to_string_cast(vm, *item));
        } else {
            names.push(vm_data::to_string_cast(vm, name_v));
        }
        if names[i].is_empty() {
            names[i] = format!("{i}");
        }
    }
    let mut links = Vec::new();
    if let Some(link_items) = vm_data::elements(vm, links_v) {
        for item in link_items {
            let (s, t, v) = if item.as_object().is_some()
                && !vm_data::field(vm, item, live_id!(source)).is_nil()
            {
                (
                    vm_data::field(vm, item, live_id!(source)).as_number(),
                    vm_data::field(vm, item, live_id!(target)).as_number(),
                    vm_data::field(vm, item, live_id!(value))
                        .as_number()
                        .unwrap_or(1.0),
                )
            } else if let Some(parts) = vm_data::elements(vm, item) {
                (
                    parts.first().and_then(|v| v.as_number()),
                    parts.get(1).and_then(|v| v.as_number()),
                    parts.get(2).and_then(|v| v.as_number()).unwrap_or(1.0),
                )
            } else {
                (None, None, 1.0)
            };
            if let (Some(s), Some(t)) = (s, t) {
                let (s, t) = (s as usize, t as usize);
                if s < names.len() && t < names.len() && s != t && v > 0.0 {
                    links.push((s, t, v));
                }
            }
        }
    }
    if names.is_empty() {
        return None;
    }
    Some(GraphData { names, links })
}

pub(crate) fn demo_graph() -> GraphData {
    GraphData {
        names: ["Coal", "Gas", "Solar", "Power", "Industry", "Homes"]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        links: vec![
            (0, 3, 10.0),
            (1, 3, 8.0),
            (2, 3, 4.0),
            (3, 4, 13.0),
            (3, 5, 9.0),
            (1, 4, 3.0),
        ],
    }
}

fn cat_color(i: usize, alpha: f32) -> Vec4f {
    let mut c = rgba_to_vec4f(CategoricalScale::category10().get(i));
    c.w = alpha;
    c
}

// ================= Sankey =================

struct SankeyLayout {
    /// per node: (x, y, height, layer)
    nodes: Vec<(f64, f64, f64, usize)>,
    /// per link: (source, target, thickness, sy, ty)
    links: Vec<(usize, usize, f64, f64, f64)>,
}

fn sankey_layout(graph: &GraphData, w: f64, h: f64, node_w: f64, pad: f64) -> SankeyLayout {
    let n = graph.names.len();

    // Node values: max(in, out); pure sources/sinks take the non-zero side.
    let mut in_sum = vec![0.0; n];
    let mut out_sum = vec![0.0; n];
    for &(s, t, v) in &graph.links {
        out_sum[s] += v;
        in_sum[t] += v;
    }
    let value: Vec<f64> = (0..n)
        .map(|i| in_sum[i].max(out_sum[i]).max(f64::EPSILON))
        .collect();

    // Layering: longest path from sources (bounded relaxation).
    let mut layer = vec![0usize; n];
    for _ in 0..n {
        let mut changed = false;
        for &(s, t, _) in &graph.links {
            if layer[t] < layer[s] + 1 {
                layer[t] = layer[s] + 1;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    let max_layer = layer.iter().copied().max().unwrap_or(0);
    // Justify: sinks move to the last layer.
    for i in 0..n {
        if out_sum[i] == 0.0 {
            layer[i] = max_layer;
        }
    }

    // Vertical scale from the densest layer.
    let mut ky = f64::INFINITY;
    for l in 0..=max_layer {
        let members: Vec<usize> = (0..n).filter(|&i| layer[i] == l).collect();
        if members.is_empty() {
            continue;
        }
        let sum: f64 = members.iter().map(|&i| value[i]).sum();
        let avail = h - (members.len().saturating_sub(1)) as f64 * pad;
        if sum > 0.0 {
            ky = ky.min((avail / sum).max(0.1));
        }
    }
    if !ky.is_finite() {
        ky = 1.0;
    }

    let lx = if max_layer > 0 {
        (w - node_w) / max_layer as f64
    } else {
        0.0
    };
    let x: Vec<f64> = (0..n).map(|i| layer[i] as f64 * lx).collect();
    let height: Vec<f64> = (0..n).map(|i| value[i] * ky).collect();

    // Initial stacking per layer, then a few relaxation passes.
    let mut y = vec![0.0; n];
    for l in 0..=max_layer {
        let mut cursor = 0.0;
        for i in 0..n {
            if layer[i] == l {
                y[i] = cursor;
                cursor += height[i] + pad;
            }
        }
    }
    for pass in 0..6 {
        let forward = pass % 2 == 0;
        for i in 0..n {
            let mut weighted = 0.0;
            let mut total = 0.0;
            for &(s, t, v) in &graph.links {
                if forward && t == i {
                    weighted += (y[s] + height[s] * 0.5) * v;
                    total += v;
                } else if !forward && s == i {
                    weighted += (y[t] + height[t] * 0.5) * v;
                    total += v;
                }
            }
            if total > 0.0 {
                y[i] = (weighted / total - height[i] * 0.5).max(0.0);
            }
        }
        // Resolve collisions per layer.
        for l in 0..=max_layer {
            let mut members: Vec<usize> = (0..n).filter(|&i| layer[i] == l).collect();
            members.sort_by(|&a, &b| y[a].partial_cmp(&y[b]).unwrap_or(std::cmp::Ordering::Equal));
            let mut cursor = 0.0_f64;
            for &i in &members {
                if y[i] < cursor {
                    y[i] = cursor;
                }
                cursor = y[i] + height[i] + pad;
            }
            // Clamp back into the canvas from the bottom.
            let overflow = cursor - pad - h;
            if overflow > 0.0 {
                let mut floor = h;
                for &i in members.iter().rev() {
                    y[i] = (y[i] - overflow).min(floor - height[i]).max(0.0);
                    floor = y[i] - pad;
                }
            }
        }
    }

    // Link slots: offsets within source/target nodes, ordered by far-end y.
    let mut links_out: Vec<(usize, usize, f64, f64, f64)> = Vec::with_capacity(graph.links.len());
    let mut order: Vec<usize> = (0..graph.links.len()).collect();
    order.sort_by(|&a, &b| {
        let ka = (graph.links[a].0, (y[graph.links[a].1] * 16.0) as i64);
        let kb = (graph.links[b].0, (y[graph.links[b].1] * 16.0) as i64);
        ka.cmp(&kb)
    });
    let mut s_cursor = vec![0.0; n];
    let mut slots: Vec<(f64, f64)> = vec![(0.0, 0.0); graph.links.len()];
    for &li in &order {
        let (s, _, v) = graph.links[li];
        let th = v * ky;
        slots[li].0 = y[s] + s_cursor[s] + th * 0.5;
        s_cursor[s] += th;
    }
    order.sort_by(|&a, &b| {
        let ka = (graph.links[a].1, (y[graph.links[a].0] * 16.0) as i64);
        let kb = (graph.links[b].1, (y[graph.links[b].0] * 16.0) as i64);
        ka.cmp(&kb)
    });
    let mut t_cursor = vec![0.0; n];
    for &li in &order {
        let (_, t, v) = graph.links[li];
        let th = v * ky;
        slots[li].1 = y[t] + t_cursor[t] + th * 0.5;
        t_cursor[t] += th;
    }
    for (li, &(s, t, v)) in graph.links.iter().enumerate() {
        links_out.push((s, t, v * ky, slots[li].0, slots[li].1));
    }

    SankeyLayout {
        nodes: (0..n).map(|i| (x[i], y[i], height[i], layer[i])).collect(),
        links: links_out,
    }
}

/// Directed flow diagram, exposed as `d3.Sankey`.
#[derive(Script, Widget)]
pub struct D3Sankey {
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

    /// Node bar width.
    #[live(14.0)]
    pub node_width: f32,
    /// Vertical padding between nodes in a layer.
    #[live(10.0)]
    pub node_padding: f32,
    /// Ribbon opacity.
    #[live(0.35)]
    pub link_alpha: f32,
    /// Margins around the plot area.
    #[live]
    pub plot_margin: Inset,

    /// Fired with the node index on click.
    #[live]
    on_click: ScriptFnRef,

    #[rust]
    graph: Option<GraphData>,
    #[rust]
    rect: Rect,
    #[rust]
    plot: Rect,
    #[rust]
    node_rects: Vec<Rect>,
}

impl D3Sankey {
    /// Replace the graph from Rust.
    pub fn set_graph(&mut self, graph: GraphData) {
        self.graph = Some(graph);
    }
}

impl ScriptHook for D3Sankey {
    fn on_after_apply(
        &mut self,
        vm: &mut ScriptVm,
        _apply: &Apply,
        _scope: &mut Scope,
        _value: ScriptValue,
    ) {
        if let Some(graph) = parse_graph(vm, self.data) {
            self.graph = Some(graph);
        }
    }
}

impl Widget for D3Sankey {
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
        if let Hit::FingerUp(fe) = event.hits(cx, self.draw_bg.area()) {
            if fe.is_over && fe.is_primary_hit() {
                if let Some(i) = self.node_rects.iter().position(|r| r.contains(fe.abs)) {
                    fire_index_callback(cx, self.uid, &self.source, &self.on_click, i);
                }
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.graph.is_none() {
            self.graph = Some(demo_graph());
        }
        self.rect = cx.walk_turtle(walk);
        self.plot = compute_plot_rect(&self.rect, &self.plot_margin);
        self.draw_bg.draw_abs(cx, self.rect);

        begin_plot(cx, &self.rect, &self.plot_margin, &mut self.draw_vector);

        let graph = self.graph.as_ref().unwrap();
        let layout = sankey_layout(
            graph,
            self.plot.size.x,
            self.plot.size.y,
            self.node_width as f64,
            self.node_padding as f64,
        );
        let ox = self.plot.pos.x;
        let oy = self.plot.pos.y;
        let nw = self.node_width as f64;

        // Ribbons first (under the node bars).
        for &(s, t, th, sy, ty) in &layout.links {
            let x0 = ox + layout.nodes[s].0 + nw;
            let x1 = ox + layout.nodes[t].0;
            let y0 = oy + sy;
            let y1 = oy + ty;
            let c = cat_color(s % 10, self.link_alpha);
            self.draw_vector.set_color(c.x, c.y, c.z, c.w);
            // Sampled cubic band: top edge out, bottom edge back.
            let half = th * 0.5;
            let segs = 24;
            let bez = |t: f64| -> (f64, f64) {
                let u = 1.0 - t;
                let bx = u * u * u * x0
                    + 3.0 * u * u * t * (x0 + (x1 - x0) * 0.5)
                    + 3.0 * u * t * t * (x1 - (x1 - x0) * 0.5)
                    + t * t * t * x1;
                let by =
                    u * u * u * y0 + 3.0 * u * u * t * y0 + 3.0 * u * t * t * y1 + t * t * t * y1;
                (bx, by)
            };
            let (px, py) = bez(0.0);
            self.draw_vector.move_to(px as f32, (py - half) as f32);
            for k in 1..=segs {
                let (px, py) = bez(k as f64 / segs as f64);
                self.draw_vector.line_to(px as f32, (py - half) as f32);
            }
            for k in (0..=segs).rev() {
                let (px, py) = bez(k as f64 / segs as f64);
                self.draw_vector.line_to(px as f32, (py + half) as f32);
            }
            self.draw_vector.close();
            self.draw_vector.fill();
        }

        // Node bars.
        self.node_rects.clear();
        for (i, &(x, y, h, _)) in layout.nodes.iter().enumerate() {
            let r = Rect {
                pos: DVec2 {
                    x: ox + x,
                    y: oy + y,
                },
                size: DVec2 {
                    x: nw,
                    y: h.max(1.0),
                },
            };
            self.node_rects.push(r);
            let c = cat_color(i % 10, 1.0);
            self.draw_vector.set_color(c.x, c.y, c.z, c.w);
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

// ================= Chord diagram =================

/// Circular flow matrix diagram, exposed as `d3.ChordDiagram`.
#[derive(Script, Widget)]
pub struct D3ChordDiagram {
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

    /// Declarative data: square flow matrix (rows of numbers).
    #[live]
    data: ScriptValue,
    /// Ribbon opacity.
    #[live(0.55)]
    pub ribbon_alpha: f32,
    /// Margins around the plot area.
    #[live]
    pub plot_margin: Inset,

    #[rust]
    matrix: Vec<Vec<f64>>,
    #[rust]
    rect: Rect,
    #[rust]
    plot: Rect,
}

impl D3ChordDiagram {
    /// Replace the flow matrix from Rust.
    pub fn set_matrix(&mut self, matrix: Vec<Vec<f64>>) {
        self.matrix = matrix;
    }

    fn ensure_demo_data(&mut self) {
        if self.matrix.is_empty() {
            self.matrix = vec![
                vec![0.0, 6.0, 3.0, 1.0],
                vec![2.0, 0.0, 5.0, 4.0],
                vec![4.0, 1.0, 0.0, 3.0],
                vec![2.0, 3.0, 2.0, 0.0],
            ];
        }
    }
}

impl ScriptHook for D3ChordDiagram {
    fn on_after_apply(
        &mut self,
        vm: &mut ScriptVm,
        _apply: &Apply,
        _scope: &mut Scope,
        _value: ScriptValue,
    ) {
        if let Some(rows) = vm_data::to_rows(vm, self.data) {
            if !rows.is_empty() {
                self.matrix = rows;
            }
        }
    }
}

impl Widget for D3ChordDiagram {
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
                    self.set_matrix(rows);
                    vm.with_cx_mut(|cx| self.redraw(cx));
                }
            }
            return ScriptAsyncResult::Return(NIL);
        }
        ScriptAsyncResult::MethodNotFound
    }

    fn handle_event(&mut self, _cx: &mut Cx, _event: &Event, _scope: &mut Scope) {}

    #[allow(clippy::needless_range_loop)]
    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        self.ensure_demo_data();
        self.rect = cx.walk_turtle(walk);
        self.plot = compute_plot_rect(&self.rect, &self.plot_margin);
        self.draw_bg.draw_abs(cx, self.rect);

        begin_plot(cx, &self.rect, &self.plot_margin, &mut self.draw_vector);

        let n = self.matrix.len();
        let center = DVec2 {
            x: self.plot.pos.x + self.plot.size.x * 0.5,
            y: self.plot.pos.y + self.plot.size.y * 0.5,
        };
        let radius = (self.plot.size.x.min(self.plot.size.y) * 0.5 - 6.0).max(6.0);
        let r_in = radius - 10.0;

        // Group spans: proportional to row+column totals, with gaps.
        let totals: Vec<f64> = (0..n)
            .map(|i| {
                let row: f64 = self.matrix[i].iter().sum();
                let col: f64 = self
                    .matrix
                    .iter()
                    .map(|r| r.get(i).copied().unwrap_or(0.0))
                    .sum();
                row + col
            })
            .collect();
        let total: f64 = totals.iter().sum::<f64>().max(f64::EPSILON);
        let gap = 0.06_f64;
        let avail = std::f64::consts::TAU - gap * n as f64;

        let mut group_start = vec![0.0; n];
        let mut cursor = 0.0;
        for i in 0..n {
            group_start[i] = cursor;
            cursor += totals[i] / total * avail + gap;
        }
        let span = |i: usize| totals[i] / total * avail;

        let pt = |angle: f64, r: f64| -> (f32, f32) {
            (
                (center.x + r * angle.sin()) as f32,
                (center.y - r * angle.cos()) as f32,
            )
        };

        // Sub-arc cursor per group: outgoing (row) flows then incoming (col).
        let mut sub_cursor = group_start.clone();
        let unit: Vec<f64> = (0..n)
            .map(|i| {
                if totals[i] > 0.0 {
                    span(i) / totals[i]
                } else {
                    0.0
                }
            })
            .collect();
        // Allocate sub-spans deterministically: for pair (i, j), the flow
        // matrix[i][j] occupies a slice on i (outgoing) and on j (incoming).
        let mut out_spans = vec![vec![(0.0, 0.0); n]; n];
        let mut in_spans = vec![vec![(0.0, 0.0); n]; n];
        for i in 0..n {
            for j in 0..n {
                let v = self.matrix[i].get(j).copied().unwrap_or(0.0);
                if v > 0.0 {
                    let w = v * unit[i];
                    out_spans[i][j] = (sub_cursor[i], sub_cursor[i] + w);
                    sub_cursor[i] += w;
                }
            }
            for j in 0..n {
                let v = self.matrix[j].get(i).copied().unwrap_or(0.0);
                if v > 0.0 {
                    let w = v * unit[i];
                    in_spans[j][i] = (sub_cursor[i], sub_cursor[i] + w);
                    sub_cursor[i] += w;
                }
            }
        }

        // Ribbons.
        for i in 0..n {
            for j in 0..n {
                let v = self.matrix[i].get(j).copied().unwrap_or(0.0);
                if v <= 0.0 {
                    continue;
                }
                let (sa0, sa1) = out_spans[i][j];
                let (ta0, ta1) = in_spans[i][j];
                let c = cat_color(i % 10, self.ribbon_alpha);
                self.draw_vector.set_color(c.x, c.y, c.z, c.w);

                let arc = |dv: &mut DrawVector, a0: f64, a1: f64, first: bool| {
                    let sweep = a1 - a0;
                    let segs = ((sweep.abs() / 0.08).ceil() as usize).clamp(1, 128);
                    for k in 0..=segs {
                        let (px, py) = pt(a0 + sweep * k as f64 / segs as f64, r_in);
                        if first && k == 0 {
                            dv.move_to(px, py);
                        } else {
                            dv.line_to(px, py);
                        }
                    }
                };
                arc(&mut self.draw_vector, sa0, sa1, true);
                // Quadratic through the center to the target arc.
                let (tx, ty_) = pt(ta0, r_in);
                self.draw_vector
                    .quad_to(center.x as f32, center.y as f32, tx, ty_);
                arc(&mut self.draw_vector, ta0, ta1, false);
                let (sx, sy_) = pt(sa0, r_in);
                self.draw_vector
                    .quad_to(center.x as f32, center.y as f32, sx, sy_);
                self.draw_vector.close();
                self.draw_vector.fill();
            }
        }

        // Group arcs (annular segments) on top.
        for i in 0..n {
            let a0 = group_start[i];
            let a1 = a0 + span(i);
            let c = cat_color(i % 10, 1.0);
            self.draw_vector.set_color(c.x, c.y, c.z, c.w);
            let sweep = a1 - a0;
            let segs = ((sweep / 0.05).ceil() as usize).clamp(2, 256);
            let step = sweep / segs as f64;
            let (sx, sy_) = pt(a0, radius);
            self.draw_vector.move_to(sx, sy_);
            for k in 1..=segs {
                let (px, py) = pt(a0 + step * k as f64, radius);
                self.draw_vector.line_to(px, py);
            }
            for k in (0..=segs).rev() {
                let (px, py) = pt(a0 + step * k as f64, r_in + 2.0);
                self.draw_vector.line_to(px, py);
            }
            self.draw_vector.close();
            self.draw_vector.fill();
        }

        end_plot(cx, &mut self.draw_vector);
        DrawStep::done()
    }
}

// ================= Arc diagram =================

/// Linear node row with arched links, exposed as `d3.ArcDiagram`.
#[derive(Script, Widget)]
pub struct D3ArcDiagram {
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
    /// Node dot radius.
    #[live(5.0)]
    pub node_radius: f32,
    /// Arc opacity.
    #[live(0.5)]
    pub arc_alpha: f32,
    /// Margins around the plot area.
    #[live]
    pub plot_margin: Inset,

    /// Fired with the node index on click.
    #[live]
    on_click: ScriptFnRef,

    #[rust]
    graph: Option<GraphData>,
    #[rust]
    rect: Rect,
    #[rust]
    plot: Rect,
}

impl D3ArcDiagram {
    /// Replace the graph from Rust.
    pub fn set_graph(&mut self, graph: GraphData) {
        self.graph = Some(graph);
    }

    fn node_x(&self, i: usize, n: usize) -> f64 {
        if n <= 1 {
            self.plot.pos.x + self.plot.size.x * 0.5
        } else {
            self.plot.pos.x + self.plot.size.x * i as f64 / (n - 1) as f64
        }
    }

    fn baseline(&self) -> f64 {
        self.plot.pos.y + self.plot.size.y - 10.0
    }
}

impl ScriptHook for D3ArcDiagram {
    fn on_after_apply(
        &mut self,
        vm: &mut ScriptVm,
        _apply: &Apply,
        _scope: &mut Scope,
        _value: ScriptValue,
    ) {
        if let Some(graph) = parse_graph(vm, self.data) {
            self.graph = Some(graph);
        }
    }
}

impl Widget for D3ArcDiagram {
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
        if let Hit::FingerUp(fe) = event.hits(cx, self.draw_bg.area()) {
            if fe.is_over && fe.is_primary_hit() {
                if let Some(graph) = &self.graph {
                    let n = graph.names.len();
                    let y = self.baseline();
                    let rr = (self.node_radius as f64 + 6.0).powi(2);
                    for i in 0..n {
                        let x = self.node_x(i, n);
                        let d = (x - fe.abs.x).powi(2) + (y - fe.abs.y).powi(2);
                        if d <= rr {
                            fire_index_callback(cx, self.uid, &self.source, &self.on_click, i);
                            break;
                        }
                    }
                }
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.graph.is_none() {
            self.graph = Some(demo_graph());
        }
        self.rect = cx.walk_turtle(walk);
        self.plot = compute_plot_rect(&self.rect, &self.plot_margin);
        self.draw_bg.draw_abs(cx, self.rect);

        begin_plot(cx, &self.rect, &self.plot_margin, &mut self.draw_vector);

        let graph = self.graph.as_ref().unwrap();
        let n = graph.names.len();
        let y = self.baseline();
        let max_v = graph.links.iter().map(|l| l.2).fold(1.0_f64, f64::max);

        // Arches: half-ellipse cubic approximation above the baseline.
        for &(s, t, v) in &graph.links {
            let x0 = self.node_x(s.min(t), n);
            let x1 = self.node_x(s.max(t), n);
            let h = ((x1 - x0) * 0.5).min(self.plot.size.y - 24.0);
            let cy = y - h * 4.0 / 3.0;
            let c = cat_color(s % 10, self.arc_alpha);
            self.draw_vector.set_color(c.x, c.y, c.z, c.w);
            self.draw_vector.move_to(x0 as f32, y as f32);
            self.draw_vector.bezier_to(
                x0 as f32, cy as f32, x1 as f32, cy as f32, x1 as f32, y as f32,
            );
            self.draw_vector.stroke((1.0 + 3.0 * v / max_v) as f32);
        }

        // Node dots.
        for i in 0..n {
            let c = cat_color(i % 10, 1.0);
            self.draw_vector.set_color(c.x, c.y, c.z, c.w);
            self.draw_vector
                .circle(self.node_x(i, n) as f32, y as f32, self.node_radius);
            self.draw_vector.fill();
        }

        end_plot(cx, &mut self.draw_vector);
        DrawStep::done()
    }
}

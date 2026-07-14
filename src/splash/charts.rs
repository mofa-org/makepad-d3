//! Chart widgets registered under the `d3.*` Splash namespace.
//!
//! Every widget follows the same contract:
//!
//! - **Declarative data**: `data:` / `labels:` properties on the script
//!   object, parsed at apply time (covers instantiation, hot reload, and
//!   runtime re-apply).
//! - **Imperative data**: `set_data(values)`, `set_labels(labels)`,
//!   `set_domain(min, max)`, and `data()` script methods.
//! - **Events**: `on_click: |index| ...` and `on_hover: |index| ...`
//!   script closures.
//! - **Rendering**: one GPU vector session per frame (`DrawVector`),
//!   grid/axis ticks from the d3 scale core, labels via `DrawText`.
//!
//! Charts with no data render a small built-in demo dataset so a bare
//! `d3.BarChart{}` in a Splash body shows something immediately.

// The `script_mod!` and `derive(Script, Widget)` macros generate public
// registration items that cannot carry doc comments.
#![allow(missing_docs)]

use crate::color::CategoricalScale;
use crate::scale::{CategoryScale, DiscreteScale, LinearScale, Scale, ScaleExt, TickOptions};
use crate::shape::{PieLayout, PieSlice};

use super::vm_data;
use makepad_widgets::makepad_script::ScriptFnRef;
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*

    mod.d3.BarChartBase = #(D3BarChart::register_widget(vm))
    mod.d3.BarChart = set_type_default() do mod.d3.BarChartBase{
        width: Fill
        height: 300
        plot_margin: Inset{left: 46.0, top: 12.0, right: 12.0, bottom: 30.0}
        bar_color: #x1f77b4
        hover_color: #x4fa3d4
        grid_color: #x3a3f52
        label_color: #x9aa0b0
        draw_bg +: { draw_depth: 0.0 color: #x00000000 }
        draw_grid +: { draw_depth: 0.1 }
        draw_vector +: { draw_depth: 2.0 }
        draw_text +: { draw_depth: 3.0 }
    }

    mod.d3.LineChartBase = #(D3LineChart::register_widget(vm))
    mod.d3.LineChart = set_type_default() do mod.d3.LineChartBase{
        width: Fill
        height: 300
        plot_margin: Inset{left: 46.0, top: 12.0, right: 12.0, bottom: 30.0}
        line_color: #x1f77b4
        line_width: 2.0
        dot_color: #xff7f0e
        dot_radius: 3.0
        grid_color: #x3a3f52
        label_color: #x9aa0b0
        draw_bg +: { draw_depth: 0.0 color: #x00000000 }
        draw_grid +: { draw_depth: 0.1 }
        draw_vector +: { draw_depth: 2.0 }
        draw_text +: { draw_depth: 3.0 }
    }

    mod.d3.AreaChartBase = #(D3AreaChart::register_widget(vm))
    mod.d3.AreaChart = set_type_default() do mod.d3.AreaChartBase{
        width: Fill
        height: 300
        plot_margin: Inset{left: 46.0, top: 12.0, right: 12.0, bottom: 30.0}
        line_color: #x1f77b4
        line_width: 2.0
        fill_color: #x1f77b455
        grid_color: #x3a3f52
        label_color: #x9aa0b0
        draw_bg +: { draw_depth: 0.0 color: #x00000000 }
        draw_grid +: { draw_depth: 0.1 }
        draw_vector +: { draw_depth: 2.0 }
        draw_text +: { draw_depth: 3.0 }
    }

    mod.d3.ScatterChartBase = #(D3ScatterChart::register_widget(vm))
    mod.d3.ScatterChart = set_type_default() do mod.d3.ScatterChartBase{
        width: Fill
        height: 300
        plot_margin: Inset{left: 46.0, top: 12.0, right: 12.0, bottom: 30.0}
        dot_color: #x2ca02c
        hover_color: #xd62728
        dot_radius: 4.0
        grid_color: #x3a3f52
        label_color: #x9aa0b0
        draw_bg +: { draw_depth: 0.0 color: #x00000000 }
        draw_grid +: { draw_depth: 0.1 }
        draw_vector +: { draw_depth: 2.0 }
        draw_text +: { draw_depth: 3.0 }
    }

    mod.d3.PieChartBase = #(D3PieChart::register_widget(vm))
    mod.d3.PieChart = set_type_default() do mod.d3.PieChartBase{
        width: Fill
        height: 300
        plot_margin: Inset{left: 12.0, top: 12.0, right: 12.0, bottom: 12.0}
        inner_radius: 0.0
        label_color: #x9aa0b0
        draw_bg +: { draw_depth: 0.0 color: #x00000000 }
        draw_grid +: { draw_depth: 0.1 }
        draw_vector +: { draw_depth: 2.0 }
        draw_text +: { draw_depth: 3.0 }
    }

    // Make `d3.` resolvable wherever the widgets prelude is imported —
    // Splash bodies open with `use mod.prelude.widgets.*`, so this line is
    // what lets them write `d3.BarChart{...}` without extra imports.
    mod.prelude.widgets.d3 = mod.d3
}

// ---- Shared helpers ----

pub(crate) fn rgba_to_vec4f(c: crate::color::Rgba) -> Vec4f {
    Vec4f {
        x: c.r,
        y: c.g,
        z: c.b,
        w: c.a,
    }
}

pub(crate) fn compute_plot_rect(rect: &Rect, m: &Inset) -> Rect {
    Rect {
        pos: DVec2 {
            x: rect.pos.x + m.left,
            y: rect.pos.y + m.top,
        },
        size: DVec2 {
            x: (rect.size.x - m.left - m.right).max(1.0),
            y: (rect.size.y - m.top - m.bottom).max(1.0),
        },
    }
}

/// Build the vertical (value) scale: pixel range is bottom -> top.
pub(crate) fn y_scale_for(
    data_min: f64,
    data_max: f64,
    explicit: Option<(f64, f64)>,
    include_zero: bool,
    plot: &Rect,
) -> LinearScale {
    let (min, max) = explicit.unwrap_or_else(|| {
        let mut min = data_min;
        let mut max = data_max;
        if include_zero {
            min = min.min(0.0);
            max = max.max(0.0);
        }
        if min.partial_cmp(&max) != Some(std::cmp::Ordering::Less) {
            let v = if min.is_finite() { min } else { 0.0 };
            min = v - 1.0;
            max = v + 1.0;
        }
        (min, max)
    });
    LinearScale::new()
        .with_domain(min, max)
        .with_nice(explicit.is_none())
        .with_range(plot.pos.y + plot.size.y, plot.pos.y)
}

/// Horizontal grid lines + value labels down the left margin.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_y_grid(
    grid: &mut DrawColor,
    text: &mut DrawText,
    cx: &mut Cx2d,
    scale: &LinearScale,
    rect: &Rect,
    plot: &Rect,
    grid_color: Vec4f,
    label_color: Vec4f,
) {
    grid.color = grid_color;
    text.color = label_color;
    for tick in scale.ticks(&TickOptions::default().with_count(6)) {
        let py = scale.scale(tick.value);
        grid.draw_abs(
            cx,
            Rect {
                pos: DVec2 {
                    x: plot.pos.x,
                    y: py,
                },
                size: DVec2 {
                    x: plot.size.x,
                    y: 1.0,
                },
            },
        );
        text.draw_abs(cx, dvec2(rect.pos.x + 4.0, py - 6.0), &tick.label);
    }
}

/// Vertical grid lines + numeric labels along the bottom margin.
pub(crate) fn draw_x_grid_linear(
    grid: &mut DrawColor,
    text: &mut DrawText,
    cx: &mut Cx2d,
    scale: &LinearScale,
    plot: &Rect,
    grid_color: Vec4f,
    label_color: Vec4f,
) {
    grid.color = grid_color;
    text.color = label_color;
    let label_y = plot.pos.y + plot.size.y + 6.0;
    for tick in scale.ticks(&TickOptions::default().with_count(8)) {
        let px = scale.scale(tick.value);
        grid.draw_abs(
            cx,
            Rect {
                pos: DVec2 {
                    x: px,
                    y: plot.pos.y,
                },
                size: DVec2 {
                    x: 1.0,
                    y: plot.size.y,
                },
            },
        );
        text.draw_abs(
            cx,
            dvec2(px - 3.0 * tick.label.len() as f64, label_y),
            &tick.label,
        );
    }
}

/// Category labels centered under their bands (thinned when crowded).
pub(crate) fn draw_x_labels_category(
    text: &mut DrawText,
    cx: &mut Cx2d,
    scale: &CategoryScale,
    labels: &[String],
    plot: &Rect,
    label_color: Vec4f,
) {
    text.color = label_color;
    let n = labels.len().max(1);
    let stride = n.div_ceil(16);
    let label_y = plot.pos.y + plot.size.y + 6.0;
    for (i, label) in labels.iter().enumerate() {
        if i % stride != 0 {
            continue;
        }
        let px = scale.scale_index(i);
        text.draw_abs(cx, dvec2(px - 3.0 * label.len() as f64, label_y), label);
    }
}

/// Begin the clipped plot turtle + vector session (mirrors the built-in
/// ChartView drawing discipline: bg at depth 0, vectors at 2, text at 3).
pub(crate) fn begin_plot(cx: &mut Cx2d, rect: &Rect, margin: &Inset, vector: &mut DrawVector) {
    cx.begin_turtle(
        Walk {
            abs_pos: Some(rect.pos),
            width: Size::Fixed(rect.size.x),
            height: Size::Fixed(rect.size.y),
            margin: Inset::default(),
            metrics: Metrics::default(),
        },
        Layout {
            clip_x: true,
            clip_y: true,
            padding: *margin,
            ..Layout::default()
        },
    );
    vector.begin();
}

pub(crate) fn end_plot(cx: &mut Cx2d, vector: &mut DrawVector) {
    vector.end(cx);
    cx.end_turtle();
}

pub(crate) fn fire_index_callback(
    cx: &mut Cx,
    uid: WidgetUid,
    source: &ScriptObjectRef,
    fn_ref: &ScriptFnRef,
    index: usize,
) {
    cx.widget_to_script_call(
        uid,
        NIL,
        source.clone(),
        fn_ref.clone(),
        &[(index as f64).into()],
    );
}

pub(crate) fn min_max(values: impl Iterator<Item = f64>) -> (f64, f64) {
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    for v in values {
        if v < min {
            min = v;
        }
        if v > max {
            max = v;
        }
    }
    (min, max)
}

pub(crate) const DEMO_VALUES: &[f64] = &[30.0, 86.0, 168.0, 281.0, 303.0, 365.0];
pub(crate) const DEMO_LABELS: &[&str] = &["A", "B", "C", "D", "E", "F"];

pub(crate) fn demo_xy(n: usize) -> Vec<(f64, f64)> {
    (0..n)
        .map(|i| {
            let x = i as f64;
            (x, 50.0 + 30.0 * (x * 0.35).sin() + 12.0 * (x * 0.9).cos())
        })
        .collect()
}

// ================= Bar chart =================

/// A categorical bar chart, exposed to Splash as `d3.BarChart`.
#[derive(Script, Widget)]
pub struct D3BarChart {
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

    /// Declarative data: array of numbers.
    #[live]
    data: ScriptValue,
    /// Declarative category labels: array of strings.
    #[live]
    labels: ScriptValue,

    /// Fill color of the bars.
    #[live]
    pub bar_color: Vec4f,
    /// Fill color of the hovered bar.
    #[live]
    pub hover_color: Vec4f,
    /// Grid line color.
    #[live]
    pub grid_color: Vec4f,
    /// Axis label color.
    #[live]
    pub label_color: Vec4f,
    /// Margins around the plot area.
    #[live]
    pub plot_margin: Inset,

    /// Fired with the bar index when a bar is clicked.
    #[live]
    on_click: ScriptFnRef,
    /// Fired with the bar index when the pointer moves onto a bar.
    #[live]
    on_hover: ScriptFnRef,

    #[rust]
    values: Vec<f64>,
    #[rust]
    label_strs: Vec<String>,
    #[rust]
    y_domain: Option<(f64, f64)>,
    #[rust]
    rect: Rect,
    #[rust]
    plot: Rect,
    #[rust]
    hovered: Option<usize>,
}

impl D3BarChart {
    /// Replace the chart data from Rust.
    pub fn set_values(&mut self, values: Vec<f64>) {
        self.values = values;
        self.hovered = None;
    }

    fn ensure_demo_data(&mut self) {
        if self.values.is_empty() {
            self.values = DEMO_VALUES.to_vec();
            if self.label_strs.is_empty() {
                self.label_strs = DEMO_LABELS.iter().map(|s| s.to_string()).collect();
            }
        }
    }

    fn scales(&self) -> (CategoryScale, LinearScale) {
        let n = self.values.len();
        let labels: Vec<String> = (0..n)
            .map(|i| {
                self.label_strs
                    .get(i)
                    .cloned()
                    .unwrap_or_else(|| format!("{i}"))
            })
            .collect();
        let x = CategoryScale::new()
            .with_labels(labels)
            .with_range(self.plot.pos.x, self.plot.pos.x + self.plot.size.x)
            .with_padding(0.15);
        let (min, max) = min_max(self.values.iter().copied());
        let y = y_scale_for(min, max, self.y_domain, true, &self.plot);
        (x, y)
    }

    fn bar_at(&self, abs: DVec2) -> Option<usize> {
        if self.values.is_empty() || !self.plot.contains(abs) {
            return None;
        }
        let (x_scale, _) = self.scales();
        let bw = x_scale.bandwidth();
        (0..self.values.len()).find(|&i| {
            let x0 = x_scale.band_start(i);
            abs.x >= x0 && abs.x <= x0 + bw
        })
    }
}

impl ScriptHook for D3BarChart {
    fn on_after_apply(
        &mut self,
        vm: &mut ScriptVm,
        _apply: &Apply,
        _scope: &mut Scope,
        _value: ScriptValue,
    ) {
        if let Some(values) = vm_data::to_f64_vec(vm, self.data) {
            self.values = values;
        }
        if let Some(labels) = vm_data::to_string_vec(vm, self.labels) {
            self.label_strs = labels;
        }
    }
}

impl Widget for D3BarChart {
    fn script_call(
        &mut self,
        vm: &mut ScriptVm,
        method: LiveId,
        args: ScriptValue,
    ) -> ScriptAsyncResult {
        if method == live_id!(set_data) {
            let value = vm_data::arg(vm, args, 0);
            if let Some(values) = vm_data::to_f64_vec(vm, value) {
                self.set_values(values);
                vm.with_cx_mut(|cx| self.redraw(cx));
            }
            return ScriptAsyncResult::Return(NIL);
        }
        if method == live_id!(set_labels) {
            let value = vm_data::arg(vm, args, 0);
            if let Some(labels) = vm_data::to_string_vec(vm, value) {
                self.label_strs = labels;
                vm.with_cx_mut(|cx| self.redraw(cx));
            }
            return ScriptAsyncResult::Return(NIL);
        }
        if method == live_id!(set_domain) {
            let min = vm_data::arg(vm, args, 0).as_number();
            let max = vm_data::arg(vm, args, 1).as_number();
            self.y_domain = match (min, max) {
                (Some(min), Some(max)) if min < max => Some((min, max)),
                _ => None,
            };
            vm.with_cx_mut(|cx| self.redraw(cx));
            return ScriptAsyncResult::Return(NIL);
        }
        if method == live_id!(data) {
            let arr = vm_data::f64_slice_to_array(vm, &self.values);
            return ScriptAsyncResult::Return(arr);
        }
        ScriptAsyncResult::MethodNotFound
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        match event.hits(cx, self.draw_bg.area()) {
            Hit::FingerHoverIn(fe) | Hit::FingerHoverOver(fe) => {
                let hit = self.bar_at(fe.abs);
                if hit != self.hovered {
                    self.hovered = hit;
                    if let Some(i) = hit {
                        fire_index_callback(cx, self.uid, &self.source, &self.on_hover, i);
                    }
                    self.redraw(cx);
                }
                cx.set_cursor(if hit.is_some() {
                    MouseCursor::Hand
                } else {
                    MouseCursor::Default
                });
            }
            Hit::FingerHoverOut(_) if self.hovered.is_some() => {
                self.hovered = None;
                self.redraw(cx);
            }
            Hit::FingerUp(fe) if fe.is_over && fe.is_primary_hit() => {
                if let Some(i) = self.bar_at(fe.abs) {
                    fire_index_callback(cx, self.uid, &self.source, &self.on_click, i);
                }
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

        let (x_scale, y_scale) = self.scales();
        let plot = self.plot;
        let rect = self.rect;
        draw_y_grid(
            &mut self.draw_grid,
            &mut self.draw_text,
            cx,
            &y_scale,
            &rect,
            &plot,
            self.grid_color,
            self.label_color,
        );
        draw_x_labels_category(
            &mut self.draw_text,
            cx,
            &x_scale,
            x_scale.labels(),
            &plot,
            self.label_color,
        );

        let baseline = y_scale.scale(y_scale.clamp_domain(0.0));
        let bw = x_scale.bandwidth();
        for (i, &v) in self.values.iter().enumerate() {
            let x0 = x_scale.band_start(i);
            let py = y_scale.scale(v);
            let top = py.min(baseline);
            let h = (py - baseline).abs().max(1.0);
            let color = if self.hovered == Some(i) {
                self.hover_color
            } else {
                self.bar_color
            };
            self.draw_vector
                .set_color(color.x, color.y, color.z, color.w);
            self.draw_vector
                .rect(x0 as f32, top as f32, bw as f32, h as f32);
            self.draw_vector.fill();
        }

        end_plot(cx, &mut self.draw_vector);
        DrawStep::done()
    }
}

// ================= Line chart =================

/// An x/y line chart, exposed to Splash as `d3.LineChart`.
#[derive(Script, Widget)]
pub struct D3LineChart {
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

    /// Declarative data: numbers, `[x y]` pairs, or `{x y}` objects.
    #[live]
    data: ScriptValue,

    /// Stroke color of the line.
    #[live]
    pub line_color: Vec4f,
    /// Stroke width of the line.
    #[live(2.0)]
    pub line_width: f32,
    /// Fill color of the hovered point marker.
    #[live]
    pub dot_color: Vec4f,
    /// Radius of the hovered point marker.
    #[live(3.0)]
    pub dot_radius: f32,
    /// Grid line color.
    #[live]
    pub grid_color: Vec4f,
    /// Axis label color.
    #[live]
    pub label_color: Vec4f,
    /// Margins around the plot area.
    #[live]
    pub plot_margin: Inset,

    /// Fired with the nearest point index when the chart is clicked.
    #[live]
    on_click: ScriptFnRef,
    /// Fired with the nearest point index while hovering.
    #[live]
    on_hover: ScriptFnRef,

    #[rust]
    points: Vec<(f64, f64)>,
    #[rust]
    y_domain: Option<(f64, f64)>,
    #[rust]
    rect: Rect,
    #[rust]
    plot: Rect,
    #[rust]
    hovered: Option<usize>,
}

impl D3LineChart {
    /// Replace the chart data from Rust.
    pub fn set_points(&mut self, points: Vec<(f64, f64)>) {
        self.points = points;
        self.hovered = None;
    }

    fn ensure_demo_data(&mut self) {
        if self.points.is_empty() {
            self.points = demo_xy(40);
        }
    }

    fn scales(&self) -> (LinearScale, LinearScale) {
        let (x_min, x_max) = min_max(self.points.iter().map(|p| p.0));
        let (y_min, y_max) = min_max(self.points.iter().map(|p| p.1));
        let x = LinearScale::new()
            .with_domain(x_min, if x_max > x_min { x_max } else { x_min + 1.0 })
            .with_range(self.plot.pos.x, self.plot.pos.x + self.plot.size.x);
        let y = y_scale_for(y_min, y_max, self.y_domain, false, &self.plot);
        (x, y)
    }

    fn nearest_point(&self, abs: DVec2) -> Option<usize> {
        if self.points.is_empty() || !self.plot.contains(abs) {
            return None;
        }
        let (x_scale, _) = self.scales();
        let mut best = None;
        let mut best_d = f64::INFINITY;
        for (i, p) in self.points.iter().enumerate() {
            let d = (x_scale.scale(p.0) - abs.x).abs();
            if d < best_d {
                best_d = d;
                best = Some(i);
            }
        }
        best
    }

    fn draw_series(&mut self, x_scale: &LinearScale, y_scale: &LinearScale) {
        if self.points.len() < 2 {
            return;
        }
        let c = self.line_color;
        self.draw_vector.set_color(c.x, c.y, c.z, c.w);
        let (px, py) = (
            x_scale.scale(self.points[0].0) as f32,
            y_scale.scale(self.points[0].1) as f32,
        );
        self.draw_vector.move_to(px, py);
        for p in &self.points[1..] {
            self.draw_vector
                .line_to(x_scale.scale(p.0) as f32, y_scale.scale(p.1) as f32);
        }
        self.draw_vector.stroke(self.line_width);

        if let Some(i) = self.hovered {
            if let Some(p) = self.points.get(i) {
                let c = self.dot_color;
                self.draw_vector.set_color(c.x, c.y, c.z, c.w);
                self.draw_vector.circle(
                    x_scale.scale(p.0) as f32,
                    y_scale.scale(p.1) as f32,
                    self.dot_radius,
                );
                self.draw_vector.fill();
            }
        }
    }
}

impl ScriptHook for D3LineChart {
    fn on_after_apply(
        &mut self,
        vm: &mut ScriptVm,
        _apply: &Apply,
        _scope: &mut Scope,
        _value: ScriptValue,
    ) {
        if let Some(points) = vm_data::to_xy_vec(vm, self.data) {
            self.points = points;
        }
    }
}

impl Widget for D3LineChart {
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
        if method == live_id!(set_domain) {
            let min = vm_data::arg(vm, args, 0).as_number();
            let max = vm_data::arg(vm, args, 1).as_number();
            self.y_domain = match (min, max) {
                (Some(min), Some(max)) if min < max => Some((min, max)),
                _ => None,
            };
            vm.with_cx_mut(|cx| self.redraw(cx));
            return ScriptAsyncResult::Return(NIL);
        }
        if method == live_id!(data) {
            let ys: Vec<f64> = self.points.iter().map(|p| p.1).collect();
            let arr = vm_data::f64_slice_to_array(vm, &ys);
            return ScriptAsyncResult::Return(arr);
        }
        ScriptAsyncResult::MethodNotFound
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        match event.hits(cx, self.draw_bg.area()) {
            Hit::FingerHoverIn(fe) | Hit::FingerHoverOver(fe) => {
                let hit = self.nearest_point(fe.abs);
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
                if let Some(i) = self.nearest_point(fe.abs) {
                    fire_index_callback(cx, self.uid, &self.source, &self.on_click, i);
                }
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

        let (x_scale, y_scale) = self.scales();
        let plot = self.plot;
        let rect = self.rect;
        draw_y_grid(
            &mut self.draw_grid,
            &mut self.draw_text,
            cx,
            &y_scale,
            &rect,
            &plot,
            self.grid_color,
            self.label_color,
        );
        draw_x_grid_linear(
            &mut self.draw_grid,
            &mut self.draw_text,
            cx,
            &x_scale,
            &plot,
            self.grid_color,
            self.label_color,
        );

        self.draw_series(&x_scale, &y_scale);

        end_plot(cx, &mut self.draw_vector);
        DrawStep::done()
    }
}

// ================= Area chart =================

/// An x/y area chart, exposed to Splash as `d3.AreaChart`.
#[derive(Script, Widget)]
pub struct D3AreaChart {
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

    /// Declarative data: numbers, `[x y]` pairs, or `{x y}` objects.
    #[live]
    data: ScriptValue,

    /// Stroke color of the top line.
    #[live]
    pub line_color: Vec4f,
    /// Stroke width of the top line.
    #[live(2.0)]
    pub line_width: f32,
    /// Fill color of the area below the line.
    #[live]
    pub fill_color: Vec4f,
    /// Grid line color.
    #[live]
    pub grid_color: Vec4f,
    /// Axis label color.
    #[live]
    pub label_color: Vec4f,
    /// Margins around the plot area.
    #[live]
    pub plot_margin: Inset,

    /// Fired with the nearest point index when the chart is clicked.
    #[live]
    on_click: ScriptFnRef,

    #[rust]
    points: Vec<(f64, f64)>,
    #[rust]
    y_domain: Option<(f64, f64)>,
    #[rust]
    rect: Rect,
    #[rust]
    plot: Rect,
}

impl D3AreaChart {
    /// Replace the chart data from Rust.
    pub fn set_points(&mut self, points: Vec<(f64, f64)>) {
        self.points = points;
    }

    fn ensure_demo_data(&mut self) {
        if self.points.is_empty() {
            self.points = demo_xy(40);
        }
    }

    fn scales(&self) -> (LinearScale, LinearScale) {
        let (x_min, x_max) = min_max(self.points.iter().map(|p| p.0));
        let (y_min, y_max) = min_max(self.points.iter().map(|p| p.1));
        let x = LinearScale::new()
            .with_domain(x_min, if x_max > x_min { x_max } else { x_min + 1.0 })
            .with_range(self.plot.pos.x, self.plot.pos.x + self.plot.size.x);
        let y = y_scale_for(y_min, y_max, self.y_domain, true, &self.plot);
        (x, y)
    }
}

impl ScriptHook for D3AreaChart {
    fn on_after_apply(
        &mut self,
        vm: &mut ScriptVm,
        _apply: &Apply,
        _scope: &mut Scope,
        _value: ScriptValue,
    ) {
        if let Some(points) = vm_data::to_xy_vec(vm, self.data) {
            self.points = points;
        }
    }
}

impl Widget for D3AreaChart {
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
        if method == live_id!(set_domain) {
            let min = vm_data::arg(vm, args, 0).as_number();
            let max = vm_data::arg(vm, args, 1).as_number();
            self.y_domain = match (min, max) {
                (Some(min), Some(max)) if min < max => Some((min, max)),
                _ => None,
            };
            vm.with_cx_mut(|cx| self.redraw(cx));
            return ScriptAsyncResult::Return(NIL);
        }
        if method == live_id!(data) {
            let ys: Vec<f64> = self.points.iter().map(|p| p.1).collect();
            let arr = vm_data::f64_slice_to_array(vm, &ys);
            return ScriptAsyncResult::Return(arr);
        }
        ScriptAsyncResult::MethodNotFound
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        if let Hit::FingerUp(fe) = event.hits(cx, self.draw_bg.area()) {
            if fe.is_over && fe.is_primary_hit() && !self.points.is_empty() {
                let (x_scale, _) = self.scales();
                let mut best = 0;
                let mut best_d = f64::INFINITY;
                for (i, p) in self.points.iter().enumerate() {
                    let d = (x_scale.scale(p.0) - fe.abs.x).abs();
                    if d < best_d {
                        best_d = d;
                        best = i;
                    }
                }
                fire_index_callback(cx, self.uid, &self.source, &self.on_click, best);
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        self.ensure_demo_data();
        self.rect = cx.walk_turtle(walk);
        self.plot = compute_plot_rect(&self.rect, &self.plot_margin);
        self.draw_bg.draw_abs(cx, self.rect);

        begin_plot(cx, &self.rect, &self.plot_margin, &mut self.draw_vector);

        let (x_scale, y_scale) = self.scales();
        let plot = self.plot;
        let rect = self.rect;
        draw_y_grid(
            &mut self.draw_grid,
            &mut self.draw_text,
            cx,
            &y_scale,
            &rect,
            &plot,
            self.grid_color,
            self.label_color,
        );
        draw_x_grid_linear(
            &mut self.draw_grid,
            &mut self.draw_text,
            cx,
            &x_scale,
            &plot,
            self.grid_color,
            self.label_color,
        );

        if self.points.len() >= 2 {
            let (dmin, _) = y_scale.domain();
            let base = y_scale.scale(dmin) as f32;

            // Filled area down to the domain floor
            let c = self.fill_color;
            self.draw_vector.set_color(c.x, c.y, c.z, c.w);
            let first_x = x_scale.scale(self.points[0].0) as f32;
            self.draw_vector.move_to(first_x, base);
            for p in &self.points {
                self.draw_vector
                    .line_to(x_scale.scale(p.0) as f32, y_scale.scale(p.1) as f32);
            }
            let last_x = x_scale.scale(self.points[self.points.len() - 1].0) as f32;
            self.draw_vector.line_to(last_x, base);
            self.draw_vector.close();
            self.draw_vector.fill();

            // Line on top
            let c = self.line_color;
            self.draw_vector.set_color(c.x, c.y, c.z, c.w);
            self.draw_vector.move_to(
                x_scale.scale(self.points[0].0) as f32,
                y_scale.scale(self.points[0].1) as f32,
            );
            for p in &self.points[1..] {
                self.draw_vector
                    .line_to(x_scale.scale(p.0) as f32, y_scale.scale(p.1) as f32);
            }
            self.draw_vector.stroke(self.line_width);
        }

        end_plot(cx, &mut self.draw_vector);
        DrawStep::done()
    }
}

// ================= Scatter chart =================

/// An x/y scatter plot, exposed to Splash as `d3.ScatterChart`.
#[derive(Script, Widget)]
pub struct D3ScatterChart {
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

    /// Declarative data: numbers, `[x y]` pairs, or `{x y}` objects.
    #[live]
    data: ScriptValue,

    /// Fill color of the dots.
    #[live]
    pub dot_color: Vec4f,
    /// Fill color of the hovered dot.
    #[live]
    pub hover_color: Vec4f,
    /// Dot radius in pixels.
    #[live(4.0)]
    pub dot_radius: f32,
    /// Grid line color.
    #[live]
    pub grid_color: Vec4f,
    /// Axis label color.
    #[live]
    pub label_color: Vec4f,
    /// Margins around the plot area.
    #[live]
    pub plot_margin: Inset,

    /// Fired with the dot index when a dot is clicked.
    #[live]
    on_click: ScriptFnRef,
    /// Fired with the dot index when the pointer moves onto a dot.
    #[live]
    on_hover: ScriptFnRef,

    #[rust]
    points: Vec<(f64, f64)>,
    #[rust]
    y_domain: Option<(f64, f64)>,
    #[rust]
    rect: Rect,
    #[rust]
    plot: Rect,
    #[rust]
    hovered: Option<usize>,
}

impl D3ScatterChart {
    /// Replace the chart data from Rust.
    pub fn set_points(&mut self, points: Vec<(f64, f64)>) {
        self.points = points;
        self.hovered = None;
    }

    fn ensure_demo_data(&mut self) {
        if self.points.is_empty() {
            self.points = (0..30)
                .map(|i| {
                    let x = i as f64 * 3.3;
                    (x, x * 0.7 + 25.0 * ((i as f64 * 2.7).sin()))
                })
                .collect();
        }
    }

    fn scales(&self) -> (LinearScale, LinearScale) {
        let (x_min, x_max) = min_max(self.points.iter().map(|p| p.0));
        let (y_min, y_max) = min_max(self.points.iter().map(|p| p.1));
        let x = LinearScale::new()
            .with_domain(x_min, if x_max > x_min { x_max } else { x_min + 1.0 })
            .with_nice(true)
            .with_range(self.plot.pos.x, self.plot.pos.x + self.plot.size.x);
        let y = y_scale_for(y_min, y_max, self.y_domain, false, &self.plot);
        (x, y)
    }

    fn dot_at(&self, abs: DVec2) -> Option<usize> {
        if self.points.is_empty() || !self.plot.contains(abs) {
            return None;
        }
        let (x_scale, y_scale) = self.scales();
        let threshold = (self.dot_radius as f64 + 6.0).powi(2);
        let mut best = None;
        let mut best_d = f64::INFINITY;
        for (i, p) in self.points.iter().enumerate() {
            let dx = x_scale.scale(p.0) - abs.x;
            let dy = y_scale.scale(p.1) - abs.y;
            let d = dx * dx + dy * dy;
            if d < best_d && d <= threshold {
                best_d = d;
                best = Some(i);
            }
        }
        best
    }
}

impl ScriptHook for D3ScatterChart {
    fn on_after_apply(
        &mut self,
        vm: &mut ScriptVm,
        _apply: &Apply,
        _scope: &mut Scope,
        _value: ScriptValue,
    ) {
        if let Some(points) = vm_data::to_xy_vec(vm, self.data) {
            self.points = points;
        }
    }
}

impl Widget for D3ScatterChart {
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
        if method == live_id!(set_domain) {
            let min = vm_data::arg(vm, args, 0).as_number();
            let max = vm_data::arg(vm, args, 1).as_number();
            self.y_domain = match (min, max) {
                (Some(min), Some(max)) if min < max => Some((min, max)),
                _ => None,
            };
            vm.with_cx_mut(|cx| self.redraw(cx));
            return ScriptAsyncResult::Return(NIL);
        }
        if method == live_id!(data) {
            let ys: Vec<f64> = self.points.iter().map(|p| p.1).collect();
            let arr = vm_data::f64_slice_to_array(vm, &ys);
            return ScriptAsyncResult::Return(arr);
        }
        ScriptAsyncResult::MethodNotFound
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        match event.hits(cx, self.draw_bg.area()) {
            Hit::FingerHoverIn(fe) | Hit::FingerHoverOver(fe) => {
                let hit = self.dot_at(fe.abs);
                if hit != self.hovered {
                    self.hovered = hit;
                    if let Some(i) = hit {
                        fire_index_callback(cx, self.uid, &self.source, &self.on_hover, i);
                    }
                    self.redraw(cx);
                }
                cx.set_cursor(if hit.is_some() {
                    MouseCursor::Hand
                } else {
                    MouseCursor::Default
                });
            }
            Hit::FingerHoverOut(_) if self.hovered.is_some() => {
                self.hovered = None;
                self.redraw(cx);
            }
            Hit::FingerUp(fe) if fe.is_over && fe.is_primary_hit() => {
                if let Some(i) = self.dot_at(fe.abs) {
                    fire_index_callback(cx, self.uid, &self.source, &self.on_click, i);
                }
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

        let (x_scale, y_scale) = self.scales();
        let plot = self.plot;
        let rect = self.rect;
        draw_y_grid(
            &mut self.draw_grid,
            &mut self.draw_text,
            cx,
            &y_scale,
            &rect,
            &plot,
            self.grid_color,
            self.label_color,
        );
        draw_x_grid_linear(
            &mut self.draw_grid,
            &mut self.draw_text,
            cx,
            &x_scale,
            &plot,
            self.grid_color,
            self.label_color,
        );

        for (i, p) in self.points.iter().enumerate() {
            let color = if self.hovered == Some(i) {
                self.hover_color
            } else {
                self.dot_color
            };
            self.draw_vector
                .set_color(color.x, color.y, color.z, color.w);
            self.draw_vector.circle(
                x_scale.scale(p.0) as f32,
                y_scale.scale(p.1) as f32,
                self.dot_radius,
            );
            self.draw_vector.fill();
        }

        end_plot(cx, &mut self.draw_vector);
        DrawStep::done()
    }
}

// ================= Pie chart =================

/// A pie/donut chart, exposed to Splash as `d3.PieChart`.
///
/// Slice colors follow the d3 Category10 scheme. Set `inner_radius`
/// (0.0–0.95, fraction of the outer radius) for a donut.
#[derive(Script, Widget)]
pub struct D3PieChart {
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

    /// Declarative data: array of numbers (slice values).
    #[live]
    data: ScriptValue,
    /// Declarative slice labels: array of strings.
    #[live]
    labels: ScriptValue,

    /// Donut hole radius as a fraction of the outer radius (0 = pie).
    #[live(0.0)]
    pub inner_radius: f32,
    /// Slice label color.
    #[live]
    pub label_color: Vec4f,
    /// Margins around the plot area.
    #[live]
    pub plot_margin: Inset,

    /// Fired with the slice index when a slice is clicked.
    #[live]
    on_click: ScriptFnRef,
    /// Fired with the slice index when the pointer moves onto a slice.
    #[live]
    on_hover: ScriptFnRef,

    #[rust]
    values: Vec<f64>,
    #[rust]
    label_strs: Vec<String>,
    #[rust]
    rect: Rect,
    #[rust]
    plot: Rect,
    #[rust]
    hovered: Option<usize>,
}

impl D3PieChart {
    /// Replace the chart data from Rust.
    pub fn set_values(&mut self, values: Vec<f64>) {
        self.values = values;
        self.hovered = None;
    }

    fn ensure_demo_data(&mut self) {
        if self.values.is_empty() {
            self.values = vec![35.0, 25.0, 18.0, 12.0, 10.0];
            if self.label_strs.is_empty() {
                self.label_strs = ["A", "B", "C", "D", "E"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect();
            }
        }
    }

    fn slices(&self) -> Vec<PieSlice<f64>> {
        PieLayout::new().compute(&self.values)
    }

    fn geometry(&self) -> (DVec2, f64) {
        let center = DVec2 {
            x: self.plot.pos.x + self.plot.size.x * 0.5,
            y: self.plot.pos.y + self.plot.size.y * 0.5,
        };
        let radius = (self.plot.size.x.min(self.plot.size.y) * 0.5 - 4.0).max(4.0);
        (center, radius)
    }

    fn slice_at(&self, abs: DVec2) -> Option<usize> {
        if self.values.is_empty() {
            return None;
        }
        let (center, radius) = self.geometry();
        let dx = abs.x - center.x;
        let dy = abs.y - center.y;
        let dist = (dx * dx + dy * dy).sqrt();
        let inner = radius * self.inner_radius.clamp(0.0, 0.95) as f64;
        if dist > radius || dist < inner {
            return None;
        }
        // d3 angle convention: 0 at 12 o'clock, increasing clockwise
        let mut angle = dx.atan2(-dy);
        if angle < 0.0 {
            angle += std::f64::consts::TAU;
        }
        self.slices()
            .iter()
            .position(|s| angle >= s.start_angle && angle <= s.end_angle)
    }
}

impl ScriptHook for D3PieChart {
    fn on_after_apply(
        &mut self,
        vm: &mut ScriptVm,
        _apply: &Apply,
        _scope: &mut Scope,
        _value: ScriptValue,
    ) {
        if let Some(values) = vm_data::to_f64_vec(vm, self.data) {
            self.values = values;
        }
        if let Some(labels) = vm_data::to_string_vec(vm, self.labels) {
            self.label_strs = labels;
        }
    }
}

impl Widget for D3PieChart {
    fn script_call(
        &mut self,
        vm: &mut ScriptVm,
        method: LiveId,
        args: ScriptValue,
    ) -> ScriptAsyncResult {
        if method == live_id!(set_data) {
            let value = vm_data::arg(vm, args, 0);
            if let Some(values) = vm_data::to_f64_vec(vm, value) {
                self.set_values(values);
                vm.with_cx_mut(|cx| self.redraw(cx));
            }
            return ScriptAsyncResult::Return(NIL);
        }
        if method == live_id!(set_labels) {
            let value = vm_data::arg(vm, args, 0);
            if let Some(labels) = vm_data::to_string_vec(vm, value) {
                self.label_strs = labels;
                vm.with_cx_mut(|cx| self.redraw(cx));
            }
            return ScriptAsyncResult::Return(NIL);
        }
        if method == live_id!(data) {
            let arr = vm_data::f64_slice_to_array(vm, &self.values);
            return ScriptAsyncResult::Return(arr);
        }
        ScriptAsyncResult::MethodNotFound
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        match event.hits(cx, self.draw_bg.area()) {
            Hit::FingerHoverIn(fe) | Hit::FingerHoverOver(fe) => {
                let hit = self.slice_at(fe.abs);
                if hit != self.hovered {
                    self.hovered = hit;
                    if let Some(i) = hit {
                        fire_index_callback(cx, self.uid, &self.source, &self.on_hover, i);
                    }
                    self.redraw(cx);
                }
                cx.set_cursor(if hit.is_some() {
                    MouseCursor::Hand
                } else {
                    MouseCursor::Default
                });
            }
            Hit::FingerHoverOut(_) if self.hovered.is_some() => {
                self.hovered = None;
                self.redraw(cx);
            }
            Hit::FingerUp(fe) if fe.is_over && fe.is_primary_hit() => {
                if let Some(i) = self.slice_at(fe.abs) {
                    fire_index_callback(cx, self.uid, &self.source, &self.on_click, i);
                }
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
        let inner = radius * self.inner_radius.clamp(0.0, 0.95) as f64;
        let palette = CategoricalScale::category10();
        let hovered = self.hovered;

        // d3 angle convention: 0 at 12 o'clock, clockwise.
        let point_at = |angle: f64, r: f64| -> (f32, f32) {
            (
                (center.x + r * angle.sin()) as f32,
                (center.y - r * angle.cos()) as f32,
            )
        };

        for slice in self.slices() {
            let i = slice.index;
            let mut color = rgba_to_vec4f(palette.get(i));
            if hovered == Some(i) {
                color.x = (color.x * 1.25).min(1.0);
                color.y = (color.y * 1.25).min(1.0);
                color.z = (color.z * 1.25).min(1.0);
            }
            self.draw_vector
                .set_color(color.x, color.y, color.z, color.w);

            let sweep = (slice.end_angle - slice.start_angle).max(0.0);
            let segs = ((sweep / 0.05).ceil() as usize).clamp(2, 256);
            let step = sweep / segs as f64;

            // Outer arc
            let (sx, sy) = point_at(slice.start_angle, radius);
            self.draw_vector.move_to(sx, sy);
            for k in 1..=segs {
                let (px, py) = point_at(slice.start_angle + step * k as f64, radius);
                self.draw_vector.line_to(px, py);
            }
            if inner > 0.5 {
                // Inner arc back (donut)
                for k in (0..=segs).rev() {
                    let (px, py) = point_at(slice.start_angle + step * k as f64, inner);
                    self.draw_vector.line_to(px, py);
                }
            } else {
                let (cx_px, cy_px) = point_at(0.0, 0.0);
                self.draw_vector.line_to(cx_px, cy_px);
            }
            self.draw_vector.close();
            self.draw_vector.fill();
        }

        // Labels at mid-angle, just outside the arc midpoint
        self.draw_text.color = self.label_color;
        for slice in self.slices() {
            let i = slice.index;
            let Some(label) = self.label_strs.get(i) else {
                continue;
            };
            if label.is_empty() {
                continue;
            }
            let mid = (slice.start_angle + slice.end_angle) * 0.5;
            let r = (radius + inner) * 0.5;
            let px = center.x + r * mid.sin();
            let py = center.y - r * mid.cos();
            self.draw_text
                .draw_abs(cx, dvec2(px - 3.0 * label.len() as f64, py - 6.0), label);
        }

        end_plot(cx, &mut self.draw_vector);
        DrawStep::done()
    }
}

//! Statistical chart widgets: `d3.Histogram`, `d3.Heatmap`,
//! `d3.RadarChart`, `d3.BoxPlot`.
//!
//! Same contract as the base charts: declarative `data:`, `set_data(...)`
//! script calls, `on_click`/`on_hover` closures, `DrawVector` rendering.

// The `script_mod!` and `derive(Script, Widget)` macros generate public
// registration items that cannot carry doc comments.
#![allow(missing_docs)]

use crate::render3d::Colormap;
use crate::scale::{CategoryScale, DiscreteScale, LinearScale, Scale, ScaleExt};

use super::charts::{
    begin_plot, compute_plot_rect, draw_x_grid_linear, draw_x_labels_category, draw_y_grid,
    end_plot, fire_index_callback, min_max, y_scale_for,
};
use super::vm_data;
use makepad_widgets::makepad_script::ScriptFnRef;
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*

    mod.d3.HistogramBase = #(D3Histogram::register_widget(vm))
    mod.d3.Histogram = set_type_default() do mod.d3.HistogramBase{
        width: Fill
        height: 300
        plot_margin: Inset{left: 46.0, top: 12.0, right: 12.0, bottom: 30.0}
        bins: 20
        bar_color: #x1f77b4
        hover_color: #x4fa3d4
        grid_color: #x3a3f52
        label_color: #x9aa0b0
        draw_bg +: { draw_depth: 0.0 color: #x00000000 }
        draw_grid +: { draw_depth: 0.1 }
        draw_vector +: { draw_depth: 2.0 }
        draw_text +: { draw_depth: 3.0 }
    }

    mod.d3.HeatmapBase = #(D3Heatmap::register_widget(vm))
    mod.d3.Heatmap = set_type_default() do mod.d3.HeatmapBase{
        width: Fill
        height: 300
        plot_margin: Inset{left: 12.0, top: 12.0, right: 12.0, bottom: 12.0}
        colormap: "viridis"
        cell_gap: 1.0
        draw_bg +: { draw_depth: 0.0 color: #x00000000 }
        draw_grid +: { draw_depth: 0.1 }
        draw_vector +: { draw_depth: 2.0 }
        draw_text +: { draw_depth: 3.0 }
    }

    mod.d3.RadarChartBase = #(D3RadarChart::register_widget(vm))
    mod.d3.RadarChart = set_type_default() do mod.d3.RadarChartBase{
        width: Fill
        height: 300
        plot_margin: Inset{left: 24.0, top: 24.0, right: 24.0, bottom: 24.0}
        line_color: #x1f77b4
        fill_color: #x1f77b455
        grid_color: #x3a3f52
        label_color: #x9aa0b0
        rings: 4
        draw_bg +: { draw_depth: 0.0 color: #x00000000 }
        draw_grid +: { draw_depth: 0.1 }
        draw_vector +: { draw_depth: 2.0 }
        draw_text +: { draw_depth: 3.0 }
    }

    mod.d3.BoxPlotBase = #(D3BoxPlot::register_widget(vm))
    mod.d3.BoxPlot = set_type_default() do mod.d3.BoxPlotBase{
        width: Fill
        height: 300
        plot_margin: Inset{left: 46.0, top: 12.0, right: 12.0, bottom: 30.0}
        box_color: #x1f77b4
        median_color: #xffffff
        whisker_color: #x9aa0b0
        outlier_color: #xd62728
        grid_color: #x3a3f52
        label_color: #x9aa0b0
        draw_bg +: { draw_depth: 0.0 color: #x00000000 }
        draw_grid +: { draw_depth: 0.1 }
        draw_vector +: { draw_depth: 2.0 }
        draw_text +: { draw_depth: 3.0 }
    }
}

pub(crate) fn colormap_from_name(name: &str) -> Colormap {
    match name {
        "plasma" => Colormap::Plasma,
        "inferno" => Colormap::Inferno,
        "magma" => Colormap::Magma,
        "coolwarm" | "cool_warm" => Colormap::CoolWarm,
        "turbo" => Colormap::Turbo,
        "gray" | "grayscale" => Colormap::Grayscale,
        _ => Colormap::Viridis,
    }
}

pub(crate) fn colormap_vec4f(cm: Colormap, t: f64) -> Vec4f {
    let c = cm.sample_rgba(t as f32);
    Vec4f {
        x: c[0],
        y: c[1],
        z: c[2],
        w: c[3],
    }
}

/// Parse `data:` as rows of numbers (2D grid).
pub(crate) fn to_grid(vm: &mut ScriptVm, value: ScriptValue) -> Option<Vec<Vec<f64>>> {
    let rows = vm_data::to_rows(vm, value)?;
    Some(rows)
}

// ================= Histogram =================

/// Value-distribution histogram, exposed to Splash as `d3.Histogram`.
#[derive(Script, Widget)]
pub struct D3Histogram {
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

    /// Declarative data: raw sample values.
    #[live]
    data: ScriptValue,

    /// Number of bins.
    #[live(20.0)]
    pub bins: f64,
    /// Bar fill color.
    #[live]
    pub bar_color: Vec4f,
    /// Hovered bar fill color.
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

    /// Fired with the bin index on click.
    #[live]
    on_click: ScriptFnRef,
    /// Fired with the bin index on hover.
    #[live]
    on_hover: ScriptFnRef,

    #[rust]
    values: Vec<f64>,
    #[rust]
    rect: Rect,
    #[rust]
    plot: Rect,
    #[rust]
    hovered: Option<usize>,
}

struct HistogramBins {
    counts: Vec<usize>,
    min: f64,
    max: f64,
}

impl D3Histogram {
    /// Replace the sample values from Rust.
    pub fn set_values(&mut self, values: Vec<f64>) {
        self.values = values;
        self.hovered = None;
    }

    fn ensure_demo_data(&mut self) {
        if self.values.is_empty() {
            // deterministic pseudo-normal samples
            let mut seed = 42u64;
            let mut r = || {
                seed ^= seed << 13;
                seed ^= seed >> 7;
                seed ^= seed << 17;
                seed as f64 / u64::MAX as f64
            };
            self.values = (0..400)
                .map(|_| (r() + r() + r() + r() - 2.0) * 25.0 + 50.0)
                .collect();
        }
    }

    fn compute_bins(&self) -> HistogramBins {
        let n = (self.bins.max(1.0) as usize).min(200);
        let (min, max) = min_max(self.values.iter().copied());
        let span = (max - min).max(f64::EPSILON);
        let mut counts = vec![0usize; n];
        for &v in &self.values {
            let i = (((v - min) / span) * n as f64) as usize;
            counts[i.min(n - 1)] += 1;
        }
        HistogramBins { counts, min, max }
    }

    fn bin_at(&self, abs: DVec2) -> Option<usize> {
        if self.values.is_empty() || !self.plot.contains(abs) {
            return None;
        }
        let n = (self.bins.max(1.0) as usize).min(200);
        let frac = (abs.x - self.plot.pos.x) / self.plot.size.x;
        if !(0.0..=1.0).contains(&frac) {
            return None;
        }
        Some(((frac * n as f64) as usize).min(n - 1))
    }
}

impl ScriptHook for D3Histogram {
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
    }
}

impl Widget for D3Histogram {
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
        if method == live_id!(set_bins) {
            if let Some(n) = vm_data::arg(vm, args, 0).as_number() {
                self.bins = n.clamp(1.0, 200.0);
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
                let hit = self.bin_at(fe.abs);
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
                if let Some(i) = self.bin_at(fe.abs) {
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

        let bins = self.compute_bins();
        let max_count = bins.counts.iter().copied().max().unwrap_or(1).max(1) as f64;
        let x_scale = LinearScale::new()
            .with_domain(bins.min, bins.max)
            .with_range(self.plot.pos.x, self.plot.pos.x + self.plot.size.x);
        let y_scale = y_scale_for(0.0, max_count, None, true, &self.plot);

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

        let n = bins.counts.len();
        let baseline = y_scale.scale(0.0);
        let bw = plot.size.x / n as f64;
        for (i, &count) in bins.counts.iter().enumerate() {
            let x0 = plot.pos.x + i as f64 * bw;
            let py = y_scale.scale(count as f64);
            let color = if self.hovered == Some(i) {
                self.hover_color
            } else {
                self.bar_color
            };
            self.draw_vector
                .set_color(color.x, color.y, color.z, color.w);
            self.draw_vector.rect(
                x0 as f32 + 0.5,
                py as f32,
                (bw - 1.0).max(1.0) as f32,
                (baseline - py).max(0.0) as f32,
            );
            self.draw_vector.fill();
        }

        end_plot(cx, &mut self.draw_vector);
        DrawStep::done()
    }
}

// ================= Heatmap =================

/// Grid heatmap with scientific colormaps, exposed as `d3.Heatmap`.
#[derive(Script, Widget)]
pub struct D3Heatmap {
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

    /// Declarative data: array of rows, each an array of numbers.
    #[live]
    data: ScriptValue,

    /// Colormap name: viridis, plasma, inferno, magma, coolwarm, turbo, gray.
    #[live]
    pub colormap: String,
    /// Gap between cells in pixels.
    #[live(1.0)]
    pub cell_gap: f32,
    /// Margins around the plot area.
    #[live]
    pub plot_margin: Inset,

    /// Fired with (row, col) on click.
    #[live]
    on_click: ScriptFnRef,
    /// Fired with (row, col) on hover.
    #[live]
    on_hover: ScriptFnRef,

    #[rust]
    grid: Vec<Vec<f64>>,
    #[rust]
    rect: Rect,
    #[rust]
    plot: Rect,
    #[rust]
    hovered: Option<(usize, usize)>,
}

impl D3Heatmap {
    /// Replace the grid data from Rust.
    pub fn set_grid(&mut self, grid: Vec<Vec<f64>>) {
        self.grid = grid;
        self.hovered = None;
    }

    fn ensure_demo_data(&mut self) {
        if self.grid.is_empty() {
            self.grid = (0..12)
                .map(|r| {
                    (0..24)
                        .map(|c| {
                            let x = c as f64 * 0.3;
                            let y = r as f64 * 0.45;
                            (x.sin() * y.cos() + 1.0) * 0.5
                        })
                        .collect()
                })
                .collect();
        }
    }

    fn dims(&self) -> (usize, usize) {
        let rows = self.grid.len();
        let cols = self.grid.iter().map(|r| r.len()).max().unwrap_or(0);
        (rows, cols)
    }

    fn cell_at(&self, abs: DVec2) -> Option<(usize, usize)> {
        let (rows, cols) = self.dims();
        if rows == 0 || cols == 0 || !self.plot.contains(abs) {
            return None;
        }
        let col = (((abs.x - self.plot.pos.x) / self.plot.size.x) * cols as f64) as usize;
        let row = (((abs.y - self.plot.pos.y) / self.plot.size.y) * rows as f64) as usize;
        Some((row.min(rows - 1), col.min(cols - 1)))
    }

    fn fire_cell(&self, cx: &mut Cx, fn_ref: &ScriptFnRef, row: usize, col: usize) {
        cx.widget_to_script_call(
            self.uid,
            NIL,
            self.source.clone(),
            fn_ref.clone(),
            &[(row as f64).into(), (col as f64).into()],
        );
    }
}

impl ScriptHook for D3Heatmap {
    fn on_after_apply(
        &mut self,
        vm: &mut ScriptVm,
        _apply: &Apply,
        _scope: &mut Scope,
        _value: ScriptValue,
    ) {
        if let Some(grid) = to_grid(vm, self.data) {
            if !grid.is_empty() {
                self.grid = grid;
            }
        }
    }
}

impl Widget for D3Heatmap {
    fn script_call(
        &mut self,
        vm: &mut ScriptVm,
        method: LiveId,
        args: ScriptValue,
    ) -> ScriptAsyncResult {
        if method == live_id!(set_data) {
            let value = vm_data::arg(vm, args, 0);
            if let Some(grid) = to_grid(vm, value) {
                self.set_grid(grid);
                vm.with_cx_mut(|cx| self.redraw(cx));
            }
            return ScriptAsyncResult::Return(NIL);
        }
        ScriptAsyncResult::MethodNotFound
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        match event.hits(cx, self.draw_bg.area()) {
            Hit::FingerHoverIn(fe) | Hit::FingerHoverOver(fe) => {
                let hit = self.cell_at(fe.abs);
                if hit != self.hovered {
                    self.hovered = hit;
                    if let Some((r, c)) = hit {
                        let on_hover = self.on_hover.clone();
                        self.fire_cell(cx, &on_hover, r, c);
                    }
                    self.redraw(cx);
                }
            }
            Hit::FingerHoverOut(_) if self.hovered.is_some() => {
                self.hovered = None;
                self.redraw(cx);
            }
            Hit::FingerUp(fe) if fe.is_over && fe.is_primary_hit() => {
                if let Some((r, c)) = self.cell_at(fe.abs) {
                    let on_click = self.on_click.clone();
                    self.fire_cell(cx, &on_click, r, c);
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

        let (rows, cols) = self.dims();
        if rows > 0 && cols > 0 {
            let (min, max) = min_max(self.grid.iter().flatten().copied());
            let span = (max - min).max(f64::EPSILON);
            let cm = colormap_from_name(&self.colormap);
            let cw = self.plot.size.x / cols as f64;
            let ch = self.plot.size.y / rows as f64;
            let gap = self.cell_gap.min((cw.min(ch) * 0.4) as f32) as f64;

            for (r, row) in self.grid.iter().enumerate() {
                for (c, &v) in row.iter().enumerate() {
                    let t = (v - min) / span;
                    let mut color = colormap_vec4f(cm, t);
                    if self.hovered == Some((r, c)) {
                        color.x = (color.x * 1.3 + 0.1).min(1.0);
                        color.y = (color.y * 1.3 + 0.1).min(1.0);
                        color.z = (color.z * 1.3 + 0.1).min(1.0);
                    }
                    self.draw_vector
                        .set_color(color.x, color.y, color.z, color.w);
                    self.draw_vector.rect(
                        (self.plot.pos.x + c as f64 * cw) as f32,
                        (self.plot.pos.y + r as f64 * ch) as f32,
                        (cw - gap).max(1.0) as f32,
                        (ch - gap).max(1.0) as f32,
                    );
                    self.draw_vector.fill();
                }
            }
        }

        end_plot(cx, &mut self.draw_vector);
        DrawStep::done()
    }
}

// ================= Radar chart =================

/// Polar radar/spider chart, exposed as `d3.RadarChart`.
#[derive(Script, Widget)]
pub struct D3RadarChart {
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

    /// Declarative data: one value per axis.
    #[live]
    data: ScriptValue,
    /// Declarative axis labels.
    #[live]
    labels: ScriptValue,

    /// Polygon stroke color.
    #[live]
    pub line_color: Vec4f,
    /// Polygon fill color.
    #[live]
    pub fill_color: Vec4f,
    /// Grid ring/spoke color.
    #[live]
    pub grid_color: Vec4f,
    /// Axis label color.
    #[live]
    pub label_color: Vec4f,
    /// Number of grid rings.
    #[live(4.0)]
    pub rings: f64,
    /// Margins around the plot area.
    #[live]
    pub plot_margin: Inset,

    /// Fired with the axis index on click.
    #[live]
    on_click: ScriptFnRef,

    #[rust]
    values: Vec<f64>,
    #[rust]
    label_strs: Vec<String>,
    #[rust]
    rect: Rect,
    #[rust]
    plot: Rect,
}

impl D3RadarChart {
    /// Replace the per-axis values from Rust.
    pub fn set_values(&mut self, values: Vec<f64>) {
        self.values = values;
    }

    fn ensure_demo_data(&mut self) {
        if self.values.is_empty() {
            self.values = vec![80.0, 55.0, 70.0, 90.0, 45.0, 65.0];
            if self.label_strs.is_empty() {
                self.label_strs = ["Speed", "Power", "Range", "Skill", "Luck", "Style"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect();
            }
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

    fn axis_point(&self, i: usize, t: f64) -> (f32, f32) {
        let n = self.values.len().max(1);
        let (center, radius) = self.geometry();
        let angle = std::f64::consts::TAU * i as f64 / n as f64;
        (
            (center.x + radius * t * angle.sin()) as f32,
            (center.y - radius * t * angle.cos()) as f32,
        )
    }
}

impl ScriptHook for D3RadarChart {
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

impl Widget for D3RadarChart {
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
        if let Hit::FingerUp(fe) = event.hits(cx, self.draw_bg.area()) {
            if fe.is_over && fe.is_primary_hit() && !self.values.is_empty() {
                let (center, _) = self.geometry();
                let mut angle = (fe.abs.x - center.x).atan2(-(fe.abs.y - center.y));
                if angle < 0.0 {
                    angle += std::f64::consts::TAU;
                }
                let n = self.values.len();
                let i = ((angle / std::f64::consts::TAU * n as f64) + 0.5) as usize % n;
                fire_index_callback(cx, self.uid, &self.source, &self.on_click, i);
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        self.ensure_demo_data();
        self.rect = cx.walk_turtle(walk);
        self.plot = compute_plot_rect(&self.rect, &self.plot_margin);
        self.draw_bg.draw_abs(cx, self.rect);

        begin_plot(cx, &self.rect, &self.plot_margin, &mut self.draw_vector);

        let n = self.values.len();
        if n >= 3 {
            let (_, max) = min_max(self.values.iter().copied());
            let max = if max > 0.0 { max } else { 1.0 };
            let g = self.grid_color;

            // Rings
            let rings = (self.rings.max(1.0) as usize).min(10);
            self.draw_vector.set_color(g.x, g.y, g.z, g.w);
            for ring in 1..=rings {
                let t = ring as f64 / rings as f64;
                let (px, py) = self.axis_point(0, t);
                self.draw_vector.move_to(px, py);
                for i in 1..=n {
                    let (px, py) = self.axis_point(i % n, t);
                    self.draw_vector.line_to(px, py);
                }
                self.draw_vector.stroke(1.0);
            }
            // Spokes
            for i in 0..n {
                let (cx_px, cy_px) = self.axis_point(i, 0.0);
                let (px, py) = self.axis_point(i, 1.0);
                self.draw_vector.set_color(g.x, g.y, g.z, g.w);
                self.draw_vector.move_to(cx_px, cy_px);
                self.draw_vector.line_to(px, py);
                self.draw_vector.stroke(1.0);
            }

            // Data polygon: fill then stroke
            let f = self.fill_color;
            self.draw_vector.set_color(f.x, f.y, f.z, f.w);
            let (px0, py0) = self.axis_point(0, (self.values[0] / max).clamp(0.0, 1.0));
            self.draw_vector.move_to(px0, py0);
            for (i, &v) in self.values.iter().enumerate().skip(1) {
                let (px, py) = self.axis_point(i, (v / max).clamp(0.0, 1.0));
                self.draw_vector.line_to(px, py);
            }
            self.draw_vector.close();
            self.draw_vector.fill();

            let l = self.line_color;
            self.draw_vector.set_color(l.x, l.y, l.z, l.w);
            self.draw_vector.move_to(px0, py0);
            for (i, &v) in self.values.iter().enumerate().skip(1) {
                let (px, py) = self.axis_point(i, (v / max).clamp(0.0, 1.0));
                self.draw_vector.line_to(px, py);
            }
            self.draw_vector.close();
            self.draw_vector.stroke(2.0);

            // Axis labels just outside the outer ring
            self.draw_text.color = self.label_color;
            for (i, label) in self.label_strs.iter().enumerate().take(n) {
                let (px, py) = self.axis_point(i, 1.08);
                self.draw_text.draw_abs(
                    cx,
                    dvec2(px as f64 - 3.0 * label.len() as f64, py as f64 - 6.0),
                    label,
                );
            }
        }

        end_plot(cx, &mut self.draw_vector);
        DrawStep::done()
    }
}

// ================= Box plot =================

/// Distribution box plot (Tukey whiskers + outliers), exposed as `d3.BoxPlot`.
#[derive(Script, Widget)]
pub struct D3BoxPlot {
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

    /// Declarative data: array of distributions (arrays of numbers).
    #[live]
    data: ScriptValue,
    /// Declarative per-distribution labels.
    #[live]
    labels: ScriptValue,

    /// Box fill color.
    #[live]
    pub box_color: Vec4f,
    /// Median line color.
    #[live]
    pub median_color: Vec4f,
    /// Whisker color.
    #[live]
    pub whisker_color: Vec4f,
    /// Outlier dot color.
    #[live]
    pub outlier_color: Vec4f,
    /// Grid line color.
    #[live]
    pub grid_color: Vec4f,
    /// Axis label color.
    #[live]
    pub label_color: Vec4f,
    /// Margins around the plot area.
    #[live]
    pub plot_margin: Inset,

    /// Fired with the series index on click.
    #[live]
    on_click: ScriptFnRef,

    #[rust]
    series: Vec<Vec<f64>>,
    #[rust]
    label_strs: Vec<String>,
    #[rust]
    rect: Rect,
    #[rust]
    plot: Rect,
}

struct BoxStats {
    q1: f64,
    median: f64,
    q3: f64,
    lo: f64,
    hi: f64,
    outliers: Vec<f64>,
}

fn quantile(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let pos = q * (sorted.len() - 1) as f64;
    let base = pos.floor() as usize;
    let rest = pos - base as f64;
    if base + 1 < sorted.len() {
        sorted[base] + rest * (sorted[base + 1] - sorted[base])
    } else {
        sorted[base]
    }
}

fn box_stats(values: &[f64]) -> BoxStats {
    let mut sorted: Vec<f64> = values.iter().copied().filter(|v| v.is_finite()).collect();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let q1 = quantile(&sorted, 0.25);
    let median = quantile(&sorted, 0.5);
    let q3 = quantile(&sorted, 0.75);
    let iqr = q3 - q1;
    let lo_fence = q1 - 1.5 * iqr;
    let hi_fence = q3 + 1.5 * iqr;
    let lo = sorted
        .iter()
        .copied()
        .find(|&v| v >= lo_fence)
        .unwrap_or(q1);
    let hi = sorted
        .iter()
        .rev()
        .copied()
        .find(|&v| v <= hi_fence)
        .unwrap_or(q3);
    let outliers = sorted
        .iter()
        .copied()
        .filter(|&v| v < lo_fence || v > hi_fence)
        .collect();
    BoxStats {
        q1,
        median,
        q3,
        lo,
        hi,
        outliers,
    }
}

impl D3BoxPlot {
    /// Replace the distributions from Rust.
    pub fn set_series(&mut self, series: Vec<Vec<f64>>) {
        self.series = series;
    }

    fn ensure_demo_data(&mut self) {
        if self.series.is_empty() {
            let mut seed = 7u64;
            let mut r = || {
                seed ^= seed << 13;
                seed ^= seed >> 7;
                seed ^= seed << 17;
                seed as f64 / u64::MAX as f64
            };
            self.series = (0..4)
                .map(|s| {
                    let center = 40.0 + s as f64 * 15.0;
                    (0..60)
                        .map(|_| {
                            center + (r() + r() - 1.0) * 20.0 + if r() > 0.96 { 45.0 } else { 0.0 }
                        })
                        .collect()
                })
                .collect();
            if self.label_strs.is_empty() {
                self.label_strs = ["A", "B", "C", "D"].iter().map(|s| s.to_string()).collect();
            }
        }
    }

    fn x_scale(&self) -> CategoryScale {
        let n = self.series.len();
        let labels: Vec<String> = (0..n)
            .map(|i| {
                self.label_strs
                    .get(i)
                    .cloned()
                    .unwrap_or_else(|| format!("{i}"))
            })
            .collect();
        CategoryScale::new()
            .with_labels(labels)
            .with_range(self.plot.pos.x, self.plot.pos.x + self.plot.size.x)
            .with_padding(0.3)
    }
}

impl ScriptHook for D3BoxPlot {
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
        if let Some(labels) = vm_data::to_string_vec(vm, self.labels) {
            self.label_strs = labels;
        }
    }
}

impl Widget for D3BoxPlot {
    fn script_call(
        &mut self,
        vm: &mut ScriptVm,
        method: LiveId,
        args: ScriptValue,
    ) -> ScriptAsyncResult {
        if method == live_id!(set_data) {
            let value = vm_data::arg(vm, args, 0);
            if let Some(rows) = vm_data::to_rows(vm, value) {
                self.set_series(rows);
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
        ScriptAsyncResult::MethodNotFound
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        if let Hit::FingerUp(fe) = event.hits(cx, self.draw_bg.area()) {
            if fe.is_over && fe.is_primary_hit() && !self.series.is_empty() {
                let x_scale = self.x_scale();
                let bw = x_scale.bandwidth();
                for i in 0..self.series.len() {
                    let x0 = x_scale.band_start(i);
                    if fe.abs.x >= x0 && fe.abs.x <= x0 + bw {
                        fire_index_callback(cx, self.uid, &self.source, &self.on_click, i);
                        break;
                    }
                }
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        self.ensure_demo_data();
        self.rect = cx.walk_turtle(walk);
        self.plot = compute_plot_rect(&self.rect, &self.plot_margin);
        self.draw_bg.draw_abs(cx, self.rect);

        begin_plot(cx, &self.rect, &self.plot_margin, &mut self.draw_vector);

        let stats: Vec<BoxStats> = self.series.iter().map(|s| box_stats(s)).collect();
        let (min, max) = min_max(stats.iter().flat_map(|s| {
            s.outliers
                .iter()
                .copied()
                .chain([s.lo, s.hi])
                .collect::<Vec<f64>>()
        }));
        let y_scale = y_scale_for(min, max, None, false, &self.plot);
        let x_scale = self.x_scale();

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

        let bw = x_scale.bandwidth();
        for (i, st) in stats.iter().enumerate() {
            let x0 = x_scale.band_start(i);
            let cx_px = x0 + bw * 0.5;
            let y_q1 = y_scale.scale(st.q1);
            let y_q3 = y_scale.scale(st.q3);
            let y_med = y_scale.scale(st.median);
            let y_lo = y_scale.scale(st.lo);
            let y_hi = y_scale.scale(st.hi);

            // Whisker line + caps
            let w = self.whisker_color;
            self.draw_vector.set_color(w.x, w.y, w.z, w.w);
            self.draw_vector.move_to(cx_px as f32, y_hi as f32);
            self.draw_vector.line_to(cx_px as f32, y_lo as f32);
            self.draw_vector.stroke(1.0);
            for y in [y_lo, y_hi] {
                self.draw_vector
                    .move_to((cx_px - bw * 0.2) as f32, y as f32);
                self.draw_vector
                    .line_to((cx_px + bw * 0.2) as f32, y as f32);
                self.draw_vector.stroke(1.0);
            }

            // Box
            let b = self.box_color;
            self.draw_vector.set_color(b.x, b.y, b.z, b.w);
            self.draw_vector.rect(
                x0 as f32,
                y_q3 as f32,
                bw as f32,
                (y_q1 - y_q3).max(1.0) as f32,
            );
            self.draw_vector.fill();

            // Median
            let m = self.median_color;
            self.draw_vector.set_color(m.x, m.y, m.z, m.w);
            self.draw_vector.move_to(x0 as f32, y_med as f32);
            self.draw_vector.line_to((x0 + bw) as f32, y_med as f32);
            self.draw_vector.stroke(2.0);

            // Outliers
            let o = self.outlier_color;
            self.draw_vector.set_color(o.x, o.y, o.z, o.w);
            for &v in &st.outliers {
                self.draw_vector
                    .circle(cx_px as f32, y_scale.scale(v) as f32, 2.5);
                self.draw_vector.fill();
            }
        }

        end_plot(cx, &mut self.draw_vector);
        DrawStep::done()
    }
}

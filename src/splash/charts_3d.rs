//! 3D chart widgets on the render3d core: `d3.Surface3D`, `d3.Scatter3D`,
//! `d3.Bar3D`.
//!
//! Geometry, projection, and depth sorting come from [`crate::render3d`];
//! faces arrive as projected screen quads and are painted back-to-front
//! through the GPU vector path renderer with CPU-side lighting. Drag to
//! orbit, scroll to zoom.

// The `script_mod!` and `derive(Script, Widget)` macros generate public
// registration items that cannot carry doc comments.
#![allow(missing_docs)]

use crate::render3d::{Bar3D, CameraEvent, Scatter3D, ScatterPoint3D, Surface3D};

use super::charts::{begin_plot, compute_plot_rect, end_plot};
use super::charts_stat::colormap_from_name;
use super::vm_data;
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*

    mod.d3.Surface3DBase = #(D3Surface3D::register_widget(vm))
    mod.d3.Surface3D = set_type_default() do mod.d3.Surface3DBase{
        width: Fill
        height: 320
        plot_margin: Inset{left: 8.0, top: 8.0, right: 8.0, bottom: 8.0}
        colormap: "viridis"
        draw_bg +: { draw_depth: 0.0 color: #x00000000 }
        draw_grid +: { draw_depth: 0.1 }
        draw_vector +: { draw_depth: 2.0 }
        draw_text +: { draw_depth: 3.0 }
    }

    mod.d3.Scatter3DBase = #(D3Scatter3D::register_widget(vm))
    mod.d3.Scatter3D = set_type_default() do mod.d3.Scatter3DBase{
        width: Fill
        height: 320
        plot_margin: Inset{left: 8.0, top: 8.0, right: 8.0, bottom: 8.0}
        colormap: "plasma"
        draw_bg +: { draw_depth: 0.0 color: #x00000000 }
        draw_grid +: { draw_depth: 0.1 }
        draw_vector +: { draw_depth: 2.0 }
        draw_text +: { draw_depth: 3.0 }
    }

    mod.d3.Bar3DBase = #(D3Bar3D::register_widget(vm))
    mod.d3.Bar3D = set_type_default() do mod.d3.Bar3DBase{
        width: Fill
        height: 320
        plot_margin: Inset{left: 8.0, top: 8.0, right: 8.0, bottom: 8.0}
        colormap: "viridis"
        draw_bg +: { draw_depth: 0.0 color: #x00000000 }
        draw_grid +: { draw_depth: 0.1 }
        draw_vector +: { draw_depth: 2.0 }
        draw_text +: { draw_depth: 3.0 }
    }
}

/// CPU Phong-ish shading factor for a world-space normal.
fn light_factor(normal: [f32; 3]) -> f32 {
    let len = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2])
        .sqrt()
        .max(1e-6);
    let n = [normal[0] / len, normal[1] / len, normal[2] / len];
    // Light from top-right-front (matches the shader port defaults)
    let l = [0.3f32, 0.8, 0.5];
    let ll = (l[0] * l[0] + l[1] * l[1] + l[2] * l[2]).sqrt();
    let ndl = ((n[0] * l[0] + n[1] * l[1] + n[2] * l[2]) / ll).max(0.0);
    0.35 + 0.65 * ndl
}

fn fill_quad(dv: &mut DrawVector, verts: &[[f64; 2]; 4]) {
    dv.move_to(verts[0][0] as f32, verts[0][1] as f32);
    for v in &verts[1..] {
        dv.line_to(v[0] as f32, v[1] as f32);
    }
    dv.close();
    dv.fill();
}

/// Route pointer/scroll hits to a render3d camera; returns true on change.
fn camera_hit<F: FnMut(CameraEvent) -> bool>(hit: Hit, mut apply: F) -> bool {
    match hit {
        Hit::FingerDown(fe) if fe.is_primary_hit() => apply(CameraEvent::PointerDown {
            pos: [fe.abs.x, fe.abs.y],
            shift: fe.modifiers.shift,
        }),
        Hit::FingerMove(fe) => apply(CameraEvent::PointerMove {
            pos: [fe.abs.x, fe.abs.y],
            shift: fe.modifiers.shift,
        }),
        Hit::FingerUp(_) => apply(CameraEvent::PointerUp),
        Hit::FingerScroll(fs) => apply(CameraEvent::Scroll {
            delta_y: fs.scroll.y,
        }),
        _ => false,
    }
}

// ================= Surface3D =================

/// 3D surface plot, exposed as `d3.Surface3D`.
///
/// `data:` accepts a height grid (array of rows); with no data a demo
/// ripple function is shown.
#[derive(Script, Widget)]
pub struct D3Surface3D {
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

    /// Declarative data: height grid (array of rows of numbers).
    #[live]
    data: ScriptValue,
    /// Colormap name (see `d3.Heatmap`).
    #[live]
    pub colormap: String,
    /// Margins around the plot area.
    #[live]
    pub plot_margin: Inset,

    #[rust]
    surface: Option<Surface3D>,
    #[rust]
    pending_grid: Option<Vec<Vec<f64>>>,
    #[rust]
    rect: Rect,
    #[rust]
    plot: Rect,
}

impl D3Surface3D {
    /// Replace the height grid from Rust.
    pub fn set_grid(&mut self, grid: Vec<Vec<f64>>) {
        self.pending_grid = Some(grid);
    }

    fn ensure_surface(&mut self) {
        if let Some(grid) = self.pending_grid.take() {
            let rows = grid.len();
            let cols = grid.iter().map(|r| r.len()).min().unwrap_or(0);
            if rows >= 2 && cols >= 2 {
                let mut surface = Surface3D::new();
                let g = grid.clone();
                let res = rows.min(cols).min(96);
                surface.set_function(res, (-2.0, 2.0), (-2.0, 2.0), move |x, z| {
                    // Bilinear sample of the grid over the unit domain
                    let u = ((x + 2.0) / 4.0).clamp(0.0, 1.0) * (cols - 1) as f64;
                    let v = ((z + 2.0) / 4.0).clamp(0.0, 1.0) * (rows - 1) as f64;
                    let (c0, r0) = (u.floor() as usize, v.floor() as usize);
                    let (c1, r1) = ((c0 + 1).min(cols - 1), (r0 + 1).min(rows - 1));
                    let (fu, fv) = (u - c0 as f64, v - r0 as f64);
                    let top = g[r0][c0] * (1.0 - fu) + g[r0][c1] * fu;
                    let bot = g[r1][c0] * (1.0 - fu) + g[r1][c1] * fu;
                    top * (1.0 - fv) + bot * fv
                });
                surface.rebuild_mesh();
                self.surface = Some(surface);
            }
        }
        if self.surface.is_none() {
            let mut surface = Surface3D::new();
            surface.set_function(48, (-2.0, 2.0), (-2.0, 2.0), |x, z| {
                (x * x + z * z).sqrt().sin() * 0.6
            });
            surface.rebuild_mesh();
            self.surface = Some(surface);
        }
        if let Some(surface) = &mut self.surface {
            surface.set_colormap(colormap_from_name(&self.colormap));
            if surface.needs_rebuild() {
                surface.rebuild_mesh();
            }
        }
    }
}

impl ScriptHook for D3Surface3D {
    fn on_after_apply(
        &mut self,
        vm: &mut ScriptVm,
        _apply: &Apply,
        _scope: &mut Scope,
        _value: ScriptValue,
    ) {
        if let Some(rows) = vm_data::to_rows(vm, self.data) {
            if rows.len() >= 2 {
                self.pending_grid = Some(rows);
                self.surface = None;
            }
        }
    }
}

impl Widget for D3Surface3D {
    fn script_call(
        &mut self,
        vm: &mut ScriptVm,
        method: LiveId,
        args: ScriptValue,
    ) -> ScriptAsyncResult {
        if method == live_id!(set_data) {
            let value = vm_data::arg(vm, args, 0);
            if let Some(rows) = vm_data::to_rows(vm, value) {
                if rows.len() >= 2 {
                    self.set_grid(rows);
                    self.surface = None;
                    vm.with_cx_mut(|cx| self.redraw(cx));
                }
            }
            return ScriptAsyncResult::Return(NIL);
        }
        ScriptAsyncResult::MethodNotFound
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        let hit = event.hits(cx, self.draw_bg.area());
        if matches!(hit, Hit::FingerDown(_) | Hit::FingerHoverIn(_)) {
            cx.set_cursor(MouseCursor::Grab);
        }
        if let Some(surface) = &mut self.surface {
            if camera_hit(hit, |e| surface.handle_camera_event(e)) {
                self.redraw(cx);
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        self.ensure_surface();
        self.rect = cx.walk_turtle(walk);
        self.plot = compute_plot_rect(&self.rect, &self.plot_margin);
        self.draw_bg.draw_abs(cx, self.rect);

        begin_plot(cx, &self.rect, &self.plot_margin, &mut self.draw_vector);

        if let Some(surface) = &self.surface {
            let cm = colormap_from_name(&self.colormap);
            let faces = surface.get_sorted_faces(self.plot.size.x, self.plot.size.y);
            let ox = self.plot.pos.x;
            let oy = self.plot.pos.y;
            for face in &faces {
                let base = cm.sample_rgba(face.data_value);
                let l = light_factor(face.normal);
                self.draw_vector
                    .set_color(base[0] * l, base[1] * l, base[2] * l, 1.0);
                let verts = [
                    [ox + face.screen_verts[0][0], oy + face.screen_verts[0][1]],
                    [ox + face.screen_verts[1][0], oy + face.screen_verts[1][1]],
                    [ox + face.screen_verts[2][0], oy + face.screen_verts[2][1]],
                    [ox + face.screen_verts[3][0], oy + face.screen_verts[3][1]],
                ];
                fill_quad(&mut self.draw_vector, &verts);
            }
        }

        end_plot(cx, &mut self.draw_vector);
        DrawStep::done()
    }
}

// ================= Scatter3D =================

/// 3D scatter plot, exposed as `d3.Scatter3D`.
///
/// `data:` accepts `[{x y z value?}, ...]` or `[[x y z], ...]`.
#[derive(Script, Widget)]
pub struct D3Scatter3D {
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

    /// Declarative data: 3D points.
    #[live]
    data: ScriptValue,
    /// Colormap name (see `d3.Heatmap`).
    #[live]
    pub colormap: String,
    /// Margins around the plot area.
    #[live]
    pub plot_margin: Inset,

    #[rust]
    scatter: Option<Scatter3D>,
    #[rust]
    rect: Rect,
    #[rust]
    plot: Rect,
}

fn parse_points_3d(vm: &mut ScriptVm, value: ScriptValue) -> Option<Vec<ScatterPoint3D>> {
    let items = vm_data::elements(vm, value)?;
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        if item.as_object().is_some() && !vm_data::field(vm, item, live_id!(x)).is_nil() {
            let x = vm_data::field(vm, item, live_id!(x)).as_number();
            let y = vm_data::field(vm, item, live_id!(y)).as_number();
            let z = vm_data::field(vm, item, live_id!(z)).as_number();
            if let (Some(x), Some(y), Some(z)) = (x, y, z) {
                let mut p = ScatterPoint3D::new(x, y, z);
                if let Some(v) = vm_data::field(vm, item, live_id!(value)).as_number() {
                    p = p.with_value(v);
                }
                out.push(p);
            }
        } else if let Some(parts) = vm_data::elements(vm, item) {
            if parts.len() >= 3 {
                if let (Some(x), Some(y), Some(z)) = (
                    parts[0].as_number(),
                    parts[1].as_number(),
                    parts[2].as_number(),
                ) {
                    let mut p = ScatterPoint3D::new(x, y, z);
                    if let Some(v) = parts.get(3).and_then(|v| v.as_number()) {
                        p = p.with_value(v);
                    }
                    out.push(p);
                }
            }
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

fn demo_points_3d() -> Vec<ScatterPoint3D> {
    let mut seed = 5u64;
    let mut r = || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed as f64 / u64::MAX as f64 * 2.0 - 1.0
    };
    (0..250)
        .map(|_| {
            let (x, y, z) = (r() * 1.6, r() * 1.6, r() * 1.6);
            let v = ((x * x + y * y + z * z) / 3.0).sqrt();
            ScatterPoint3D::new(x, y, z).with_value(v)
        })
        .collect()
}

impl D3Scatter3D {
    /// Replace the points from Rust.
    pub fn set_points(&mut self, points: Vec<ScatterPoint3D>) {
        self.scatter_mut().set_points(points);
    }

    fn scatter_mut(&mut self) -> &mut Scatter3D {
        if self.scatter.is_none() {
            let mut scatter = Scatter3D::new();
            scatter.set_points(demo_points_3d());
            self.scatter = Some(scatter);
        }
        self.scatter.as_mut().unwrap()
    }
}

impl ScriptHook for D3Scatter3D {
    fn on_after_apply(
        &mut self,
        vm: &mut ScriptVm,
        _apply: &Apply,
        _scope: &mut Scope,
        _value: ScriptValue,
    ) {
        if let Some(points) = parse_points_3d(vm, self.data) {
            self.scatter_mut().set_points(points);
        }
    }
}

impl Widget for D3Scatter3D {
    fn script_call(
        &mut self,
        vm: &mut ScriptVm,
        method: LiveId,
        args: ScriptValue,
    ) -> ScriptAsyncResult {
        if method == live_id!(set_data) {
            let value = vm_data::arg(vm, args, 0);
            if let Some(points) = parse_points_3d(vm, value) {
                self.scatter_mut().set_points(points);
                vm.with_cx_mut(|cx| self.redraw(cx));
            }
            return ScriptAsyncResult::Return(NIL);
        }
        ScriptAsyncResult::MethodNotFound
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        let hit = event.hits(cx, self.draw_bg.area());
        if matches!(hit, Hit::FingerDown(_) | Hit::FingerHoverIn(_)) {
            cx.set_cursor(MouseCursor::Grab);
        }
        let scatter = self.scatter_mut();
        if camera_hit(hit, |e| scatter.handle_camera_event(e)) {
            self.redraw(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        self.rect = cx.walk_turtle(walk);
        self.plot = compute_plot_rect(&self.rect, &self.plot_margin);
        self.draw_bg.draw_abs(cx, self.rect);

        begin_plot(cx, &self.rect, &self.plot_margin, &mut self.draw_vector);

        let colormap = colormap_from_name(&self.colormap);
        let plot = self.plot;
        let scatter = self.scatter_mut();
        scatter.colormap = colormap;
        let points = scatter.get_projected_points(plot.size.x, plot.size.y);
        for p in &points {
            self.draw_vector
                .set_color(p.color[0], p.color[1], p.color[2], p.color[3]);
            self.draw_vector.circle(
                (plot.pos.x + p.screen_x) as f32,
                (plot.pos.y + p.screen_y) as f32,
                (p.size * 0.5).max(1.0) as f32,
            );
            self.draw_vector.fill();
        }

        end_plot(cx, &mut self.draw_vector);
        DrawStep::done()
    }
}

// ================= Bar3D =================

/// 3D bar chart over a value grid, exposed as `d3.Bar3D`.
///
/// `data:` accepts a grid (array of rows), one bar per cell.
#[derive(Script, Widget)]
pub struct D3Bar3D {
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

    /// Declarative data: value grid (array of rows of numbers).
    #[live]
    data: ScriptValue,
    /// Colormap name (see `d3.Heatmap`).
    #[live]
    pub colormap: String,
    /// Margins around the plot area.
    #[live]
    pub plot_margin: Inset,

    #[rust]
    bars: Option<Bar3D>,
    #[rust]
    rect: Rect,
    #[rust]
    plot: Rect,
}

impl D3Bar3D {
    /// Replace the value grid from Rust.
    pub fn set_grid(&mut self, grid: Vec<Vec<f64>>) {
        self.bars_mut().set_data(grid);
    }

    fn bars_mut(&mut self) -> &mut Bar3D {
        if self.bars.is_none() {
            let mut bars = Bar3D::new();
            bars.set_data(
                (0..6)
                    .map(|r| {
                        (0..6)
                            .map(|c| 1.0 + ((r as f64 * 0.9).sin() + (c as f64 * 0.7).cos() + 2.0))
                            .collect()
                    })
                    .collect(),
            );
            self.bars = Some(bars);
        }
        self.bars.as_mut().unwrap()
    }
}

impl ScriptHook for D3Bar3D {
    fn on_after_apply(
        &mut self,
        vm: &mut ScriptVm,
        _apply: &Apply,
        _scope: &mut Scope,
        _value: ScriptValue,
    ) {
        if let Some(rows) = vm_data::to_rows(vm, self.data) {
            if !rows.is_empty() {
                self.bars_mut().set_data(rows);
            }
        }
    }
}

impl Widget for D3Bar3D {
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
                    self.bars_mut().set_data(rows);
                    vm.with_cx_mut(|cx| self.redraw(cx));
                }
            }
            return ScriptAsyncResult::Return(NIL);
        }
        ScriptAsyncResult::MethodNotFound
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        let hit = event.hits(cx, self.draw_bg.area());
        if matches!(hit, Hit::FingerDown(_) | Hit::FingerHoverIn(_)) {
            cx.set_cursor(MouseCursor::Grab);
        }
        let bars = self.bars_mut();
        if camera_hit(hit, |e| bars.handle_camera_event(e)) {
            self.redraw(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        self.rect = cx.walk_turtle(walk);
        self.plot = compute_plot_rect(&self.rect, &self.plot_margin);
        self.draw_bg.draw_abs(cx, self.rect);

        begin_plot(cx, &self.rect, &self.plot_margin, &mut self.draw_vector);

        let cm = colormap_from_name(&self.colormap);
        let plot = self.plot;
        let bars = self.bars_mut();
        let faces = bars.get_sorted_faces(plot.size.x, plot.size.y);
        for face in &faces {
            let base = cm.sample_rgba(face.data_value);
            // Face-dependent brightness: top brightest, sides darker.
            let l = match face.face_type {
                0 => 1.0f32,
                1 => 0.82,
                4 => 0.78,
                _ => 0.68,
            };
            self.draw_vector
                .set_color(base[0] * l, base[1] * l, base[2] * l, 1.0);
            let verts = [
                [
                    plot.pos.x + face.screen_verts[0][0],
                    plot.pos.y + face.screen_verts[0][1],
                ],
                [
                    plot.pos.x + face.screen_verts[1][0],
                    plot.pos.y + face.screen_verts[1][1],
                ],
                [
                    plot.pos.x + face.screen_verts[2][0],
                    plot.pos.y + face.screen_verts[2][1],
                ],
                [
                    plot.pos.x + face.screen_verts[3][0],
                    plot.pos.y + face.screen_verts[3][1],
                ],
            ];
            fill_quad(&mut self.draw_vector, &verts);
        }

        end_plot(cx, &mut self.draw_vector);
        DrawStep::done()
    }
}

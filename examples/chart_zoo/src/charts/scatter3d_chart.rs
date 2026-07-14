//! 3D Scatter Plot Widget
//!
//! Displays 3D point data with perspective projection and color mapping.
//! GPU-accelerated with orbital camera controls.

use makepad_widgets::*;
use makepad_d3::render3d::{
    Scatter3D, ScatterPoint3D, Colormap,
    CameraEvent, DrawPoint3D,
};
use super::draw_primitives::DrawChartLine;
use super::animation::{ChartAnimator, EasingType};

live_design! {
    link widgets;
    use link::shaders::*;
    use super::draw_primitives::DrawChartLine;
    use crate::render3d::draw::DrawPoint3D;

    pub Scatter3DWidget = {{Scatter3DWidget}} {
        width: Fill,
        height: Fill,
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct Scatter3DWidget {
    #[redraw]
    #[live]
    draw_point: DrawPoint3D,

    #[redraw]
    #[live]
    draw_line: DrawChartLine,

    #[walk]
    walk: Walk,

    #[rust]
    scatter: Scatter3D,

    #[rust]
    animator: ChartAnimator,

    #[rust]
    initialized: bool,

    #[rust]
    area: Area,

    #[rust]
    chart_rect: Rect,

    #[rust(true)]
    animate_rotation: bool,

    #[rust(0.0)]
    time_offset: f64,

    #[rust(false)]
    is_dragging: bool,

    #[rust]
    animation_progress: f64,

    #[rust(true)]
    show_axes: bool,
}

impl Widget for Scatter3DWidget {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        match event {
            Event::MouseDown(me) => {
                if self.area.rect(cx).contains(me.abs) {
                    self.is_dragging = true;
                    self.scatter.handle_camera_event(CameraEvent::PointerDown {
                        pos: [me.abs.x, me.abs.y],
                        shift: me.modifiers.shift,
                    });
                    self.animate_rotation = false;
                }
            }
            Event::MouseMove(me) => {
                if self.is_dragging {
                    self.scatter.handle_camera_event(CameraEvent::PointerMove {
                        pos: [me.abs.x, me.abs.y],
                        shift: me.modifiers.shift,
                    });
                    self.redraw(cx);
                }
            }
            Event::MouseUp(_me) => {
                if self.is_dragging {
                    self.is_dragging = false;
                    self.scatter.handle_camera_event(CameraEvent::PointerUp);
                }
            }
            Event::Scroll(se) => {
                if self.area.rect(cx).contains(se.abs) {
                    self.scatter.handle_camera_event(CameraEvent::Scroll {
                        delta_y: se.scroll.y,
                    });
                    self.redraw(cx);
                }
            }
            Event::KeyDown(ke) => {
                match ke.key_code {
                    KeyCode::KeyR => {
                        self.scatter.camera_mut().reset();
                        self.animate_rotation = true;
                        self.redraw(cx);
                    }
                    KeyCode::Space => {
                        self.animate_rotation = !self.animate_rotation;
                        self.redraw(cx);
                    }
                    _ => {}
                }
            }
            Event::NextFrame(ne) => {
                let time = ne.time;

                if self.animator.is_running() {
                    if self.animator.update(time) {
                        self.animation_progress = self.animator.get_progress();
                        self.redraw(cx);
                    }
                }

                if self.animate_rotation && !self.animator.is_running() {
                    let dt = time - self.time_offset;
                    self.time_offset = time;
                    self.scatter.camera_mut().yaw += dt * 0.3;
                    self.redraw(cx);
                }

                if self.scatter.is_animating() {
                    self.scatter.handle_camera_event(CameraEvent::Frame { dt: 1.0 / 60.0 });
                    self.redraw(cx);
                }

                cx.new_next_frame();
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

            if !self.initialized {
                self.initialize_data();
                self.start_animation(cx.cx);
                self.initialized = true;
            }

            self.draw_scatter_plot(cx);
        }

        DrawStep::done()
    }
}

impl Scatter3DWidget {
    fn initialize_data(&mut self) {
        // Generate sample 3D scatter data - a helix
        let mut points = Vec::new();

        let n_points = 200;
        for i in 0..n_points {
            let t = i as f64 / n_points as f64;
            let theta = t * 4.0 * std::f64::consts::PI;
            let r = 0.8;

            let x = r * theta.cos();
            let y = t * 2.0 - 1.0; // -1 to 1
            let z = r * theta.sin();

            points.push(
                ScatterPoint3D::new(x, y, z)
                    .with_value(t)
                    .with_size(1.0 + t * 0.5)
            );
        }

        // Add some random points around the helix
        let mut rng_seed = 42u64;
        for _ in 0..100 {
            // Simple LCG random
            rng_seed = rng_seed.wrapping_mul(1103515245).wrapping_add(12345);
            let r1 = ((rng_seed >> 16) & 0x7fff) as f64 / 32767.0;
            rng_seed = rng_seed.wrapping_mul(1103515245).wrapping_add(12345);
            let r2 = ((rng_seed >> 16) & 0x7fff) as f64 / 32767.0;
            rng_seed = rng_seed.wrapping_mul(1103515245).wrapping_add(12345);
            let r3 = ((rng_seed >> 16) & 0x7fff) as f64 / 32767.0;

            let x = (r1 - 0.5) * 2.0;
            let y = (r2 - 0.5) * 2.0;
            let z = (r3 - 0.5) * 2.0;

            points.push(
                ScatterPoint3D::new(x, y, z)
                    .with_value(r1)
                    .with_size(0.5)
            );
        }

        self.scatter.set_points(points);
        self.scatter.set_colormap(Colormap::Turbo);
        self.scatter.set_point_size(10.0);
    }

    pub fn set_points(&mut self, points: Vec<ScatterPoint3D>) {
        self.scatter.set_points(points);
    }

    pub fn set_colormap(&mut self, colormap: Colormap) {
        self.scatter.set_colormap(colormap);
    }

    fn start_animation(&mut self, cx: &mut Cx) {
        let time = cx.seconds_since_app_start();
        self.animator = ChartAnimator::new(1000.0)
            .with_easing(EasingType::EaseOutCubic);
        self.animator.start(time);
        self.time_offset = time;
        self.animation_progress = 0.0;
        cx.new_next_frame();
    }

    pub fn replay_animation(&mut self, cx: &mut Cx) {
        self.animator.reset();
        self.start_animation(cx);
        self.redraw(cx);
    }

    fn draw_scatter_plot(&mut self, cx: &mut Cx2d) {
        let rect = self.chart_rect;

        // Draw axes if enabled
        if self.show_axes {
            self.draw_axes(cx, rect);
        }

        // Get projected points
        let points = self.scatter.get_projected_points(rect.size.x, rect.size.y);

        // Draw points (back to front)
        for point in &points {
            // Apply animation progress to size
            let animated_size = point.size * self.animation_progress;
            if animated_size < 1.0 {
                continue;
            }

            let center = dvec2(rect.pos.x + point.screen_x, rect.pos.y + point.screen_y);

            // Set color
            self.draw_point.color = vec4(
                point.color[0],
                point.color[1],
                point.color[2],
                point.color[3],
            );

            self.draw_point.draw_point(cx, center, animated_size);
        }
    }

    fn draw_axes(&mut self, cx: &mut Cx2d, rect: Rect) {
        // Simple axis lines at origin
        // This is a simplified version - in a full implementation,
        // we'd project the 3D axis lines properly

        let center = dvec2(rect.pos.x + rect.size.x / 2.0, rect.pos.y + rect.size.y / 2.0);
        let axis_len = rect.size.x.min(rect.size.y) * 0.3;

        // X axis (red)
        self.draw_line.color = vec4(0.8, 0.2, 0.2, 0.5);
        self.draw_line.draw_line(cx, center, dvec2(center.x + axis_len, center.y), 1.0);

        // Y axis (green)
        self.draw_line.color = vec4(0.2, 0.8, 0.2, 0.5);
        self.draw_line.draw_line(cx, center, dvec2(center.x, center.y - axis_len), 1.0);

        // Z axis (blue) - projected at 45 degrees
        self.draw_line.color = vec4(0.2, 0.2, 0.8, 0.5);
        self.draw_line.draw_line(
            cx,
            center,
            dvec2(center.x - axis_len * 0.5, center.y + axis_len * 0.3),
            1.0,
        );
    }
}

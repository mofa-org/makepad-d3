//! 3D Bar Chart Widget
//!
//! Displays 3D bar chart data with perspective projection and color mapping.
//! GPU-accelerated with orbital camera controls.

use makepad_widgets::*;
use makepad_d3::render3d::{
    Bar3D, Colormap, CameraEvent,
};
use super::draw_primitives::{DrawChartLine, DrawTriangle};
use super::animation::{ChartAnimator, EasingType};

live_design! {
    link widgets;
    use link::shaders::*;
    use super::draw_primitives::DrawChartLine;
    use super::draw_primitives::DrawTriangle;

    pub Bar3DWidget = {{Bar3DWidget}} {
        width: Fill,
        height: Fill,
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct Bar3DWidget {
    #[redraw]
    #[live]
    draw_face: DrawTriangle,

    #[redraw]
    #[live]
    draw_line: DrawChartLine,

    #[walk]
    walk: Walk,

    #[rust]
    bar_chart: Bar3D,

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
    show_outlines: bool,
}

impl Widget for Bar3DWidget {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        match event {
            Event::MouseDown(me) => {
                if self.area.rect(cx).contains(me.abs) {
                    self.is_dragging = true;
                    self.bar_chart.handle_camera_event(CameraEvent::PointerDown {
                        pos: [me.abs.x, me.abs.y],
                        shift: me.modifiers.shift,
                    });
                    self.animate_rotation = false;
                }
            }
            Event::MouseMove(me) => {
                if self.is_dragging {
                    self.bar_chart.handle_camera_event(CameraEvent::PointerMove {
                        pos: [me.abs.x, me.abs.y],
                        shift: me.modifiers.shift,
                    });
                    self.redraw(cx);
                }
            }
            Event::MouseUp(_me) => {
                if self.is_dragging {
                    self.is_dragging = false;
                    self.bar_chart.handle_camera_event(CameraEvent::PointerUp);
                }
            }
            Event::Scroll(se) => {
                if self.area.rect(cx).contains(se.abs) {
                    self.bar_chart.handle_camera_event(CameraEvent::Scroll {
                        delta_y: se.scroll.y,
                    });
                    self.redraw(cx);
                }
            }
            Event::KeyDown(ke) => {
                match ke.key_code {
                    KeyCode::KeyR => {
                        self.bar_chart.camera_mut().reset();
                        self.animate_rotation = true;
                        self.redraw(cx);
                    }
                    KeyCode::Space => {
                        self.animate_rotation = !self.animate_rotation;
                        self.redraw(cx);
                    }
                    KeyCode::KeyO => {
                        self.show_outlines = !self.show_outlines;
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
                    self.bar_chart.camera_mut().yaw += dt * 0.2;
                    self.redraw(cx);
                }

                if self.bar_chart.is_animating() {
                    self.bar_chart.handle_camera_event(CameraEvent::Frame { dt: 1.0 / 60.0 });
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

            self.draw_bar_chart(cx);
        }

        DrawStep::done()
    }
}

impl Bar3DWidget {
    fn initialize_data(&mut self) {
        // Generate sample 3D bar chart data
        // A 5x5 grid with varying heights
        let data = vec![
            vec![0.2, 0.5, 0.8, 0.6, 0.3],
            vec![0.4, 0.7, 1.0, 0.8, 0.5],
            vec![0.3, 0.6, 0.9, 0.7, 0.4],
            vec![0.5, 0.8, 0.7, 0.5, 0.3],
            vec![0.2, 0.4, 0.5, 0.3, 0.1],
        ];

        self.bar_chart.set_data(data);
        self.bar_chart.set_colormap(Colormap::Viridis);
        self.bar_chart.set_bar_size(0.8, 0.8);
        self.bar_chart.set_gap(0.1);
    }

    pub fn set_data(&mut self, data: Vec<Vec<f64>>) {
        self.bar_chart.set_data(data);
    }

    pub fn set_colormap(&mut self, colormap: Colormap) {
        self.bar_chart.set_colormap(colormap);
    }

    fn start_animation(&mut self, cx: &mut Cx) {
        let time = cx.seconds_since_app_start();
        self.animator = ChartAnimator::new(1200.0)
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

    fn draw_bar_chart(&mut self, cx: &mut Cx2d) {
        let rect = self.chart_rect;

        // Get sorted faces
        let faces = self.bar_chart.get_sorted_faces(rect.size.x, rect.size.y);

        // Draw faces (back to front)
        for face in &faces {
            // Apply animation progress
            let animated_verts: Vec<[f64; 2]> = face.screen_verts
                .iter()
                .map(|v| {
                    // Animate y position based on progress
                    let y_center = rect.pos.y + rect.size.y / 2.0;
                    let animated_y = y_center + (v[1] - y_center) * self.animation_progress;
                    [rect.pos.x + v[0], animated_y]
                })
                .collect();

            let p0 = dvec2(animated_verts[0][0], animated_verts[0][1]);
            let p1 = dvec2(animated_verts[1][0], animated_verts[1][1]);
            let p2 = dvec2(animated_verts[2][0], animated_verts[2][1]);
            let p3 = dvec2(animated_verts[3][0], animated_verts[3][1]);

            // Get face color
            let color = self.bar_chart.get_face_color(face);
            self.draw_face.color = vec4(color[0], color[1], color[2], color[3]);
            self.draw_face.disable_gradient();

            // Draw two triangles to form quad
            self.draw_face.draw_triangle(cx, p0, p1, p2);
            self.draw_face.draw_triangle(cx, p0, p2, p3);

            // Draw outlines if enabled
            if self.show_outlines {
                let outline_color = self.bar_chart.outline_color;
                self.draw_line.color = vec4(
                    outline_color[0],
                    outline_color[1],
                    outline_color[2],
                    outline_color[3],
                );

                self.draw_line.draw_line(cx, p0, p1, 1.0);
                self.draw_line.draw_line(cx, p1, p2, 1.0);
                self.draw_line.draw_line(cx, p2, p3, 1.0);
                self.draw_line.draw_line(cx, p3, p0, 1.0);
            }
        }
    }
}

//! 3D Surface Plot Widget
//!
//! Displays 3D surface data with perspective projection and color mapping.
//! GPU-accelerated with rotation animation and proper depth sorting.
//!
//! This implementation uses the makepad_d3::render3d module for:
//! - Pre-computed mesh geometry
//! - Camera-based transforms
//! - GPU shader lighting and colormaps

use makepad_widgets::*;
use makepad_d3::render3d::{
    Surface3D, SurfaceData, Colormap,
    CameraEvent, DrawSurface3D, DrawWireframe3D,
};
use super::draw_primitives::{DrawChartLine, DrawTriangle};
use super::animation::{ChartAnimator, EasingType};

live_design! {
    link widgets;
    use link::shaders::*;
    use super::draw_primitives::DrawChartLine;
    use super::draw_primitives::DrawTriangle;
    use crate::render3d::draw::DrawSurface3D;
    use crate::render3d::draw::DrawWireframe3D;

    pub SurfacePlotWidget = {{SurfacePlotWidget}} {
        width: Fill,
        height: Fill,
    }
}

/// Color map types for surface visualization (legacy compatibility)
#[derive(Clone, Copy, Debug, Default)]
pub enum ColorMap {
    #[default]
    Viridis,
    Plasma,
    Inferno,
    Magma,
    CoolWarm,
    Terrain,
    Rainbow,
}

impl ColorMap {
    /// Convert to render3d Colormap
    pub fn to_colormap(&self) -> Colormap {
        match self {
            ColorMap::Viridis => Colormap::Viridis,
            ColorMap::Plasma => Colormap::Plasma,
            ColorMap::Inferno => Colormap::Inferno,
            ColorMap::Magma => Colormap::Magma,
            ColorMap::CoolWarm => Colormap::CoolWarm,
            ColorMap::Terrain => Colormap::Viridis, // Fallback
            ColorMap::Rainbow => Colormap::Turbo,   // Close approximation
        }
    }

    pub fn get_color(&self, t: f64) -> Vec4 {
        let color = self.to_colormap().sample(t as f32);
        vec4(color.x, color.y, color.z, 1.0)
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct SurfacePlotWidget {
    #[redraw]
    #[live]
    draw_line: DrawChartLine,

    #[redraw]
    #[live]
    draw_face: DrawTriangle,

    #[redraw]
    #[live]
    draw_surface: DrawSurface3D,

    #[redraw]
    #[live]
    draw_wireframe: DrawWireframe3D,

    #[walk]
    walk: Walk,

    #[rust]
    surface: Surface3D,

    #[rust]
    animator: ChartAnimator,

    #[rust]
    initialized: bool,

    #[rust]
    area: Area,

    #[rust]
    chart_rect: Rect,

    #[rust(25usize)]
    resolution: usize,

    #[rust]
    color_map: ColorMap,

    #[rust(true)]
    show_wireframe: bool,

    #[rust(true)]
    show_surface: bool,

    #[rust(true)]
    animate_rotation: bool,

    #[rust(0.0)]
    time_offset: f64,

    #[rust(false)]
    is_dragging: bool,

    #[rust]
    last_mouse_pos: DVec2,
}

impl Widget for SurfacePlotWidget {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        match event {
            Event::MouseDown(me) => {
                if self.area.rect(cx).contains(me.abs) {
                    self.is_dragging = true;
                    self.last_mouse_pos = me.abs;
                    self.surface.handle_camera_event(CameraEvent::PointerDown {
                        pos: [me.abs.x, me.abs.y],
                        shift: me.modifiers.shift,
                    });
                    // Stop auto-rotation when dragging
                    self.animate_rotation = false;
                }
            }
            Event::MouseMove(me) => {
                if self.is_dragging {
                    self.surface.handle_camera_event(CameraEvent::PointerMove {
                        pos: [me.abs.x, me.abs.y],
                        shift: me.modifiers.shift,
                    });
                    self.last_mouse_pos = me.abs;
                    self.redraw(cx);
                }
            }
            Event::MouseUp(_me) => {
                if self.is_dragging {
                    self.is_dragging = false;
                    self.surface.handle_camera_event(CameraEvent::PointerUp);
                }
            }
            Event::Scroll(se) => {
                if self.area.rect(cx).contains(se.abs) {
                    self.surface.handle_camera_event(CameraEvent::Scroll {
                        delta_y: se.scroll.y,
                    });
                    self.redraw(cx);
                }
            }
            Event::KeyDown(ke) => {
                // Handle keyboard shortcuts
                match ke.key_code {
                    KeyCode::KeyR => {
                        // Reset camera and restart animation
                        self.surface.camera_mut().reset();
                        self.animate_rotation = true;
                        self.redraw(cx);
                    }
                    KeyCode::KeyW => {
                        // Toggle wireframe
                        self.show_wireframe = !self.show_wireframe;
                        self.redraw(cx);
                    }
                    KeyCode::KeyS => {
                        // Toggle surface
                        self.show_surface = !self.show_surface;
                        self.redraw(cx);
                    }
                    KeyCode::Space => {
                        // Toggle rotation animation
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
                        let progress = self.animator.get_progress();
                        self.surface.set_animation_progress(progress);
                        self.redraw(cx);
                    }
                }

                // Continuous rotation animation
                if self.animate_rotation && !self.animator.is_running() {
                    let dt = time - self.time_offset;
                    self.time_offset = time;
                    // Rotate camera slowly
                    self.surface.camera_mut().yaw += dt * 0.3;
                    self.redraw(cx);
                }

                // Camera animation update
                if self.surface.camera_controller.needs_update() {
                    self.surface.handle_camera_event(CameraEvent::Frame { dt: 1.0 / 60.0 });
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

            self.draw_surface_plot(cx);
        }

        DrawStep::done()
    }
}

impl SurfacePlotWidget {
    fn initialize_data(&mut self) {
        // Set default resolution
        if self.resolution == 0 {
            self.resolution = 25;
        }

        // Create interesting mathematical surface using Surface3D
        self.surface.set_function(
            self.resolution,
            (-2.0, 2.0),
            (-2.0, 2.0),
            |x, z| {
                // Combination of sinusoidal functions (like a standing wave)
                let r = (x * x + z * z).sqrt();
                let wave = (r * 3.0).sin() * (-r * 0.3).exp();

                // Add some Gaussian bumps
                let bump1 = (-((x - 0.8).powi(2) + (z - 0.8).powi(2)) / 0.3).exp() * 0.5;
                let bump2 = (-((x + 0.8).powi(2) + (z + 0.8).powi(2)) / 0.4).exp() * 0.4;

                wave + bump1 + bump2
            },
        );

        // Rebuild mesh data
        self.surface.rebuild_mesh();

        // Set default color map
        self.color_map = ColorMap::Viridis;
        self.surface.set_colormap(self.color_map.to_colormap());

        // Configure surface options
        self.surface.show_surface = self.show_surface;
        self.surface.show_wireframe = self.show_wireframe;
    }

    pub fn set_data(&mut self, data: SurfaceData) {
        self.surface.set_data(data);
        self.surface.rebuild_mesh();
        self.initialized = true;
    }

    pub fn set_color_map(&mut self, color_map: ColorMap) {
        self.color_map = color_map;
        self.surface.set_colormap(color_map.to_colormap());
    }

    fn start_animation(&mut self, cx: &mut Cx) {
        let time = cx.seconds_since_app_start();
        self.animator = ChartAnimator::new(1500.0)
            .with_easing(EasingType::EaseOutCubic);
        self.animator.start(time);
        self.time_offset = time;
        self.surface.set_animation_progress(0.0);
        cx.new_next_frame();
    }

    pub fn replay_animation(&mut self, cx: &mut Cx) {
        self.animator.reset();
        self.start_animation(cx);
        self.redraw(cx);
    }

    fn draw_surface_plot(&mut self, cx: &mut Cx2d) {
        let rect = self.chart_rect;

        // Get sorted faces from Surface3D
        let faces = self.surface.get_sorted_faces(rect.size.x, rect.size.y);

        if faces.is_empty() {
            return;
        }

        // Draw faces (back to front for proper depth ordering)
        for face in &faces {
            // Offset screen positions by chart rect position
            let p0 = dvec2(rect.pos.x + face.screen_verts[0][0], rect.pos.y + face.screen_verts[0][1]);
            let p1 = dvec2(rect.pos.x + face.screen_verts[1][0], rect.pos.y + face.screen_verts[1][1]);
            let p2 = dvec2(rect.pos.x + face.screen_verts[2][0], rect.pos.y + face.screen_verts[2][1]);
            let p3 = dvec2(rect.pos.x + face.screen_verts[3][0], rect.pos.y + face.screen_verts[3][1]);

            if self.show_surface {
                // Set up surface shader
                self.draw_surface.set_normal(face.normal[0], face.normal[1], face.normal[2]);
                self.draw_surface.set_data_value(face.data_value);
                self.draw_surface.set_colormap(self.surface.colormap);

                // Draw two triangles to form quad
                // Use draw_face.draw_triangle for now since DrawSurface3D is quad-based
                let color = self.color_map.get_color(face.data_value as f64);

                // Apply lighting based on normal (simplified)
                let light_dir = [0.3f32, 0.8, 0.5];
                let light_len = (light_dir[0] * light_dir[0] + light_dir[1] * light_dir[1] + light_dir[2] * light_dir[2]).sqrt();
                let light_norm = [light_dir[0] / light_len, light_dir[1] / light_len, light_dir[2] / light_len];

                let n_dot_l = (face.normal[0] * light_norm[0]
                    + face.normal[1] * light_norm[1]
                    + face.normal[2] * light_norm[2]).max(0.0);

                let ambient = 0.3;
                let diffuse = 0.6 * n_dot_l;
                let brightness = ambient + diffuse;

                let lit_color = vec4(
                    (color.x * brightness).min(1.0),
                    (color.y * brightness).min(1.0),
                    (color.z * brightness).min(1.0),
                    color.w,
                );

                self.draw_face.color = lit_color;
                self.draw_face.disable_gradient();

                self.draw_face.draw_triangle(cx, p0, p1, p2);
                self.draw_face.draw_triangle(cx, p0, p2, p3);
            }

            if self.show_wireframe {
                // Draw edges with darker color
                let base_color = self.color_map.get_color(face.data_value as f64);
                let line_color = vec4(
                    base_color.x * 0.5,
                    base_color.y * 0.5,
                    base_color.z * 0.5,
                    0.7,
                );
                self.draw_line.color = line_color;

                self.draw_line.draw_line(cx, p0, p1, 1.0);
                self.draw_line.draw_line(cx, p1, p2, 1.0);
                self.draw_line.draw_line(cx, p2, p3, 1.0);
                self.draw_line.draw_line(cx, p3, p0, 1.0);
            }
        }
    }
}

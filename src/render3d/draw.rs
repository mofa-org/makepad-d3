//! GPU-accelerated 3D drawing primitives
//!
//! This module provides Makepad-compatible drawing primitives with
//! 3D lighting effects and colormap support.
//!
//! The shader types are registered into the Splash script VM under the
//! `d3` module by [`crate::script_mod`]; call that from your app's
//! `AppMain::script_mod` (after `makepad_widgets::script_mod`) before
//! using any widget that draws with these primitives.

// The `script_mod!` macro generates a public registration function that
// cannot carry a doc comment.
#![allow(missing_docs)]

use makepad_widgets::*;

script_mod! {
    use mod.pod.*
    use mod.math.*
    use mod.shader.*
    use mod.draw

    // 3D surface face with Phong lighting and colormap.
    // Renders a quad as a 3D surface patch with diffuse and specular lighting.
    mod.d3.DrawSurface3D = mod.std.set_type_default() do #(DrawSurface3D::script_shader(vm)){
        ..mod.draw.DrawQuad

        pixel: fn(){
            // Sample colormap based on data value
            let t = clamp(self.data_value, 0.0, 1.0);
            let base_color = self.apply_colormap(t);

            // Normal vector (interpolated across surface)
            let normal = normalize(vec3(
                self.normal_x,
                self.normal_y,
                self.normal_z
            ));

            // Light direction (from top-right-front)
            let light_dir = normalize(vec3(0.3, 0.8, 0.5));

            // View direction (from front)
            let view_dir = normalize(vec3(0.0, 0.0, 1.0));

            // Diffuse lighting (Lambertian)
            let n_dot_l = max(dot(normal, light_dir), 0.0);
            let diffuse = self.ambient + n_dot_l * self.diffuse;

            // Specular lighting (Blinn-Phong)
            let halfway = normalize(light_dir + view_dir);
            let n_dot_h = max(dot(normal, halfway), 0.0);
            let specular = pow(n_dot_h, self.shininess) * self.specular;

            // Two-tone shading (darker undersides)
            let bottom_factor = max(0.0, -normal.y) * 0.3;
            let adjusted_diffuse = diffuse * (1.0 - bottom_factor);

            // Final color
            let lit_color = base_color * adjusted_diffuse + vec3(specular, specular, specular);

            // Apply opacity
            let alpha = self.opacity;

            return vec4(lit_color * alpha, alpha);
        }

        apply_colormap: fn(t){
            let t = clamp(t, 0.0, 1.0);

            // Colormap selection based on colormap_type instance value
            if self.colormap_type < 0.5 {
                // Viridis
                return self.colormap_viridis(t);
            }
            if self.colormap_type < 1.5 {
                // Plasma
                return self.colormap_plasma(t);
            }
            if self.colormap_type < 2.5 {
                // Inferno
                return self.colormap_inferno(t);
            }
            if self.colormap_type < 3.5 {
                // Magma (simplified to viridis-like)
                return self.colormap_viridis(t);
            }
            if self.colormap_type < 4.5 {
                // Cool-Warm
                return self.colormap_cool_warm(t);
            }
            if self.colormap_type < 5.5 {
                // Turbo
                return self.colormap_turbo(t);
            }
            // Grayscale
            return vec3(t, t, t);
        }

        colormap_viridis: fn(t){
            let c0 = vec3(0.267, 0.004, 0.329);
            let c1 = vec3(0.282, 0.140, 0.458);
            let c2 = vec3(0.254, 0.265, 0.530);
            let c3 = vec3(0.163, 0.471, 0.558);
            let c4 = vec3(0.134, 0.658, 0.517);
            let c5 = vec3(0.477, 0.821, 0.318);
            let c6 = vec3(0.993, 0.906, 0.144);

            let t6 = t * 6.0;
            let i = floor(t6);
            let f = fract(t6);

            if i < 1.0 { return mix(c0, c1, f); }
            if i < 2.0 { return mix(c1, c2, f); }
            if i < 3.0 { return mix(c2, c3, f); }
            if i < 4.0 { return mix(c3, c4, f); }
            if i < 5.0 { return mix(c4, c5, f); }
            return mix(c5, c6, f);
        }

        colormap_plasma: fn(t){
            let c0 = vec3(0.050, 0.030, 0.528);
            let c1 = vec3(0.494, 0.012, 0.658);
            let c2 = vec3(0.798, 0.280, 0.470);
            let c3 = vec3(0.973, 0.580, 0.254);
            let c4 = vec3(0.940, 0.975, 0.131);

            let t4 = t * 4.0;
            let i = floor(t4);
            let f = fract(t4);

            if i < 1.0 { return mix(c0, c1, f); }
            if i < 2.0 { return mix(c1, c2, f); }
            if i < 3.0 { return mix(c2, c3, f); }
            return mix(c3, c4, f);
        }

        colormap_inferno: fn(t){
            let c0 = vec3(0.001, 0.000, 0.014);
            let c1 = vec3(0.329, 0.059, 0.406);
            let c2 = vec3(0.735, 0.216, 0.330);
            let c3 = vec3(0.976, 0.559, 0.040);
            let c4 = vec3(0.988, 0.998, 0.645);

            let t4 = t * 4.0;
            let i = floor(t4);
            let f = fract(t4);

            if i < 1.0 { return mix(c0, c1, f); }
            if i < 2.0 { return mix(c1, c2, f); }
            if i < 3.0 { return mix(c2, c3, f); }
            return mix(c3, c4, f);
        }

        colormap_cool_warm: fn(t){
            let blue = vec3(0.230, 0.299, 0.754);
            let white = vec3(0.865, 0.865, 0.865);
            let red = vec3(0.706, 0.016, 0.150);

            if t < 0.5 {
                return mix(blue, white, t * 2.0);
            }
            return mix(white, red, (t - 0.5) * 2.0);
        }

        colormap_turbo: fn(t){
            let r = 0.13572138 + t * (4.61539260 + t * (-42.66032258 + t * (132.13108234 + t * (-152.94239396 + t * 59.28637943))));
            let g = 0.09140261 + t * (2.19418839 + t * (4.84296658 + t * (-14.18503333 + t * (4.27729857 + t * 2.82956604))));
            let b = 0.10667330 + t * (12.64194608 + t * (-60.58204836 + t * (110.36276771 + t * (-89.90310912 + t * 27.34824973))));
            return vec3(clamp(r, 0.0, 1.0), clamp(g, 0.0, 1.0), clamp(b, 0.0, 1.0));
        }
    }

    // 3D wireframe line with depth-based fading
    mod.d3.DrawWireframe3D = mod.std.set_type_default() do #(DrawWireframe3D::script_shader(vm)){
        ..mod.draw.DrawQuad

        pixel: fn(){
            let uv = self.pos;

            // Line endpoints in normalized coordinates (0-1)
            let p1 = vec2(self.x1, self.y1);
            let p2 = vec2(self.x2, self.y2);

            let p = uv;

            // Vector from p1 to p2
            let line_vec = p2 - p1;
            let line_len = length(line_vec);

            if line_len < 0.001 {
                return vec4(0.0, 0.0, 0.0, 0.0);
            }

            // Project point onto line
            let t = clamp(dot(p - p1, line_vec) / (line_len * line_len), 0.0, 1.0);
            let closest = p1 + t * line_vec;

            // Distance from point to line
            let dist = length(p - closest);

            // Line width in normalized coordinates
            let half_width = self.line_width * 0.5;

            // Anti-aliased edge
            let aa = 0.02;
            let alpha = 1.0 - smoothstep(half_width - aa, half_width + aa, dist);

            if alpha < 0.01 {
                return vec4(0.0, 0.0, 0.0, 0.0);
            }

            // Depth-based fading (further = more transparent)
            let depth_factor = 1.0 - self.depth * 0.5;
            let final_alpha = alpha * self.opacity * depth_factor;

            return vec4(self.color.rgb * final_alpha, final_alpha);
        }
    }

    // 3D point/sphere with lighting
    mod.d3.DrawPoint3D = mod.std.set_type_default() do #(DrawPoint3D::script_shader(vm)){
        ..mod.draw.DrawQuad

        pixel: fn(){
            let uv = self.pos;
            let center = vec2(0.5, 0.5);
            let dist = distance(uv, center) * 2.0;

            // Check if inside circle
            if dist > 1.0 {
                return vec4(0.0, 0.0, 0.0, 0.0);
            }

            // Calculate sphere normal from UV position
            let nx = (uv.x - 0.5) * 2.0;
            let ny = (uv.y - 0.5) * 2.0;
            let nz_sq = 1.0 - nx * nx - ny * ny;

            if nz_sq < 0.0 {
                return vec4(0.0, 0.0, 0.0, 0.0);
            }

            let nz = sqrt(nz_sq);
            let normal = normalize(vec3(nx, -ny, nz));

            // Light direction
            let light_dir = normalize(vec3(0.3, 0.8, 0.5));
            let view_dir = vec3(0.0, 0.0, 1.0);

            // Diffuse
            let n_dot_l = max(dot(normal, light_dir), 0.0);
            let diffuse = 0.3 + n_dot_l * 0.7;

            // Specular
            let halfway = normalize(light_dir + view_dir);
            let n_dot_h = max(dot(normal, halfway), 0.0);
            let specular = pow(n_dot_h, 32.0) * 0.5;

            // Get base color from colormap or direct color
            let base_color = self.color.rgb;

            // Apply lighting
            let lit_color = base_color * diffuse + vec3(specular, specular, specular);

            // Anti-alias the edge
            let aa = 0.05;
            let alpha = 1.0 - smoothstep(1.0 - aa, 1.0, dist);

            return vec4(lit_color * alpha, alpha);
        }
    }

    // 3D bar/column with per-face lighting
    mod.d3.DrawBar3D = mod.std.set_type_default() do #(DrawBar3D::script_shader(vm)){
        ..mod.draw.DrawQuad

        pixel: fn(){
            // Base color from colormap
            let t = clamp(self.data_value, 0.0, 1.0);
            let base_color = self.apply_colormap(t);

            // Different shading per face: 0 = top (brightest),
            // 1 = front (medium), 2 = right side (darkest)
            if self.face_type < 0.5 {
                return vec4(base_color * 1.0, 1.0);
            }
            if self.face_type < 1.5 {
                return vec4(base_color * 0.85, 1.0);
            }
            return vec4(base_color * 0.7, 1.0);
        }

        apply_colormap: fn(t){
            // Simplified viridis for bars
            let c0 = vec3(0.267, 0.004, 0.329);
            let c1 = vec3(0.254, 0.265, 0.530);
            let c2 = vec3(0.134, 0.658, 0.517);
            let c3 = vec3(0.993, 0.906, 0.144);

            let t3 = t * 3.0;
            let i = floor(t3);
            let f = fract(t3);

            if i < 1.0 { return mix(c0, c1, f); }
            if i < 2.0 { return mix(c1, c2, f); }
            return mix(c2, c3, f);
        }
    }

    // Grid plane for 3D scene reference
    mod.d3.DrawGrid3D = mod.std.set_type_default() do #(DrawGrid3D::script_shader(vm)){
        ..mod.draw.DrawQuad

        pixel: fn(){
            let uv = self.pos;

            // Grid parameters
            let grid_cells = self.grid_divisions;
            let gx = uv.x * grid_cells;
            let gy = uv.y * grid_cells;

            // Distance to nearest grid line
            let dx = abs(gx - floor(gx + 0.5));
            let dy = abs(gy - floor(gy + 0.5));

            let line_width = self.line_width;

            // Anti-aliased grid lines
            let line_x = 1.0 - smoothstep(line_width - 0.01, line_width + 0.01, dx);
            let line_y = 1.0 - smoothstep(line_width - 0.01, line_width + 0.01, dy);

            let line = max(line_x, line_y);

            if line < 0.01 {
                return vec4(0.0, 0.0, 0.0, 0.0);
            }

            // Fade based on depth
            let depth_fade = 1.0 - self.depth * 0.3;
            let alpha = line * self.opacity * depth_fade;

            return vec4(self.color.rgb * alpha, alpha);
        }
    }

    // Axis line with arrow
    mod.d3.DrawAxis3D = mod.std.set_type_default() do #(DrawAxis3D::script_shader(vm)){
        ..mod.draw.DrawQuad

        pixel: fn(){
            let uv = self.pos;

            // Line from (0.1, 0.5) to (0.85, 0.5) with arrow at end
            let line_start = vec2(0.1, 0.5);
            let line_end = vec2(0.85, 0.5);

            let p = uv;
            let line_vec = line_end - line_start;
            let line_len = length(line_vec);

            // Project onto line
            let t = clamp(dot(p - line_start, line_vec) / (line_len * line_len), 0.0, 1.0);
            let closest = line_start + t * line_vec;
            let dist = length(p - closest);

            // Line thickness
            let half_width = 0.03;
            let line_alpha = 1.0 - smoothstep(half_width - 0.01, half_width + 0.01, dist);

            // Arrow head (triangle at end)
            let arrow_size = 0.08;
            let arrow_tip = vec2(0.95, 0.5);
            let arrow_top = vec2(0.85, 0.5 + arrow_size);
            let arrow_bot = vec2(0.85, 0.5 - arrow_size);

            // Check if in arrow triangle
            let in_arrow = self.point_in_triangle(p, arrow_tip, arrow_top, arrow_bot);

            let alpha = max(line_alpha, in_arrow);

            if alpha < 0.01 {
                return vec4(0.0, 0.0, 0.0, 0.0);
            }

            return vec4(self.color.rgb * alpha, alpha);
        }

        point_in_triangle: fn(p, v0, v1, v2){
            let d00 = dot(v1 - v0, v1 - v0);
            let d01 = dot(v1 - v0, v2 - v0);
            let d11 = dot(v2 - v0, v2 - v0);
            let d20 = dot(p - v0, v1 - v0);
            let d21 = dot(p - v0, v2 - v0);

            let denom = d00 * d11 - d01 * d01;
            if abs(denom) < 0.0001 {
                return 0.0;
            }

            let inv_denom = 1.0 / denom;
            let u = (d11 * d20 - d01 * d21) * inv_denom;
            let v = (d00 * d21 - d01 * d20) * inv_denom;

            if u >= 0.0 && v >= 0.0 && (u + v) <= 1.0 {
                return 1.0;
            }
            return 0.0;
        }
    }
}

// ============ Rust Structs ============

/// 3D surface face drawing primitive with lighting and colormaps
#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawSurface3D {
    /// Base quad shader state
    #[deref]
    pub draw_super: DrawQuad,

    /// Normal vector X component
    #[live]
    pub normal_x: f32,
    /// Normal vector Y component
    #[live]
    pub normal_y: f32,
    /// Normal vector Z component
    #[live(1.0)]
    pub normal_z: f32,

    /// Data value for colormap (0.0 to 1.0)
    #[live]
    pub data_value: f32,

    /// Colormap type (0=Viridis, 1=Plasma, 2=Inferno, 3=Magma, 4=CoolWarm, 5=Turbo, 6=Gray)
    #[live]
    pub colormap_type: f32,

    /// Ambient light strength
    #[live(0.3)]
    pub ambient: f32,
    /// Diffuse light strength
    #[live(0.6)]
    pub diffuse: f32,
    /// Specular highlight strength
    #[live(0.4)]
    pub specular: f32,
    /// Specular shininess exponent
    #[live(32.0)]
    pub shininess: f32,

    /// Overall opacity
    #[live(1.0)]
    pub opacity: f32,
}

impl DrawSurface3D {
    /// Draw a surface patch at the given screen rectangle
    pub fn draw_face(&mut self, cx: &mut Cx2d, rect: Rect) {
        self.draw_abs(cx, rect);
    }

    /// Set the surface normal
    pub fn set_normal(&mut self, nx: f32, ny: f32, nz: f32) {
        self.normal_x = nx;
        self.normal_y = ny;
        self.normal_z = nz;
    }

    /// Set the data value for colormap lookup
    pub fn set_data_value(&mut self, value: f32) {
        self.data_value = value.clamp(0.0, 1.0);
    }

    /// Set the colormap type
    pub fn set_colormap(&mut self, colormap: super::colormap::Colormap) {
        self.colormap_type = colormap.to_shader_value();
    }

    /// Set lighting parameters
    pub fn set_lighting(&mut self, ambient: f32, diffuse: f32, specular: f32, shininess: f32) {
        self.ambient = ambient;
        self.diffuse = diffuse;
        self.specular = specular;
        self.shininess = shininess;
    }
}

/// 3D wireframe line drawing primitive
#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawWireframe3D {
    /// Base quad shader state
    #[deref]
    pub draw_super: DrawQuad,

    /// Line color
    #[live]
    pub color: Vec4f,

    /// Line endpoint X1 (normalized 0-1 within bounding rect)
    #[live]
    pub x1: f32,
    /// Line endpoint Y1 (normalized 0-1 within bounding rect)
    #[live]
    pub y1: f32,
    /// Line endpoint X2 (normalized 0-1 within bounding rect)
    #[live]
    pub x2: f32,
    /// Line endpoint Y2 (normalized 0-1 within bounding rect)
    #[live]
    pub y2: f32,

    /// Line width
    #[live(0.02)]
    pub line_width: f32,

    /// Depth value for fading (0=near, 1=far)
    #[live]
    pub depth: f32,

    /// Opacity
    #[live(1.0)]
    pub opacity: f32,
}

impl DrawWireframe3D {
    /// Draw a line between two screen points
    pub fn draw_line(&mut self, cx: &mut Cx2d, p1: DVec2, p2: DVec2, width: f64) {
        let padding = width * 2.0;
        let min_x = p1.x.min(p2.x) - padding;
        let max_x = p1.x.max(p2.x) + padding;
        let min_y = p1.y.min(p2.y) - padding;
        let max_y = p1.y.max(p2.y) + padding;

        let rect_width = max_x - min_x;
        let rect_height = max_y - min_y;

        if rect_width < 1.0 || rect_height < 1.0 {
            return;
        }

        self.x1 = ((p1.x - min_x) / rect_width) as f32;
        self.y1 = ((p1.y - min_y) / rect_height) as f32;
        self.x2 = ((p2.x - min_x) / rect_width) as f32;
        self.y2 = ((p2.y - min_y) / rect_height) as f32;

        let min_dim = rect_width.min(rect_height);
        self.line_width = (width / min_dim) as f32;

        let rect = Rect {
            pos: dvec2(min_x, min_y),
            size: dvec2(rect_width, rect_height),
        };

        self.draw_abs(cx, rect);
    }
}

/// 3D point/sphere drawing primitive
#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawPoint3D {
    /// Base quad shader state
    #[deref]
    pub draw_super: DrawQuad,

    /// Point color
    #[live]
    pub color: Vec4f,

    /// Data value for colormap
    #[live]
    pub data_value: f32,

    /// Use colormap instead of direct color
    #[live]
    pub use_colormap: f32,

    /// Colormap type
    #[live]
    pub colormap_type: f32,
}

impl DrawPoint3D {
    /// Draw a point at the given center with the given size
    pub fn draw_point(&mut self, cx: &mut Cx2d, center: DVec2, size: f64) {
        let rect = Rect {
            pos: dvec2(center.x - size / 2.0, center.y - size / 2.0),
            size: dvec2(size, size),
        };
        self.draw_abs(cx, rect);
    }

    /// Set point color directly
    pub fn set_color(&mut self, color: Vec4f) {
        self.color = color;
        self.use_colormap = 0.0;
    }

    /// Set point color from data value using colormap
    pub fn set_data_color(&mut self, value: f32, colormap: super::colormap::Colormap) {
        self.data_value = value;
        self.colormap_type = colormap.to_shader_value();
        self.use_colormap = 1.0;
    }
}

/// 3D bar drawing primitive
#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawBar3D {
    /// Base quad shader state
    #[deref]
    pub draw_super: DrawQuad,

    /// Data value for colormap
    #[live]
    pub data_value: f32,

    /// Face type (0=top, 1=front, 2=side)
    #[live]
    pub face_type: f32,

    /// Colormap type
    #[live]
    pub colormap_type: f32,
}

impl DrawBar3D {
    /// Draw a bar face
    pub fn draw_face(&mut self, cx: &mut Cx2d, rect: Rect, face: BarFace, value: f32) {
        self.data_value = value;
        self.face_type = match face {
            BarFace::Top => 0.0,
            BarFace::Front => 1.0,
            BarFace::Side => 2.0,
        };
        self.draw_abs(cx, rect);
    }

    /// Set colormap
    pub fn set_colormap(&mut self, colormap: super::colormap::Colormap) {
        self.colormap_type = colormap.to_shader_value();
    }
}

/// Bar face type
#[derive(Clone, Copy, Debug)]
pub enum BarFace {
    /// Top face (brightest)
    Top,
    /// Front face (medium brightness)
    Front,
    /// Side face (darkest)
    Side,
}

/// 3D grid plane drawing primitive
#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawGrid3D {
    /// Base quad shader state
    #[deref]
    pub draw_super: DrawQuad,

    /// Grid line color
    #[live]
    pub color: Vec4f,

    /// Number of grid divisions
    #[live(10.0)]
    pub grid_divisions: f32,

    /// Grid line width
    #[live(0.02)]
    pub line_width: f32,

    /// Depth for fading
    #[live]
    pub depth: f32,

    /// Opacity
    #[live(0.5)]
    pub opacity: f32,
}

impl DrawGrid3D {
    /// Draw the grid at the given rectangle
    pub fn draw_grid(&mut self, cx: &mut Cx2d, rect: Rect) {
        self.draw_abs(cx, rect);
    }

    /// Set grid parameters
    pub fn set_grid(&mut self, divisions: f32, line_width: f32) {
        self.grid_divisions = divisions;
        self.line_width = line_width;
    }
}

/// 3D axis drawing primitive
#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawAxis3D {
    /// Base quad shader state
    #[deref]
    pub draw_super: DrawQuad,

    /// Axis color
    #[live]
    pub color: Vec4f,
}

impl DrawAxis3D {
    /// Draw an axis at the given rectangle
    pub fn draw_axis(&mut self, cx: &mut Cx2d, rect: Rect, color: Vec4f) {
        self.color = color;
        self.draw_abs(cx, rect);
    }
}

//! Custom drawing primitives for chart rendering
//!
//! Provides GPU-accelerated drawing types for bars, lines, points, and arcs.

use makepad_widgets::*;

live_design! {
    use link::shaders::*;

    // Bar drawing with gradient support
    pub DrawBar = {{DrawBar}} {
        fn pixel(self) -> vec4 {
            let uv = self.pos;

            // Calculate final color with gradient support
            if self.gradient_enabled > 0.5 {
                // Vertical gradient: bottom color at bottom, top color at top
                let t = 1.0 - uv.y; // Invert so gradient goes bottom to top
                let final_color = mix(self.gradient_bottom_color, self.gradient_top_color, t);
                return vec4(final_color.rgb * final_color.a, final_color.a);
            }

            return vec4(self.color.rgb * self.color.a, self.color.a);
        }
    }

    // Point/circle drawing with anti-aliasing
    pub DrawPoint = {{DrawPoint}} {
        fn pixel(self) -> vec4 {
            let uv = self.pos;
            let center = vec2(0.5, 0.5);
            let dist = distance(uv, center) * 2.0; // 0 at center, 1 at edge

            // Check if inside circle
            if dist > 1.0 {
                return vec4(0.0, 0.0, 0.0, 0.0);
            }

            // Anti-alias the edge
            let aa = 0.05;
            let alpha = 1.0 - smoothstep(1.0 - aa, 1.0, dist);

            // Calculate color with gradient support
            if self.gradient_enabled > 0.5 {
                // Radial gradient: center color at center, outer color at edge
                let final_color = mix(self.gradient_center_color, self.gradient_outer_color, dist);
                return vec4(final_color.rgb * final_color.a * alpha, final_color.a * alpha);
            }

            return vec4(self.color.rgb * self.color.a * alpha, self.color.a * alpha);
        }
    }

    // Line segment drawing with anti-aliasing
    pub DrawChartLine = {{DrawChartLine}} {
        fn pixel(self) -> vec4 {
            let uv = self.pos;

            // Line endpoints in normalized coordinates (0-1)
            let p1 = vec2(self.x1, self.y1);
            let p2 = vec2(self.x2, self.y2);

            // Current pixel position
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

            return vec4(self.color.rgb * self.color.a * alpha, self.color.a * alpha);
        }
    }

    // Arc/pie slice drawing
    pub DrawArc = {{DrawArc}} {
        fn pixel(self) -> vec4 {
            let pi_val = 3.14159265;
            let two_pi_val = 6.28318530;

            // Center of the quad is at (0.5, 0.5), map to (-0.5, 0.5)
            let px = self.pos.x - 0.5;
            let py = self.pos.y - 0.5;

            // Calculate distance from center
            let distance = sqrt(px * px + py * py);

            // Radii in normalized space (0.5 = full quad radius)
            let inner_rad = self.inner_radius * 0.5;
            let outer_rad = 0.5;

            // Distance mask: 1.0 if in ring, 0.0 otherwise
            let dist_mask = step(inner_rad, distance) * step(distance, outer_rad);

            // Calculate angle using atan2
            let pixel_ang = atan(py, px);

            // Angle calculation
            let sweep_val = self.end_angle - self.start_angle;
            let rel_ang = pixel_ang - self.start_angle;
            let norm_ang = rel_ang + two_pi_val * 4.0;
            let wrap_ang = mod(norm_ang, two_pi_val);

            // Angle mask: 1.0 if within sweep
            let ang_mask = step(wrap_ang, sweep_val) * step(0.001, sweep_val);

            // Combined mask
            let final_mask = dist_mask * ang_mask;

            // Anti-aliased edges
            let edge_aa = 0.008;
            let outer_aa = 1.0 - smoothstep(outer_rad - edge_aa, outer_rad + edge_aa, distance);
            let inner_aa = smoothstep(inner_rad - edge_aa, inner_rad + edge_aa, distance);
            let aa_alpha = outer_aa * inner_aa;

            let alpha_val = final_mask * aa_alpha;

            // Calculate final color with gradient support
            let final_color = self.color;

            // Gradient mode: 0 = none, 1 = radial (inner to outer), 2 = angular (along arc)
            if self.gradient_enabled > 0.5 {
                if self.gradient_type < 0.5 {
                    // Radial gradient: interpolate from inner to outer radius
                    let ring_width = outer_rad - inner_rad;
                    let t = clamp((distance - inner_rad) / ring_width, 0.0, 1.0);
                    let final_color = mix(self.gradient_inner_color, self.gradient_outer_color, t);
                    return vec4(final_color.rgb * alpha_val, final_color.a * alpha_val);
                } else {
                    // Angular gradient: interpolate along the arc sweep
                    let t = clamp(wrap_ang / sweep_val, 0.0, 1.0);
                    let final_color = mix(self.gradient_inner_color, self.gradient_outer_color, t);
                    return vec4(final_color.rgb * alpha_val, final_color.a * alpha_val);
                }
            }

            return vec4(final_color.rgb * alpha_val, final_color.a * alpha_val);
        }
    }

    // Area fill with gradient
    pub DrawAreaFill = {{DrawAreaFill}} {
        fn pixel(self) -> vec4 {
            let uv = self.pos;

            // Vertical gradient from top to bottom
            let t = uv.y;
            let final_color = mix(self.top_color, self.bottom_color, t);

            return vec4(final_color.rgb * final_color.a, final_color.a);
        }
    }

    // Circle ring (stroke-only circle) with anti-aliasing
    pub DrawCircleRing = {{DrawCircleRing}} {
        fn pixel(self) -> vec4 {
            let uv = self.pos;
            let center = vec2(0.5, 0.5);
            let dist = distance(uv, center) * 2.0; // 0 at center, 1 at edge

            // Ring parameters
            let outer_radius = 1.0;
            let inner_radius = outer_radius - self.stroke_width * 2.0;

            // Anti-alias the edges
            let aa = 0.03;
            let outer_alpha = 1.0 - smoothstep(outer_radius - aa, outer_radius, dist);
            let inner_alpha = smoothstep(inner_radius - aa, inner_radius, dist);
            let alpha = outer_alpha * inner_alpha;

            if alpha < 0.01 {
                return vec4(0.0, 0.0, 0.0, 0.0);
            }

            return vec4(self.color.rgb * self.color.a * alpha, self.color.a * alpha);
        }
    }

    // GPU-accelerated triangle drawing with barycentric coordinates
    pub DrawTriangle = {{DrawTriangle}} {
        fn pixel(self) -> vec4 {
            // Triangle vertices in normalized coordinates (0-1)
            let v0 = vec2(self.v0x, self.v0y);
            let v1 = vec2(self.v1x, self.v1y);
            let v2 = vec2(self.v2x, self.v2y);

            let p = self.pos;

            // Compute barycentric coordinates
            let d00 = dot(v1 - v0, v1 - v0);
            let d01 = dot(v1 - v0, v2 - v0);
            let d11 = dot(v2 - v0, v2 - v0);
            let d20 = dot(p - v0, v1 - v0);
            let d21 = dot(p - v0, v2 - v0);

            let denom = d00 * d11 - d01 * d01;
            if abs(denom) < 0.0001 {
                return vec4(0.0, 0.0, 0.0, 0.0);
            }

            let inv_denom = 1.0 / denom;
            let u = (d11 * d20 - d01 * d21) * inv_denom;
            let v = (d00 * d21 - d01 * d20) * inv_denom;

            // Check if point is inside triangle
            if u >= 0.0 && v >= 0.0 && (u + v) <= 1.0 {
                // Calculate final color with gradient support
                if self.gradient_enabled > 0.5 {
                    if self.gradient_type < 0.5 {
                        // Radial gradient: v0 is center, v1/v2 are edges
                        // Barycentric weight at v0 is (1 - u - v)
                        let center_weight = 1.0 - u - v;
                        let final_color = mix(self.gradient_outer_color, self.gradient_center_color, center_weight);
                        return vec4(final_color.rgb * final_color.a, final_color.a);
                    } else {
                        // Vertical gradient: top to bottom based on Y position
                        let t = p.y; // UV y position (0 = top, 1 = bottom)
                        let final_color = mix(self.gradient_center_color, self.gradient_outer_color, t);
                        return vec4(final_color.rgb * final_color.a, final_color.a);
                    }
                }
                return vec4(self.color.rgb * self.color.a, self.color.a);
            }

            return vec4(0.0, 0.0, 0.0, 0.0);
        }
    }
}

#[derive(Live, LiveHook, LiveRegister)]
#[repr(C)]
pub struct DrawBar {
    #[deref] pub draw_super: DrawQuad,
    #[live] pub color: Vec4,
    #[live(0.0)] pub gradient_enabled: f32,
    #[live] pub gradient_bottom_color: Vec4,
    #[live] pub gradient_top_color: Vec4,
}

impl DrawBar {
    pub fn draw_bar(&mut self, cx: &mut Cx2d, rect: Rect) {
        self.draw_abs(cx, rect);
    }

    pub fn set_solid_color(&mut self, color: Vec4) {
        self.gradient_enabled = 0.0;
        self.color = color;
    }

    pub fn set_vertical_gradient(&mut self, bottom_color: Vec4, top_color: Vec4) {
        self.gradient_enabled = 1.0;
        self.gradient_bottom_color = bottom_color;
        self.gradient_top_color = top_color;
    }

    pub fn disable_gradient(&mut self) {
        self.gradient_enabled = 0.0;
    }
}

#[derive(Live, LiveHook, LiveRegister)]
#[repr(C)]
pub struct DrawPoint {
    #[deref] pub draw_super: DrawQuad,
    #[live] pub color: Vec4,
    #[live(0.0)] pub gradient_enabled: f32,
    #[live] pub gradient_center_color: Vec4,
    #[live] pub gradient_outer_color: Vec4,
}

impl DrawPoint {
    pub fn draw_point(&mut self, cx: &mut Cx2d, center: DVec2, size: f64) {
        let rect = Rect {
            pos: dvec2(center.x - size / 2.0, center.y - size / 2.0),
            size: dvec2(size, size),
        };
        self.draw_abs(cx, rect);
    }

    pub fn set_radial_gradient(&mut self, center_color: Vec4, outer_color: Vec4) {
        self.gradient_enabled = 1.0;
        self.gradient_center_color = center_color;
        self.gradient_outer_color = outer_color;
    }

    pub fn disable_gradient(&mut self) {
        self.gradient_enabled = 0.0;
    }
}

#[derive(Live, LiveHook, LiveRegister)]
#[repr(C)]
pub struct DrawChartLine {
    #[deref] pub draw_super: DrawQuad,
    #[live] pub color: Vec4,
    #[live] pub x1: f32,
    #[live] pub y1: f32,
    #[live] pub x2: f32,
    #[live] pub y2: f32,
    #[live] pub line_width: f32,
}

impl DrawChartLine {
    pub fn draw_line(&mut self, cx: &mut Cx2d, p1: DVec2, p2: DVec2, width: f64) {
        // Calculate bounding box with padding for line width
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

        // Convert absolute positions to normalized (0-1) within bounding box
        self.x1 = ((p1.x - min_x) / rect_width) as f32;
        self.y1 = ((p1.y - min_y) / rect_height) as f32;
        self.x2 = ((p2.x - min_x) / rect_width) as f32;
        self.y2 = ((p2.y - min_y) / rect_height) as f32;

        // Line width relative to smaller dimension
        let min_dim = rect_width.min(rect_height);
        self.line_width = (width / min_dim) as f32;

        let rect = Rect {
            pos: dvec2(min_x, min_y),
            size: dvec2(rect_width, rect_height),
        };

        self.draw_abs(cx, rect);
    }
}

#[derive(Live, LiveHook, LiveRegister)]
#[repr(C)]
pub struct DrawArc {
    #[deref] pub draw_super: DrawQuad,
    #[live] pub color: Vec4,
    #[live] pub start_angle: f32,
    #[live] pub end_angle: f32,
    #[live(0.0)] pub inner_radius: f32,
    #[live(1.0)] pub outer_radius: f32,
    #[live(0.0)] pub gradient_enabled: f32,
    #[live(0.0)] pub gradient_type: f32,
    #[live] pub gradient_inner_color: Vec4,
    #[live] pub gradient_outer_color: Vec4,
}

impl DrawArc {
    pub fn set_arc(&mut self, start: f64, sweep: f64, inner: f64, outer: f64) {
        self.start_angle = start as f32;
        self.end_angle = (start + sweep) as f32;
        if outer > 0.0 {
            self.inner_radius = (inner / outer) as f32;
        } else {
            self.inner_radius = 0.0;
        }
        self.outer_radius = 1.0;
    }

    pub fn draw_arc(&mut self, cx: &mut Cx2d, center: DVec2, outer_radius: f64) {
        let rect = Rect {
            pos: dvec2(center.x - outer_radius, center.y - outer_radius),
            size: dvec2(outer_radius * 2.0, outer_radius * 2.0),
        };
        self.draw_abs(cx, rect);
    }

    pub fn set_radial_gradient(&mut self, inner_color: Vec4, outer_color: Vec4) {
        self.gradient_enabled = 1.0;
        self.gradient_type = 0.0;
        self.gradient_inner_color = inner_color;
        self.gradient_outer_color = outer_color;
    }

    pub fn disable_gradient(&mut self) {
        self.gradient_enabled = 0.0;
    }

    pub fn set_solid_color(&mut self, color: Vec4) {
        self.color = color;
        self.gradient_enabled = 0.0;
    }
}

#[derive(Live, LiveHook, LiveRegister)]
#[repr(C)]
pub struct DrawAreaFill {
    #[deref] pub draw_super: DrawQuad,
    #[live] pub top_color: Vec4,
    #[live] pub bottom_color: Vec4,
}

impl DrawAreaFill {
    pub fn draw_area(&mut self, cx: &mut Cx2d, rect: Rect) {
        self.draw_abs(cx, rect);
    }
}

#[derive(Live, LiveHook, LiveRegister)]
#[repr(C)]
pub struct DrawTriangle {
    #[deref] pub draw_super: DrawQuad,
    #[live] pub color: Vec4,
    #[live] pub v0x: f32,
    #[live] pub v0y: f32,
    #[live] pub v1x: f32,
    #[live] pub v1y: f32,
    #[live] pub v2x: f32,
    #[live] pub v2y: f32,
    /// Gradient enabled (0.0 = no, 1.0 = yes)
    #[live(0.0)] pub gradient_enabled: f32,
    /// Gradient type (0.0 = radial, 1.0 = vertical)
    #[live(0.0)] pub gradient_type: f32,
    /// Center/top color for gradient
    #[live] pub gradient_center_color: Vec4,
    /// Outer/bottom color for gradient
    #[live] pub gradient_outer_color: Vec4,
}

impl DrawTriangle {
    pub fn draw_triangle(&mut self, cx: &mut Cx2d, p0: DVec2, p1: DVec2, p2: DVec2) {
        // Calculate bounding box
        let min_x = p0.x.min(p1.x).min(p2.x);
        let max_x = p0.x.max(p1.x).max(p2.x);
        let min_y = p0.y.min(p1.y).min(p2.y);
        let max_y = p0.y.max(p1.y).max(p2.y);

        let width = max_x - min_x;
        let height = max_y - min_y;

        if width < 1.0 || height < 1.0 {
            return;
        }

        // Convert to normalized coordinates within bounding box
        self.v0x = ((p0.x - min_x) / width) as f32;
        self.v0y = ((p0.y - min_y) / height) as f32;
        self.v1x = ((p1.x - min_x) / width) as f32;
        self.v1y = ((p1.y - min_y) / height) as f32;
        self.v2x = ((p2.x - min_x) / width) as f32;
        self.v2y = ((p2.y - min_y) / height) as f32;

        let rect = Rect {
            pos: dvec2(min_x, min_y),
            size: dvec2(width, height),
        };

        self.draw_abs(cx, rect);
    }

    /// Enable radial gradient (center color at v0, outer color at v1/v2)
    pub fn set_radial_gradient(&mut self, center_color: Vec4, outer_color: Vec4) {
        self.gradient_enabled = 1.0;
        self.gradient_type = 0.0;
        self.gradient_center_color = center_color;
        self.gradient_outer_color = outer_color;
    }

    /// Enable vertical gradient (top to bottom)
    pub fn set_vertical_gradient(&mut self, top_color: Vec4, bottom_color: Vec4) {
        self.gradient_enabled = 1.0;
        self.gradient_type = 1.0;
        self.gradient_center_color = top_color;
        self.gradient_outer_color = bottom_color;
    }

    /// Disable gradient (use solid color)
    pub fn disable_gradient(&mut self) {
        self.gradient_enabled = 0.0;
    }
}

/// Circle ring (stroke-only circle) drawing primitive
#[derive(Live, LiveHook, LiveRegister)]
#[repr(C)]
pub struct DrawCircleRing {
    #[deref] pub draw_super: DrawQuad,
    #[live] pub color: Vec4,
    #[live(0.05)] pub stroke_width: f32,
}

impl DrawCircleRing {
    /// Draw a circle ring at the given center with the given radius
    pub fn draw_ring(&mut self, cx: &mut Cx2d, center: DVec2, radius: f64, stroke_width: f64) {
        let size = radius + stroke_width;
        self.stroke_width = (stroke_width / size) as f32 * 0.5;
        let rect = Rect {
            pos: dvec2(center.x - size, center.y - size),
            size: dvec2(size * 2.0, size * 2.0),
        };
        self.draw_abs(cx, rect);
    }
}

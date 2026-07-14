//! Color Interpolation Comparison Chart
//!
//! Demonstrates the difference between RGB, HSL, Lab, and Oklab color interpolation.

use makepad_widgets::*;
use makepad_d3::color::{
    Rgba, interpolate_rgb, interpolate_hsl, interpolate_lab,
    interpolate_oklab, interpolate_oklch,
};

live_design! {
    link widgets;
    use link::shaders::*;

    pub DrawColorBar = {{DrawColorBar}} {
        fn pixel(self) -> vec4 {
            // Return color based on position - will be set per-segment
            return self.color;
        }
    }

    pub ColorInterpolationWidget = {{ColorInterpolationWidget}} {
        width: Fill,
        height: Fill,
    }
}

#[derive(Live, LiveHook, LiveRegister)]
#[repr(C)]
pub struct DrawColorBar {
    #[deref] pub draw_super: DrawQuad,
    #[live] pub color: Vec4,
}

#[derive(Live, LiveHook, Widget)]
pub struct ColorInterpolationWidget {
    #[redraw]
    #[live]
    draw_bar: DrawColorBar,

    #[walk]
    walk: Walk,

    #[rust]
    area: Area,
}

impl Widget for ColorInterpolationWidget {
    fn handle_event(&mut self, _cx: &mut Cx, _event: &Event, _scope: &mut Scope) {}

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let rect = cx.walk_turtle_with_area(&mut self.area, walk);

        if rect.size.x > 0.0 && rect.size.y > 0.0 {
            self.draw_comparison(cx, rect);
        }

        DrawStep::done()
    }
}

impl ColorInterpolationWidget {
    fn draw_comparison(&mut self, cx: &mut Cx2d, rect: Rect) {
        let padding = 20.0;
        let label_width = 80.0;
        let bar_height = 40.0;
        let spacing = 15.0;

        let bar_width = rect.size.x - padding * 2.0 - label_width;
        let start_x = rect.pos.x + padding + label_width;
        let mut y = rect.pos.y + padding + 30.0;

        // Color pairs to compare
        let color_pairs = [
            (Rgba::RED, Rgba::BLUE, "Red → Blue"),
            (Rgba::rgb(0.0, 0.8, 0.2), Rgba::rgb(0.8, 0.0, 0.8), "Green → Purple"),
            (Rgba::rgb(1.0, 1.0, 0.0), Rgba::rgb(0.0, 0.5, 1.0), "Yellow → Cyan"),
        ];

        for (color_a, color_b, _label) in &color_pairs {
            // Draw 5 interpolation methods for each pair
            let methods: [(&str, fn(&Rgba, &Rgba, f32) -> Rgba); 5] = [
                ("RGB", |a, b, t| interpolate_rgb(a, b, t as f64)),
                ("HSL", |a, b, t| interpolate_hsl(a, b, t as f64)),
                ("Lab", |a, b, t| interpolate_lab(a, b, t as f64)),
                ("Oklab", |a, b, t| interpolate_oklab(a, b, t)),
                ("Oklch", |a, b, t| interpolate_oklch(a, b, t)),
            ];

            for (method_name, interp_fn) in &methods {
                self.draw_gradient_bar(cx, start_x, y, bar_width, bar_height, color_a, color_b, *interp_fn);
                y += bar_height + spacing;
            }

            y += spacing * 2.0; // Extra space between color pairs
        }
    }

    fn draw_gradient_bar(
        &mut self,
        cx: &mut Cx2d,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        color_a: &Rgba,
        color_b: &Rgba,
        interp_fn: fn(&Rgba, &Rgba, f32) -> Rgba,
    ) {
        let segments = 64;
        let segment_width = width / segments as f64;

        for i in 0..segments {
            let t = i as f32 / (segments - 1) as f32;
            let color = interp_fn(color_a, color_b, t);

            self.draw_bar.color = vec4(color.r, color.g, color.b, color.a);
            self.draw_bar.draw_abs(
                cx,
                Rect {
                    pos: dvec2(x + i as f64 * segment_width, y),
                    size: dvec2(segment_width + 1.0, height), // +1 to avoid gaps
                },
            );
        }
    }
}

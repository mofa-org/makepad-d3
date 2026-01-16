//! Apollonius' Problem Widget
//!
//! Interactive visualization of Apollonius' Problem: finding circles tangent to three given circles.
//! Features draggable input circles and real-time computation of tangent circles.

use makepad_widgets::*;
use super::draw_primitives::{DrawPoint, DrawCircleRing};

live_design! {
    link widgets;
    use link::shaders::*;
    use super::draw_primitives::DrawPoint;
    use super::draw_primitives::DrawCircleRing;

    pub ApolloniusProblemWidget = {{ApolloniusProblemWidget}} {
        width: Fill,
        height: Fill,
    }
}

/// Circle with position and radius
#[derive(Clone, Copy, Default)]
struct Circle {
    x: f64,
    y: f64,
    r: f64,
}

impl Circle {
    fn new(x: f64, y: f64, r: f64) -> Self {
        Self { x, y, r }
    }
}

/// Colors from D3's schemeCategory10
const CATEGORY10_COLORS: [Vec4; 10] = [
    Vec4 { x: 0.122, y: 0.467, z: 0.706, w: 1.0 }, // #1f77b4
    Vec4 { x: 1.0, y: 0.498, z: 0.055, w: 1.0 },   // #ff7f0e
    Vec4 { x: 0.173, y: 0.627, z: 0.173, w: 1.0 }, // #2ca02c
    Vec4 { x: 0.839, y: 0.153, z: 0.157, w: 1.0 }, // #d62728
    Vec4 { x: 0.580, y: 0.404, z: 0.741, w: 1.0 }, // #9467bd
    Vec4 { x: 0.549, y: 0.337, z: 0.294, w: 1.0 }, // #8c564b
    Vec4 { x: 0.890, y: 0.467, z: 0.761, w: 1.0 }, // #e377c2
    Vec4 { x: 0.498, y: 0.498, z: 0.498, w: 1.0 }, // #7f7f7f
    Vec4 { x: 0.737, y: 0.741, z: 0.133, w: 1.0 }, // #bcbd22
    Vec4 { x: 0.090, y: 0.745, z: 0.812, w: 1.0 }, // #17becf
];

#[derive(Live, LiveHook, Widget)]
pub struct ApolloniusProblemWidget {
    #[redraw]
    #[live]
    draw_circle: DrawPoint,

    #[redraw]
    #[live]
    draw_ring: DrawCircleRing,

    #[walk]
    walk: Walk,

    #[rust]
    area: Area,

    #[rust]
    chart_rect: Rect,

    #[rust]
    initialized: bool,

    /// The three input circles
    #[rust]
    circles: [Circle; 3],

    /// Show all 8 solutions (true) or just the internally tangent one (false)
    #[rust(false)]
    show_all_solutions: bool,

    /// Currently dragged circle index (None if not dragging)
    #[rust]
    dragging: Option<usize>,

    /// Offset from circle center when drag started
    #[rust]
    drag_offset: DVec2,
}

impl Widget for ApolloniusProblemWidget {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        match event {
            Event::MouseDown(e) => {
                self.handle_mouse_down(cx, e.abs);
            }
            Event::MouseMove(e) => {
                if self.dragging.is_some() {
                    self.handle_drag(cx, e.abs);
                }
            }
            Event::MouseUp(_) => {
                self.dragging = None;
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let rect = cx.walk_turtle_with_area(&mut self.area, walk);

        if rect.size.x > 0.0 && rect.size.y > 0.0 {
            self.chart_rect = rect;

            if !self.initialized {
                self.initialize();
            }

            self.draw_visualization(cx);
        }

        DrawStep::done()
    }
}

impl ApolloniusProblemWidget {
    fn initialize(&mut self) {
        // D3 default positions
        self.circles = [
            Circle::new(180.0, 250.0, 80.0),
            Circle::new(400.0, 100.0, 20.0),
            Circle::new(400.0, 300.0, 120.0),
        ];
        self.initialized = true;
    }

    fn handle_mouse_down(&mut self, cx: &mut Cx, pos: DVec2) {
        let rect = self.chart_rect;

        // Check if click is on any circle (check in reverse order for z-order)
        for i in (0..3).rev() {
            let c = &self.circles[i];
            let cx_pos = rect.pos.x + c.x;
            let cy_pos = rect.pos.y + c.y;

            let dx = pos.x - cx_pos;
            let dy = pos.y - cy_pos;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist <= c.r {
                self.dragging = Some(i);
                self.drag_offset = dvec2(dx, dy);
                self.redraw(cx);
                return;
            }
        }
    }

    fn handle_drag(&mut self, cx: &mut Cx, pos: DVec2) {
        if let Some(i) = self.dragging {
            let rect = self.chart_rect;
            let r = self.circles[i].r;

            // Calculate new position, clamped to stay within bounds
            let new_x = (pos.x - rect.pos.x - self.drag_offset.x)
                .max(r)
                .min(rect.size.x - r);
            let new_y = (pos.y - rect.pos.y - self.drag_offset.y)
                .max(r)
                .min(rect.size.y - r);

            self.circles[i].x = new_x;
            self.circles[i].y = new_y;

            self.redraw(cx);
        }
    }

    fn draw_visualization(&mut self, cx: &mut Cx2d) {
        let rect = self.chart_rect;

        // Draw Apollonius circles first (behind input circles)
        if self.show_all_solutions {
            self.draw_all_solutions(cx, rect);
        } else {
            self.draw_single_solution(cx, rect);
        }

        // Draw the three input circles
        for c in &self.circles {
            let center = dvec2(rect.pos.x + c.x, rect.pos.y + c.y);

            // Semi-transparent fill (fill-opacity: 0.5 in D3)
            self.draw_circle.color = vec4(0.0, 0.0, 0.0, 0.5);
            self.draw_circle.gradient_enabled = 0.0;
            self.draw_circle.draw_point(cx, center, c.r * 2.0);
        }
    }

    fn draw_single_solution(&mut self, cx: &mut Cx2d, rect: Rect) {
        let c1 = self.circles[0];
        let c2 = self.circles[1];
        let c3 = self.circles[2];

        // Compute the internally tangent circle (all positive radii)
        let result = apollonius_circle(
            c1.x, c1.y, c1.r,
            c2.x, c2.y, c2.r,
            c3.x, c3.y, c3.r,
        );

        if result.r > 0.0 && !result.r.is_nan() {
            let center = dvec2(rect.pos.x + result.x, rect.pos.y + result.y);

            // Draw outer ring
            self.draw_ring.color = vec4(0.0, 0.0, 0.0, 1.0);
            self.draw_ring.draw_ring(cx, center, result.r, 1.5);

            // Draw inner ring (slightly smaller, semi-transparent)
            self.draw_ring.color = vec4(0.0, 0.0, 0.0, 0.25);
            self.draw_ring.draw_ring(cx, center, result.r - 3.0, 5.0);
        }
    }

    fn draw_all_solutions(&mut self, cx: &mut Cx2d, rect: Rect) {
        let c1 = self.circles[0];
        let c2 = self.circles[1];
        let c3 = self.circles[2];

        // All 8 combinations of +/- radii
        let sign_combinations: [(f64, f64, f64); 8] = [
            (1.0, 1.0, 1.0),
            (1.0, 1.0, -1.0),
            (1.0, -1.0, 1.0),
            (1.0, -1.0, -1.0),
            (-1.0, 1.0, 1.0),
            (-1.0, 1.0, -1.0),
            (-1.0, -1.0, 1.0),
            (-1.0, -1.0, -1.0),
        ];

        for (i, (s1, s2, s3)) in sign_combinations.iter().enumerate() {
            let result = apollonius_circle(
                c1.x, c1.y, c1.r * s1,
                c2.x, c2.y, c2.r * s2,
                c3.x, c3.y, c3.r * s3,
            );

            if !result.r.is_nan() {
                let r = result.r.abs();
                let center = dvec2(rect.pos.x + result.x, rect.pos.y + result.y);

                let color = CATEGORY10_COLORS[i % 10];

                // Draw outer ring
                self.draw_ring.color = color;
                self.draw_ring.draw_ring(cx, center, r, 1.5);

                // Draw inner ring (slightly smaller, semi-transparent)
                self.draw_ring.color = vec4(color.x, color.y, color.z, 0.25);
                self.draw_ring.draw_ring(cx, center, r - 3.0, 5.0);
            }
        }
    }

    /// Initialize with single solution mode (default)
    pub fn initialize_single_solution(&mut self) {
        self.show_all_solutions = false;
        self.initialized = false;
    }

    /// Initialize with all 8 solutions visible
    pub fn initialize_all_solutions(&mut self) {
        self.show_all_solutions = true;
        self.initialized = false;
    }
}

/// Apollonius circle computation
/// Computes the circle tangent to three given circles.
/// Use negative radius to choose different tangent circles.
fn apollonius_circle(
    x1: f64, y1: f64, r1: f64,
    x2: f64, y2: f64, r2: f64,
    x3: f64, y3: f64, r3: f64,
) -> Circle {
    // The quadratic equation:
    //   0 = (x - x1)² + (y - y1)² - (r ± r1)²
    //   0 = (x - x2)² + (y - y2)² - (r ± r2)²
    //   0 = (x - x3)² + (y - y3)² - (r ± r3)²
    //
    // Per mathworld.wolfram.com/ApolloniusProblem.html

    let a2 = 2.0 * (x1 - x2);
    let b2 = 2.0 * (y1 - y2);
    let c2 = 2.0 * (r2 - r1);
    let d2 = x1 * x1 + y1 * y1 - r1 * r1 - x2 * x2 - y2 * y2 + r2 * r2;

    let a3 = 2.0 * (x1 - x3);
    let b3 = 2.0 * (y1 - y3);
    let c3 = 2.0 * (r3 - r1);
    let d3 = x1 * x1 + y1 * y1 - r1 * r1 - x3 * x3 - y3 * y3 + r3 * r3;

    let ab = a3 * b2 - a2 * b3;

    // Check for degenerate case
    if ab.abs() < 1e-10 {
        return Circle::new(0.0, 0.0, f64::NAN);
    }

    let xa = (b2 * d3 - b3 * d2) / ab - x1;
    let xb = (b3 * c2 - b2 * c3) / ab;

    let ya = (a3 * d2 - a2 * d3) / ab - y1;
    let yb = (a2 * c3 - a3 * c2) / ab;

    // Quadratic formula coefficients
    let a = xb * xb + yb * yb - 1.0;
    let b = 2.0 * (xa * xb + ya * yb + r1);
    let c = xa * xa + ya * ya - r1 * r1;

    // Solve quadratic
    let r = if a.abs() > 1e-10 {
        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0 {
            return Circle::new(0.0, 0.0, f64::NAN);
        }
        (-b - discriminant.sqrt()) / (2.0 * a)
    } else if b.abs() > 1e-10 {
        -c / b
    } else {
        return Circle::new(0.0, 0.0, f64::NAN);
    };

    let x = xa + xb * r + x1;
    let y = ya + yb * r + y1;

    Circle::new(x, y, r)
}

/// Widget reference implementation for external initialization
impl ApolloniusProblemWidgetRef {
    pub fn initialize_single_solution(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.initialize_single_solution();
            inner.redraw(cx);
        }
    }

    pub fn initialize_all_solutions(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.initialize_all_solutions();
            inner.redraw(cx);
        }
    }
}

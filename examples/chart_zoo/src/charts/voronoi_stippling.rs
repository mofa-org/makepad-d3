//! Voronoi Stippling Widget
//!
//! CPU-based weighted Lloyd's algorithm over a grayscale image,
//! matching the Observable Voronoi stippling example.

use makepad_widgets::*;
use makepad_widgets::image_cache::ImageBuffer;
use super::draw_primitives::DrawPoint;

const OBAMA_PNG: &[u8] = include_bytes!("../../resources/obama.png");
const DEFAULT_MAX_SAMPLE_PIXELS: usize = 50_000;  // Higher = more resolution = more potential points
const DEFAULT_POINTS_DIVISOR: f64 = 20.0;
const MAX_ITERATIONS: usize = 80;
const ITERS_PER_FRAME: usize = 1;
const DEFAULT_POINT_RADIUS: f64 = 1.2;

#[derive(Clone, Copy, Debug, Default)]
struct StipplePoint {
    x: f64,
    y: f64,
}

live_design! {
    link widgets;
    use link::shaders::*;
    use super::draw_primitives::DrawPoint;

    pub VoronoiStipplingWidget = {{VoronoiStipplingWidget}} {
        width: Fill,
        height: Fill,
        draw_circle: { color: #000000 }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct VoronoiStipplingWidget {
    #[redraw]
    #[live]
    draw_circle: DrawPoint,

    #[walk]
    walk: Walk,

    #[rust]
    area: Area,
    #[rust]
    initialized: bool,
    #[rust]
    running: bool,
    #[rust]
    iteration: usize,
    #[rust]
    rng_seed: u64,

    #[rust]
    data: Vec<f64>,
    #[rust]
    data_width: usize,
    #[rust]
    data_height: usize,

    #[rust]
    points: Vec<StipplePoint>,
    #[rust]
    centroid_sums: Vec<f64>,
    #[rust]
    centroid_coords: Vec<f64>,

    #[rust]
    max_sample_pixels: usize,
    #[rust]
    points_divisor: f64,
    #[rust]
    point_radius: f64,
}

impl Widget for VoronoiStipplingWidget {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        if let Event::NextFrame(_) = event {
            if self.running {
                self.step_relaxation();
                self.redraw(cx);
                if self.running {
                    cx.new_next_frame();
                }
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let rect = cx.walk_turtle_with_area(&mut self.area, walk);

        if rect.size.x <= 0.0 || rect.size.y <= 0.0 {
            return DrawStep::done();
        }

        if !self.initialized {
            self.initialize(cx);
        }

        if self.points.is_empty() || self.data_width == 0 || self.data_height == 0 {
            return DrawStep::done();
        }

        self.draw_circle.gradient_enabled = 0.0;
        self.draw_circle.color = vec4(0.0, 0.0, 0.0, 1.0);

        let data_w = self.data_width as f64;
        let data_h = self.data_height as f64;
        let scale = (rect.size.x / data_w).min(rect.size.y / data_h);
        let offset_x = rect.pos.x + (rect.size.x - data_w * scale) * 0.5;
        let offset_y = rect.pos.y + (rect.size.y - data_h * scale) * 0.5;
        let point_size = (self.point_radius * 2.0 * scale).max(0.5);

        for point in &self.points {
            let x = offset_x + point.x * scale;
            let y = offset_y + point.y * scale;
            self.draw_circle.draw_point(cx, dvec2(x, y), point_size);
        }

        DrawStep::done()
    }
}

impl VoronoiStipplingWidget {
    fn initialize(&mut self, cx: &mut Cx2d) {
        self.ensure_defaults();
        if self.load_density_data().is_err() {
            self.initialized = true;
            self.running = false;
            return;
        }

        self.reset_simulation();
        self.running = true;
        self.initialized = true;
        cx.cx.new_next_frame();
    }

    fn load_density_data(&mut self) -> Result<(), ()> {
        self.ensure_defaults();
        let image = match ImageBuffer::from_png(OBAMA_PNG) {
            Ok(image) => image,
            Err(_) => {
                return Err(());
            }
        };

        if image.width == 0 || image.height == 0 {
            return Err(());
        }

        let (target_w, target_h) = self.scaled_dimensions(image.width, image.height);
        let mut data = vec![0.0; target_w * target_h];

        let scale_x = image.width as f64 / target_w as f64;
        let scale_y = image.height as f64 / target_h as f64;

        for y in 0..target_h {
            let src_y = ((y as f64 + 0.5) * scale_y).floor().min(image.height as f64 - 1.0) as usize;
            for x in 0..target_w {
                let src_x = ((x as f64 + 0.5) * scale_x).floor().min(image.width as f64 - 1.0) as usize;
                let argb = image.data[src_y * image.width + src_x];
                let r = ((argb >> 16) & 0xff) as f64;
                let density = (1.0 - r / 254.0).clamp(0.0, 1.0);
                data[y * target_w + x] = density;
            }
        }

        self.data = data;
        self.data_width = target_w;
        self.data_height = target_h;
        Ok(())
    }

    fn scaled_dimensions(&self, src_w: usize, src_h: usize) -> (usize, usize) {
        let pixel_count = (src_w * src_h) as f64;
        let scale = (self.max_sample_pixels as f64 / pixel_count).sqrt().min(1.0);
        let target_w = ((src_w as f64 * scale).round() as usize).max(1);
        let target_h = ((src_h as f64 * scale).round() as usize).max(1);
        (target_w, target_h)
    }

    fn reset_simulation(&mut self) {
        let total_points = ((self.data_width * self.data_height) as f64 / self.points_divisor).round() as usize;
        let total_points = total_points.max(1);

        self.points = vec![StipplePoint::default(); total_points];
        self.centroid_sums = vec![0.0; total_points];
        self.centroid_coords = vec![0.0; total_points * 2];
        self.iteration = 0;
        self.rng_seed = 12345;

        self.initialize_points();
    }

    fn initialize_points(&mut self) {
        let w = self.data_width as f64;
        let h = self.data_height as f64;

        if w == 0.0 || h == 0.0 {
            return;
        }

        for i in 0..self.points.len() {
            let mut x = 0.0;
            let mut y = 0.0;
            for _ in 0..30 {
                x = (self.rand_f64() * w).floor();
                y = (self.rand_f64() * h).floor();
                let idx = y as usize * self.data_width + x as usize;
                if idx < self.data.len() && self.rand_f64() < self.data[idx] {
                    break;
                }
            }
            self.points[i] = StipplePoint { x, y };
        }
    }

    fn step_relaxation(&mut self) {
        let remaining = MAX_ITERATIONS.saturating_sub(self.iteration);
        if remaining == 0 {
            self.running = false;
            return;
        }

        let iterations = remaining.min(ITERS_PER_FRAME);
        for _ in 0..iterations {
            self.single_iteration();
        }

        if self.iteration >= MAX_ITERATIONS {
            self.running = false;
        }
    }

    fn ensure_defaults(&mut self) {
        if self.max_sample_pixels == 0 {
            self.max_sample_pixels = DEFAULT_MAX_SAMPLE_PIXELS;
        }
        if self.points_divisor <= 0.0 {
            self.points_divisor = DEFAULT_POINTS_DIVISOR;
        }
        if self.point_radius <= 0.0 {
            self.point_radius = DEFAULT_POINT_RADIUS;
        }
    }

    fn set_points_divisor(&mut self, cx: &mut Cx, divisor: f64) {
        self.ensure_defaults();
        let divisor = divisor.max(1.0);
        if (divisor - self.points_divisor).abs() < 0.01 {
            return;
        }
        self.points_divisor = divisor;
        self.restart(cx);
    }

    fn set_point_radius(&mut self, cx: &mut Cx, radius: f64) {
        self.ensure_defaults();
        let radius = radius.max(0.2);
        if (radius - self.point_radius).abs() < 0.001 {
            return;
        }
        self.point_radius = radius;
        self.redraw(cx);
    }

    fn single_iteration(&mut self) {
        let n = self.points.len();
        if n == 0 {
            return;
        }

        for value in &mut self.centroid_sums {
            *value = 0.0;
        }
        for value in &mut self.centroid_coords {
            *value = 0.0;
        }

        let mut last = 0usize;
        let w = self.data_width;
        let h = self.data_height;

        for y in 0..h {
            let row = y * w;
            for x in 0..w {
                let weight = self.data[row + x];
                if weight <= 0.0 {
                    continue;
                }
                let idx = self.find_closest(x as f64 + 0.5, y as f64 + 0.5, last);
                last = idx;
                self.centroid_sums[idx] += weight;
                let base = idx * 2;
                self.centroid_coords[base] += weight * (x as f64 + 0.5);
                self.centroid_coords[base + 1] += weight * (y as f64 + 0.5);
            }
        }

        let wigg = ((self.iteration + 1) as f64).powf(-0.8) * 10.0;
        let max_x = (self.data_width as f64 - 1.0).max(0.0);
        let max_y = (self.data_height as f64 - 1.0).max(0.0);

        for i in 0..n {
            let x0 = self.points[i].x;
            let y0 = self.points[i].y;
            let sum = self.centroid_sums[i];
            let base = i * 2;
            let x1 = if sum > 0.0 { self.centroid_coords[base] / sum } else { x0 };
            let y1 = if sum > 0.0 { self.centroid_coords[base + 1] / sum } else { y0 };

            let jitter_x = (self.rand_f64() - 0.5) * wigg;
            let jitter_y = (self.rand_f64() - 0.5) * wigg;
            let nx = x0 + (x1 - x0) * 1.8 + jitter_x;
            let ny = y0 + (y1 - y0) * 1.8 + jitter_y;
            self.points[i].x = nx.clamp(0.0, max_x);
            self.points[i].y = ny.clamp(0.0, max_y);
        }

        self.iteration += 1;
    }

    fn find_closest(&self, x: f64, y: f64, start: usize) -> usize {
        if self.points.is_empty() {
            return 0;
        }

        let mut best = start.min(self.points.len() - 1);
        let mut best_dist = {
            let p = self.points[best];
            let dx = p.x - x;
            let dy = p.y - y;
            dx * dx + dy * dy
        };

        for (i, p) in self.points.iter().enumerate() {
            let dx = p.x - x;
            let dy = p.y - y;
            let dist = dx * dx + dy * dy;
            if dist < best_dist {
                best_dist = dist;
                best = i;
            }
        }

        best
    }

    fn rand_f64(&mut self) -> f64 {
        self.rng_seed = self.rng_seed.wrapping_mul(1103515245).wrapping_add(12345);
        ((self.rng_seed >> 16) & 0x7fff) as f64 / 32768.0
    }

    fn restart(&mut self, cx: &mut Cx) {
        self.ensure_defaults();
        if self.data.is_empty() {
            if self.load_density_data().is_err() {
                return;
            }
        }
        self.reset_simulation();
        self.running = true;
        cx.new_next_frame();
        self.redraw(cx);
    }
}

impl VoronoiStipplingWidgetRef {
    pub fn replay_animation(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.restart(cx);
        }
    }

    pub fn set_points_divisor(&self, cx: &mut Cx, divisor: f64) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_points_divisor(cx, divisor);
        }
    }

    pub fn set_point_radius(&self, cx: &mut Cx, radius: f64) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_point_radius(cx, radius);
        }
    }
}

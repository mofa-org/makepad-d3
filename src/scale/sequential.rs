//! Sequential scale implementation
//!
//! Sequential scales map a continuous domain to a continuous range using
//! an interpolator function. They are commonly used for color gradients
//! and heat maps.

use super::traits::{Scale, Tick, TickOptions};

/// An interpolator function that maps t ∈ [0, 1] to an output value
pub type Interpolator<T> = Box<dyn Fn(f64) -> T + Send + Sync>;

/// Scale that maps continuous input through an interpolator function
///
/// Sequential scales are designed for mapping continuous data to colors
/// or other interpolated values. The domain is mapped to [0, 1], which
/// is then passed to an interpolator function.
///
/// # D3.js Equivalent
/// This is equivalent to `d3.scaleSequential()` in D3.js.
///
/// # Example
/// ```
/// use makepad_d3::scale::SequentialScale;
///
/// // Create a grayscale interpolator
/// let scale = SequentialScale::new(|t| format!("rgb({0},{0},{0})", (t * 255.0) as u8))
///     .domain(0.0, 100.0);
///
/// assert_eq!(scale.interpolate(0.0), "rgb(0,0,0)");
/// assert_eq!(scale.interpolate(100.0), "rgb(255,255,255)");
/// ```
pub struct SequentialScale<T> {
    /// Start of input domain
    domain_min: f64,
    /// End of input domain
    domain_max: f64,
    /// The interpolator function
    interpolator: Interpolator<T>,
    /// Whether to clamp input to domain
    clamp: bool,
}

impl<T> SequentialScale<T> {
    /// Create a new sequential scale with the given interpolator
    ///
    /// # Example
    /// ```
    /// use makepad_d3::scale::SequentialScale;
    ///
    /// let scale = SequentialScale::new(|t| t * 100.0)
    ///     .domain(0.0, 1.0);
    /// ```
    pub fn new<F>(interpolator: F) -> Self
    where
        F: Fn(f64) -> T + Send + Sync + 'static,
    {
        Self {
            domain_min: 0.0,
            domain_max: 1.0,
            interpolator: Box::new(interpolator),
            clamp: false,
        }
    }

    /// Set the input domain
    ///
    /// # Example
    /// ```
    /// use makepad_d3::scale::SequentialScale;
    ///
    /// let scale = SequentialScale::new(|t| t)
    ///     .with_domain(0.0, 100.0);
    /// ```
    pub fn with_domain(mut self, min: f64, max: f64) -> Self {
        self.domain_min = min;
        self.domain_max = max;
        self
    }

    /// Set the input domain
    #[deprecated(
        since = "0.2.0",
        note = "Use `with_domain` instead for consistent builder pattern"
    )]
    pub fn domain(self, min: f64, max: f64) -> Self {
        self.with_domain(min, max)
    }

    /// Set the domain bounds
    pub fn set_domain(&mut self, min: f64, max: f64) {
        self.domain_min = min;
        self.domain_max = max;
    }

    /// Get the domain bounds
    pub fn get_domain(&self) -> (f64, f64) {
        (self.domain_min, self.domain_max)
    }

    /// Enable or disable clamping
    ///
    /// When clamping is enabled, input values outside the domain
    /// are clamped to [0, 1] before interpolation.
    pub fn with_clamp(mut self, clamp: bool) -> Self {
        self.clamp = clamp;
        self
    }

    /// Enable or disable clamping
    #[deprecated(
        since = "0.2.0",
        note = "Use `with_clamp` instead for consistent builder pattern"
    )]
    pub fn clamp(self, clamp: bool) -> Self {
        self.with_clamp(clamp)
    }

    /// Set clamping
    pub fn set_clamp(&mut self, clamp: bool) {
        self.clamp = clamp;
    }

    /// Check if clamping is enabled
    pub fn is_clamped(&self) -> bool {
        self.clamp
    }

    /// Set a new interpolator
    pub fn with_interpolator<F>(mut self, interpolator: F) -> Self
    where
        F: Fn(f64) -> T + Send + Sync + 'static,
    {
        self.interpolator = Box::new(interpolator);
        self
    }

    /// Set a new interpolator
    #[deprecated(
        since = "0.2.0",
        note = "Use `with_interpolator` instead for consistent builder pattern"
    )]
    pub fn interpolator<F>(self, interpolator: F) -> Self
    where
        F: Fn(f64) -> T + Send + Sync + 'static,
    {
        self.with_interpolator(interpolator)
    }

    /// Normalize a domain value to [0, 1]
    fn normalize(&self, value: f64) -> f64 {
        let span = self.domain_max - self.domain_min;
        if span.abs() < f64::EPSILON {
            return 0.5;
        }

        let t = (value - self.domain_min) / span;

        if self.clamp {
            t.clamp(0.0, 1.0)
        } else {
            t
        }
    }

    /// Map a domain value through the interpolator
    ///
    /// This is the primary method to use the scale.
    pub fn interpolate(&self, value: f64) -> T {
        let t = self.normalize(value);
        (self.interpolator)(t)
    }
}

// Implement Scale trait for SequentialScale<f64>
impl Scale for SequentialScale<f64> {
    fn scale_type(&self) -> &'static str {
        "sequential"
    }

    fn set_domain(&mut self, min: f64, max: f64) {
        self.domain_min = min;
        self.domain_max = max;
    }

    fn set_range(&mut self, _start: f64, _end: f64) {
        // Sequential scales use interpolators, not explicit ranges
        // This is a no-op for compatibility
    }

    fn domain(&self) -> (f64, f64) {
        (self.domain_min, self.domain_max)
    }

    fn range(&self) -> (f64, f64) {
        // Return interpolated values at domain bounds
        ((self.interpolator)(0.0), (self.interpolator)(1.0))
    }

    fn scale(&self, value: f64) -> f64 {
        self.interpolate(value)
    }

    fn invert(&self, _value: f64) -> f64 {
        // Inversion requires knowing the inverse of the interpolator
        // which isn't generally possible, so we return NaN
        f64::NAN
    }

    fn ticks(&self, options: &TickOptions) -> Vec<Tick> {
        // Generate evenly spaced ticks across the domain
        let count = options.count.min(options.max_count).max(options.min_count);
        let mut ticks = Vec::with_capacity(count);

        let span = self.domain_max - self.domain_min;
        let step = span / (count - 1).max(1) as f64;

        for i in 0..count {
            let value = self.domain_min + step * i as f64;
            ticks.push(Tick::new(value, format!("{:.2}", value)).with_position(value));
        }

        ticks
    }

    fn copy_from(&mut self, other: &Self) {
        self.domain_min = other.domain_min;
        self.domain_max = other.domain_max;
        self.clamp = other.clamp;
        // Note: We can't copy the interpolator as it's a Box<dyn Fn>
    }

    fn clone_box(&self) -> Box<dyn Scale> {
        // Create a simple linear interpolator as fallback
        let min = self.domain_min;
        let max = self.domain_max;
        let clamp = self.clamp;
        let range_start = (self.interpolator)(0.0);
        let range_end = (self.interpolator)(1.0);

        Box::new(
            SequentialScale::new(move |t| range_start + t * (range_end - range_start))
                .domain(min, max)
                .clamp(clamp),
        )
    }
}

/// Built-in interpolators for common use cases
pub mod interpolators {
    /// Linear interpolation between two values
    pub fn linear(start: f64, end: f64) -> impl Fn(f64) -> f64 + Send + Sync {
        move |t| start + t * (end - start)
    }

    /// Linear interpolation for RGB colors (as [r, g, b] arrays)
    pub fn rgb(start: [f64; 3], end: [f64; 3]) -> impl Fn(f64) -> [f64; 3] + Send + Sync {
        move |t| {
            [
                start[0] + t * (end[0] - start[0]),
                start[1] + t * (end[1] - start[1]),
                start[2] + t * (end[2] - start[2]),
            ]
        }
    }

    /// Linear interpolation for RGBA colors (as [r, g, b, a] arrays)
    pub fn rgba(start: [f64; 4], end: [f64; 4]) -> impl Fn(f64) -> [f64; 4] + Send + Sync {
        move |t| {
            [
                start[0] + t * (end[0] - start[0]),
                start[1] + t * (end[1] - start[1]),
                start[2] + t * (end[2] - start[2]),
                start[3] + t * (end[3] - start[3]),
            ]
        }
    }

    /// Viridis-like color scheme (simplified approximation)
    /// Returns [r, g, b] in range [0, 1]
    pub fn viridis() -> impl Fn(f64) -> [f64; 3] + Send + Sync {
        |t| {
            let t = t.clamp(0.0, 1.0);
            // Simplified viridis approximation
            let r = (0.267004 + t * (0.329415 + t * (0.677367 - t * 0.273786))).clamp(0.0, 1.0);
            let g = (0.004874 + t * (0.815803 + t * (-0.420756 + t * 0.599784))).clamp(0.0, 1.0);
            let b = (0.329415 + t * (0.549132 - t * (0.878725 - t * 0.999932))).clamp(0.0, 1.0);
            [r, g, b]
        }
    }

    /// Plasma-like color scheme (simplified approximation)
    /// Returns [r, g, b] in range [0, 1]
    pub fn plasma() -> impl Fn(f64) -> [f64; 3] + Send + Sync {
        |t| {
            let t = t.clamp(0.0, 1.0);
            // Simplified plasma approximation
            let r = (0.050383 + t * (1.404065 - t * 0.454448)).clamp(0.0, 1.0);
            let g = (0.029803 + t * (0.239107 + t * (1.266775 - t * 0.535685))).clamp(0.0, 1.0);
            let b = (0.529975 + t * (0.688297 - t * (2.118743 - t * 0.900156))).clamp(0.0, 1.0);
            [r, g, b]
        }
    }

    /// Inferno-like color scheme (simplified approximation)
    /// Returns [r, g, b] in range [0, 1]
    pub fn inferno() -> impl Fn(f64) -> [f64; 3] + Send + Sync {
        |t| {
            let t = t.clamp(0.0, 1.0);
            let r = (0.001462 + t * (1.291770 - t * 0.293108)).clamp(0.0, 1.0);
            let g = (0.000466 + t * (0.034820 + t * (1.305855 - t * 0.341095))).clamp(0.0, 1.0);
            let b = (0.013866 + t * (0.957417 - t * (1.765281 - t * 0.794098))).clamp(0.0, 1.0);
            [r, g, b]
        }
    }

    /// Cool color scheme (cyan to magenta)
    /// Returns [r, g, b] in range [0, 1]
    pub fn cool() -> impl Fn(f64) -> [f64; 3] + Send + Sync {
        |t| {
            let t = t.clamp(0.0, 1.0);
            [t, 1.0 - t, 1.0]
        }
    }

    /// Warm color scheme (red to yellow)
    /// Returns [r, g, b] in range [0, 1]
    pub fn warm() -> impl Fn(f64) -> [f64; 3] + Send + Sync {
        |t| {
            let t = t.clamp(0.0, 1.0);
            [1.0, t, 0.0]
        }
    }

    /// Grayscale interpolation
    /// Returns [r, g, b] in range [0, 1]
    pub fn grayscale() -> impl Fn(f64) -> [f64; 3] + Send + Sync {
        |t| {
            let t = t.clamp(0.0, 1.0);
            [t, t, t]
        }
    }

    /// Blues color scheme (light to dark blue)
    /// Returns [r, g, b] in range [0, 1]
    pub fn blues() -> impl Fn(f64) -> [f64; 3] + Send + Sync {
        |t| {
            let t = t.clamp(0.0, 1.0);
            [
                (0.97 - t * 0.75).clamp(0.0, 1.0),
                (0.98 - t * 0.60).clamp(0.0, 1.0),
                (1.0 - t * 0.35).clamp(0.0, 1.0),
            ]
        }
    }

    /// Greens color scheme (light to dark green)
    /// Returns [r, g, b] in range [0, 1]
    pub fn greens() -> impl Fn(f64) -> [f64; 3] + Send + Sync {
        |t| {
            let t = t.clamp(0.0, 1.0);
            [
                (0.97 - t * 0.75).clamp(0.0, 1.0),
                (1.0 - t * 0.40).clamp(0.0, 1.0),
                (0.97 - t * 0.70).clamp(0.0, 1.0),
            ]
        }
    }

    /// Reds color scheme (light to dark red)
    /// Returns [r, g, b] in range [0, 1]
    pub fn reds() -> impl Fn(f64) -> [f64; 3] + Send + Sync {
        |t| {
            let t = t.clamp(0.0, 1.0);
            [
                (1.0 - t * 0.35).clamp(0.0, 1.0),
                (0.96 - t * 0.85).clamp(0.0, 1.0),
                (0.94 - t * 0.85).clamp(0.0, 1.0),
            ]
        }
    }

    /// Rainbow color scheme using HSL
    /// Returns [r, g, b] in range [0, 1]
    pub fn rainbow() -> impl Fn(f64) -> [f64; 3] + Send + Sync {
        |t| {
            let t = t.clamp(0.0, 1.0);
            let h = t * 360.0;
            hsl_to_rgb(h, 1.0, 0.5)
        }
    }

    /// Convert HSL to RGB
    /// h: 0-360, s: 0-1, l: 0-1
    /// Returns [r, g, b] in range [0, 1]
    fn hsl_to_rgb(h: f64, s: f64, l: f64) -> [f64; 3] {
        if s == 0.0 {
            return [l, l, l];
        }

        let q = if l < 0.5 {
            l * (1.0 + s)
        } else {
            l + s - l * s
        };
        let p = 2.0 * l - q;
        let h = h / 360.0;

        let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
        let g = hue_to_rgb(p, q, h);
        let b = hue_to_rgb(p, q, h - 1.0 / 3.0);

        [r, g, b]
    }

    fn hue_to_rgb(p: f64, q: f64, mut t: f64) -> f64 {
        if t < 0.0 {
            t += 1.0;
        }
        if t > 1.0 {
            t -= 1.0;
        }
        if t < 1.0 / 6.0 {
            return p + (q - p) * 6.0 * t;
        }
        if t < 1.0 / 2.0 {
            return q;
        }
        if t < 2.0 / 3.0 {
            return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
        }
        p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequential_scale_new() {
        let scale = SequentialScale::new(|t| t * 100.0);
        assert_eq!(scale.get_domain(), (0.0, 1.0));
    }

    #[test]
    fn test_sequential_scale_domain() {
        let scale = SequentialScale::new(|t| t).domain(0.0, 100.0);

        assert_eq!(scale.get_domain(), (0.0, 100.0));
    }

    #[test]
    fn test_sequential_scale_interpolate() {
        let scale = SequentialScale::new(|t| t * 100.0).domain(0.0, 100.0);

        assert!((scale.interpolate(0.0) - 0.0).abs() < 0.01);
        assert!((scale.interpolate(50.0) - 50.0).abs() < 0.01);
        assert!((scale.interpolate(100.0) - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_sequential_scale_clamp() {
        let scale = SequentialScale::new(|t| t).domain(0.0, 100.0).clamp(true);

        // Values outside domain should be clamped
        assert!((scale.interpolate(-50.0) - 0.0).abs() < 0.01);
        assert!((scale.interpolate(150.0) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_sequential_scale_no_clamp() {
        let scale = SequentialScale::new(|t| t).domain(0.0, 100.0).clamp(false);

        // Values outside domain should extrapolate
        assert!((scale.interpolate(-50.0) - (-0.5)).abs() < 0.01);
        assert!((scale.interpolate(150.0) - 1.5).abs() < 0.01);
    }

    #[test]
    fn test_sequential_scale_rgb_interpolator() {
        let scale = SequentialScale::new(interpolators::rgb([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]))
            .domain(0.0, 100.0);

        let color = scale.interpolate(50.0);
        assert!((color[0] - 0.5).abs() < 0.01);
        assert!((color[1] - 0.5).abs() < 0.01);
        assert!((color[2] - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_sequential_scale_grayscale() {
        let scale = SequentialScale::new(interpolators::grayscale()).domain(0.0, 1.0);

        let black = scale.interpolate(0.0);
        let white = scale.interpolate(1.0);
        let gray = scale.interpolate(0.5);

        assert!((black[0]).abs() < 0.01);
        assert!((white[0] - 1.0).abs() < 0.01);
        assert!((gray[0] - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_sequential_scale_viridis() {
        let scale = SequentialScale::new(interpolators::viridis()).domain(0.0, 1.0);

        let start = scale.interpolate(0.0);
        let end = scale.interpolate(1.0);

        // Viridis starts dark purple-ish and ends yellow-ish
        assert!(start[0] < 0.5); // Low red at start
        assert!(end[1] > 0.5); // High green at end (yellow)
    }

    #[test]
    fn test_sequential_scale_cool() {
        let scale = SequentialScale::new(interpolators::cool()).domain(0.0, 1.0);

        let start = scale.interpolate(0.0);
        let end = scale.interpolate(1.0);

        // Cool goes from cyan to magenta
        assert!((start[0]).abs() < 0.01); // Start: r=0 (cyan)
        assert!((start[1] - 1.0).abs() < 0.01); // Start: g=1 (cyan)
        assert!((end[0] - 1.0).abs() < 0.01); // End: r=1 (magenta)
        assert!((end[1]).abs() < 0.01); // End: g=0 (magenta)
    }

    #[test]
    fn test_sequential_scale_warm() {
        let scale = SequentialScale::new(interpolators::warm()).domain(0.0, 1.0);

        let start = scale.interpolate(0.0);
        let end = scale.interpolate(1.0);

        // Warm goes from red to yellow
        assert!((start[0] - 1.0).abs() < 0.01); // Start: r=1 (red)
        assert!((start[1]).abs() < 0.01); // Start: g=0 (red)
        assert!((end[0] - 1.0).abs() < 0.01); // End: r=1 (yellow)
        assert!((end[1] - 1.0).abs() < 0.01); // End: g=1 (yellow)
    }

    #[test]
    fn test_sequential_scale_blues() {
        let scale = SequentialScale::new(interpolators::blues()).domain(0.0, 1.0);

        let start = scale.interpolate(0.0);
        let end = scale.interpolate(1.0);

        // Blues goes from light blue to dark blue
        assert!(start[2] > 0.9); // High blue at start
        assert!(end[2] > 0.5); // Still blue at end
        assert!(start[0] > end[0]); // Less red at end (darker)
    }

    #[test]
    fn test_sequential_scale_rainbow() {
        let scale = SequentialScale::new(interpolators::rainbow()).domain(0.0, 1.0);

        let red = scale.interpolate(0.0);
        let green = scale.interpolate(0.33);
        let blue = scale.interpolate(0.66);

        // Rainbow cycles through hues
        assert!(red[0] > 0.9); // Red at start
        assert!(green[1] > 0.5); // Green around 1/3
        assert!(blue[2] > 0.5); // Blue around 2/3
    }

    #[test]
    fn test_sequential_scale_f64_scale_trait() {
        let scale = SequentialScale::new(|t| t * 100.0).domain(0.0, 100.0);

        assert_eq!(scale.scale_type(), "sequential");
        assert_eq!(Scale::domain(&scale), (0.0, 100.0));

        assert!((scale.scale(50.0) - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_sequential_scale_ticks() {
        let scale = SequentialScale::new(|t| t).domain(0.0, 100.0);

        let ticks = scale.ticks(&TickOptions::new().with_count(5));
        assert_eq!(ticks.len(), 5);
        assert!((ticks[0].value - 0.0).abs() < 0.01);
        assert!((ticks[4].value - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_sequential_scale_clone_box() {
        let scale = SequentialScale::new(|t| t * 10.0).domain(0.0, 100.0);

        let boxed: Box<dyn Scale> = scale.clone_box();
        assert_eq!(boxed.scale_type(), "sequential");
    }

    #[test]
    fn test_linear_interpolator() {
        let interp = interpolators::linear(0.0, 100.0);
        assert!((interp(0.0) - 0.0).abs() < 0.01);
        assert!((interp(0.5) - 50.0).abs() < 0.01);
        assert!((interp(1.0) - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_rgba_interpolator() {
        let interp = interpolators::rgba([0.0, 0.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0]);

        let mid = interp(0.5);
        assert!((mid[0] - 0.5).abs() < 0.01);
        assert!((mid[3] - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_sequential_scale_custom_interpolator() {
        // Custom interpolator: quadratic ease-in
        let scale = SequentialScale::new(|t| t * t).domain(0.0, 1.0);

        assert!((scale.interpolate(0.0) - 0.0).abs() < 0.01);
        assert!((scale.interpolate(0.5) - 0.25).abs() < 0.01); // 0.5^2 = 0.25
        assert!((scale.interpolate(1.0) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_sequential_scale_negative_domain() {
        let scale = SequentialScale::new(|t| t).domain(-100.0, 100.0);

        assert!((scale.interpolate(-100.0) - 0.0).abs() < 0.01);
        assert!((scale.interpolate(0.0) - 0.5).abs() < 0.01);
        assert!((scale.interpolate(100.0) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_sequential_scale_inverted_domain() {
        let scale = SequentialScale::new(|t| t).domain(100.0, 0.0);

        // With inverted domain, 0 maps to 1 and 100 maps to 0
        assert!((scale.interpolate(0.0) - 1.0).abs() < 0.01);
        assert!((scale.interpolate(100.0) - 0.0).abs() < 0.01);
    }
}

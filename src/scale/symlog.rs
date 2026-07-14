//! Symmetric logarithmic scale implementation

use super::traits::{ContinuousScale, Scale, ScaleExt, Tick, TickOptions};
use super::utils::{format_number, nice_step};

/// Symmetric logarithmic scale for data that crosses zero
///
/// Combines linear behavior near zero with logarithmic behavior for
/// large values. Uses the "symlog" transformation: sign(x) * log(1 + |x|/c)
/// where c is a constant that determines the linear region.
///
/// # Example
/// ```
/// use makepad_d3::scale::{Scale, SymlogScale, ScaleExt};
///
/// let scale = SymlogScale::new()
///     .with_domain(-1000.0, 1000.0)
///     .with_range(0.0, 400.0);
///
/// // Zero maps to center
/// assert!((scale.scale(0.0) - 200.0).abs() < 0.01);
///
/// // Symmetric around zero
/// let pos = scale.scale(100.0);
/// let neg = scale.scale(-100.0);
/// assert!((pos - 200.0 - (200.0 - neg)).abs() < 0.01);
/// ```
#[derive(Clone, Debug)]
pub struct SymlogScale {
    domain_min: f64,
    domain_max: f64,
    range_start: f64,
    range_end: f64,
    /// Constant that determines where linear transitions to log
    constant: f64,
    clamp: bool,
}

impl SymlogScale {
    /// Create a new symlog scale with default constant = 1
    pub fn new() -> Self {
        Self {
            domain_min: -1.0,
            domain_max: 1.0,
            range_start: 0.0,
            range_end: 1.0,
            constant: 1.0,
            clamp: false,
        }
    }

    /// Set the constant that determines the linear threshold
    ///
    /// Smaller values create a smaller linear region near zero.
    /// Larger values extend the linear region.
    pub fn with_constant(mut self, constant: f64) -> Self {
        self.constant = constant.abs().max(f64::EPSILON);
        self
    }

    /// Enable clamping
    pub fn with_clamp(mut self, clamp: bool) -> Self {
        self.clamp = clamp;
        self
    }

    /// Get the current constant
    pub fn constant(&self) -> f64 {
        self.constant
    }

    /// Apply symlog transformation: sign(x) * log(1 + |x|/c)
    fn symlog_transform(&self, x: f64) -> f64 {
        x.signum() * (1.0 + (x.abs() / self.constant)).ln()
    }

    /// Inverse symlog transformation: sign(y) * c * (exp(|y|) - 1)
    fn symlog_inverse(&self, y: f64) -> f64 {
        y.signum() * self.constant * (y.abs().exp() - 1.0)
    }
}

impl Default for SymlogScale {
    fn default() -> Self {
        Self::new()
    }
}

impl Scale for SymlogScale {
    fn scale_type(&self) -> &'static str {
        "symlog"
    }

    fn set_domain(&mut self, min: f64, max: f64) {
        self.domain_min = min;
        self.domain_max = max;
    }

    fn set_range(&mut self, start: f64, end: f64) {
        self.range_start = start;
        self.range_end = end;
    }

    fn domain(&self) -> (f64, f64) {
        (self.domain_min, self.domain_max)
    }

    fn range(&self) -> (f64, f64) {
        (self.range_start, self.range_end)
    }

    fn scale(&self, value: f64) -> f64 {
        let value = if self.clamp {
            value.clamp(
                self.domain_min.min(self.domain_max),
                self.domain_min.max(self.domain_max),
            )
        } else {
            value
        };

        let sym_min = self.symlog_transform(self.domain_min);
        let sym_max = self.symlog_transform(self.domain_max);
        let sym_val = self.symlog_transform(value);

        let domain_span = sym_max - sym_min;
        if domain_span.abs() < f64::EPSILON {
            return self.range_start;
        }

        let t = (sym_val - sym_min) / domain_span;
        self.range_start + t * (self.range_end - self.range_start)
    }

    fn invert(&self, pixel: f64) -> f64 {
        let range_span = self.range_end - self.range_start;
        if range_span.abs() < f64::EPSILON {
            return self.domain_min;
        }

        let t = (pixel - self.range_start) / range_span;

        let sym_min = self.symlog_transform(self.domain_min);
        let sym_max = self.symlog_transform(self.domain_max);
        let sym_val = sym_min + t * (sym_max - sym_min);

        self.symlog_inverse(sym_val)
    }

    fn ticks(&self, options: &TickOptions) -> Vec<Tick> {
        // For symlog, we generate ticks in both positive and negative regions
        let span = self.domain_max - self.domain_min;

        let step = options
            .step_size
            .unwrap_or_else(|| nice_step(span.abs(), options.count));

        if step <= 0.0 {
            return vec![];
        }

        let mut ticks = Vec::new();
        let epsilon = step * 0.0001;

        // Start from a nice multiple of step
        let start = (self.domain_min / step).ceil() * step;

        let mut value = start;
        while value <= self.domain_max + epsilon && ticks.len() < options.max_count {
            let skip = ticks
                .last()
                .map(|t: &Tick| (t.value - value).abs() < epsilon)
                .unwrap_or(false);

            if !skip {
                let pos = self.scale(value);
                ticks.push(Tick::new(value, format_number(value)).with_position(pos));
            }
            value += step;
        }

        ticks
    }

    fn copy_from(&mut self, other: &Self) {
        self.domain_min = other.domain_min;
        self.domain_max = other.domain_max;
        self.range_start = other.range_start;
        self.range_end = other.range_end;
        self.constant = other.constant;
        self.clamp = other.clamp;
    }

    fn clone_box(&self) -> Box<dyn Scale> {
        Box::new(self.clone())
    }
}

impl ContinuousScale for SymlogScale {
    fn nice(&mut self) {
        // For symlog, we want nice bounds that are symmetric if possible
        let abs_max = self.domain_min.abs().max(self.domain_max.abs());

        // Find a nice round number using nice step logic
        let magnitude = 10.0_f64.powf(abs_max.log10().floor());
        let normalized = abs_max / magnitude;

        // Round up to a nice multiple (1, 2, 5, or 10)
        let nice_normalized = if normalized <= 1.0 {
            1.0
        } else if normalized <= 2.0 {
            2.0
        } else if normalized <= 5.0 {
            5.0
        } else {
            10.0
        };

        let nice_max = nice_normalized * magnitude;

        // If domain crosses zero, make it symmetric
        if self.domain_min < 0.0 && self.domain_max > 0.0 {
            self.domain_min = -nice_max;
            self.domain_max = nice_max;
        } else {
            // Otherwise just nice the bounds
            if self.domain_min >= 0.0 {
                self.domain_min = 0.0;
                self.domain_max = nice_max;
            } else {
                self.domain_min = -nice_max;
                self.domain_max = 0.0;
            }
        }
    }

    fn is_clamped(&self) -> bool {
        self.clamp
    }

    fn set_clamp(&mut self, clamp: bool) {
        self.clamp = clamp;
    }
}

impl ScaleExt for SymlogScale {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symlog_scale_new() {
        let scale = SymlogScale::new();
        assert_eq!(scale.domain(), (-1.0, 1.0));
        assert_eq!(scale.range(), (0.0, 1.0));
        assert_eq!(scale.constant(), 1.0);
    }

    #[test]
    fn test_symlog_scale_zero() {
        let scale = SymlogScale::new()
            .with_domain(-100.0, 100.0)
            .with_range(0.0, 200.0);

        // Zero should map to center
        assert!((scale.scale(0.0) - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_symlog_scale_symmetry() {
        let scale = SymlogScale::new()
            .with_domain(-100.0, 100.0)
            .with_range(0.0, 200.0);

        let center = scale.scale(0.0);
        let pos_10 = scale.scale(10.0);
        let neg_10 = scale.scale(-10.0);

        // Distance from center should be equal for ±10
        assert!((pos_10 - center - (center - neg_10)).abs() < 0.01);
    }

    #[test]
    fn test_symlog_scale_endpoints() {
        let scale = SymlogScale::new()
            .with_domain(-100.0, 100.0)
            .with_range(0.0, 200.0);

        assert!((scale.scale(-100.0) - 0.0).abs() < 0.01);
        assert!((scale.scale(100.0) - 200.0).abs() < 0.01);
    }

    #[test]
    fn test_symlog_scale_invert() {
        let scale = SymlogScale::new()
            .with_domain(-100.0, 100.0)
            .with_range(0.0, 200.0);

        // Test roundtrip
        for &value in &[-100.0, -50.0, -10.0, 0.0, 10.0, 50.0, 100.0] {
            let pixel = scale.scale(value);
            let roundtrip = scale.invert(pixel);
            assert!(
                (roundtrip - value).abs() < 0.1,
                "Roundtrip failed for {}: got {}",
                value,
                roundtrip
            );
        }
    }

    #[test]
    fn test_symlog_scale_constant() {
        // Smaller constant = more logarithmic behavior
        let scale_small = SymlogScale::new()
            .with_constant(0.1)
            .with_domain(-100.0, 100.0)
            .with_range(0.0, 200.0);

        // Larger constant = more linear behavior
        let scale_large = SymlogScale::new()
            .with_constant(100.0)
            .with_domain(-100.0, 100.0)
            .with_range(0.0, 200.0);

        // With small constant, small values compress more
        let small_10 = scale_small.scale(10.0);
        let large_10 = scale_large.scale(10.0);

        // Both should be positive (above center)
        assert!(small_10 > 100.0);
        assert!(large_10 > 100.0);
    }

    #[test]
    fn test_symlog_scale_clamp() {
        let scale = SymlogScale::new()
            .with_domain(-100.0, 100.0)
            .with_range(0.0, 200.0)
            .with_clamp(true);

        assert!((scale.scale(-200.0) - 0.0).abs() < 0.01);
        assert!((scale.scale(200.0) - 200.0).abs() < 0.01);
    }

    #[test]
    fn test_symlog_scale_nice() {
        let mut scale = SymlogScale::new().with_domain(-73.5, 82.1);

        scale.nice();

        // Should be symmetric around zero with nice bounds
        assert_eq!(scale.domain(), (-100.0, 100.0));
    }

    #[test]
    fn test_symlog_scale_positive_only() {
        let mut scale = SymlogScale::new().with_domain(5.0, 87.0);

        scale.nice();

        assert_eq!(scale.domain().0, 0.0);
        assert!(scale.domain().1 >= 87.0);
    }

    #[test]
    fn test_symlog_scale_ticks() {
        let scale = SymlogScale::new()
            .with_domain(-100.0, 100.0)
            .with_range(0.0, 400.0);

        let ticks = scale.ticks(&TickOptions::default());

        assert!(!ticks.is_empty());

        // Should have tick at or near zero
        let has_zero_tick = ticks.iter().any(|t| t.value.abs() < 1.0);
        assert!(has_zero_tick);
    }

    #[test]
    fn test_symlog_scale_clone_box() {
        let scale = SymlogScale::new()
            .with_domain(-100.0, 100.0)
            .with_range(0.0, 200.0);

        let boxed: Box<dyn Scale> = scale.clone_box();
        assert_eq!(boxed.scale_type(), "symlog");
    }

    #[test]
    fn test_symlog_linear_near_zero() {
        // With a large constant, behavior should be nearly linear
        let scale = SymlogScale::new()
            .with_constant(1000.0)
            .with_domain(-10.0, 10.0)
            .with_range(0.0, 200.0);

        // Should be approximately linear
        let v1 = scale.scale(5.0);
        let v2 = scale.scale(0.0);
        let v3 = scale.scale(-5.0);

        // Check approximate linearity
        let diff1 = v1 - v2;
        let diff2 = v2 - v3;
        assert!((diff1 - diff2).abs() < 1.0);
    }
}

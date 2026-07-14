//! Power scale implementation

use super::traits::{ContinuousScale, Scale, ScaleExt, Tick, TickOptions};
use super::utils::{format_number, nice_bounds, nice_step};

/// Power scale for polynomial interpolation
///
/// Maps a continuous input domain to a continuous output range using
/// a power function. Useful for perceptually linear scales (like sqrt
/// for area encoding).
///
/// # Example
/// ```
/// use makepad_d3::scale::{Scale, PowScale, ScaleExt};
///
/// // Square root scale (exponent = 0.5)
/// let scale = PowScale::sqrt()
///     .with_domain(0.0, 100.0)
///     .with_range(0.0, 100.0);
///
/// // sqrt(25) / sqrt(100) = 5/10 = 0.5 -> 50
/// assert!((scale.scale(25.0) - 50.0).abs() < 0.01);
/// ```
#[derive(Clone, Debug)]
pub struct PowScale {
    domain_min: f64,
    domain_max: f64,
    range_start: f64,
    range_end: f64,
    exponent: f64,
    clamp: bool,
}

impl PowScale {
    /// Create a new power scale with exponent 1 (linear)
    pub fn new() -> Self {
        Self {
            domain_min: 0.0,
            domain_max: 1.0,
            range_start: 0.0,
            range_end: 1.0,
            exponent: 1.0,
            clamp: false,
        }
    }

    /// Create a square root scale (exponent = 0.5)
    ///
    /// Useful for encoding area - makes visual area proportional to value.
    pub fn sqrt() -> Self {
        Self::new().with_exponent(0.5)
    }

    /// Create a square scale (exponent = 2)
    pub fn square() -> Self {
        Self::new().with_exponent(2.0)
    }

    /// Create a cubic scale (exponent = 3)
    pub fn cubic() -> Self {
        Self::new().with_exponent(3.0)
    }

    /// Set the exponent
    pub fn with_exponent(mut self, exponent: f64) -> Self {
        self.exponent = exponent;
        self
    }

    /// Enable clamping
    pub fn with_clamp(mut self, clamp: bool) -> Self {
        self.clamp = clamp;
        self
    }

    /// Get the current exponent
    pub fn exponent(&self) -> f64 {
        self.exponent
    }

    /// Apply power transformation (handles negative values)
    fn pow_transform(&self, x: f64) -> f64 {
        if self.exponent == 1.0 {
            x
        } else {
            x.signum() * x.abs().powf(self.exponent)
        }
    }

    /// Inverse power transformation
    fn pow_inverse(&self, x: f64) -> f64 {
        if self.exponent == 1.0 {
            x
        } else if self.exponent == 0.0 {
            x.signum()
        } else {
            x.signum() * x.abs().powf(1.0 / self.exponent)
        }
    }
}

impl Default for PowScale {
    fn default() -> Self {
        Self::new()
    }
}

impl Scale for PowScale {
    fn scale_type(&self) -> &'static str {
        "pow"
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

        let pow_min = self.pow_transform(self.domain_min);
        let pow_max = self.pow_transform(self.domain_max);
        let pow_val = self.pow_transform(value);

        let domain_span = pow_max - pow_min;
        if domain_span.abs() < f64::EPSILON {
            return self.range_start;
        }

        let t = (pow_val - pow_min) / domain_span;
        self.range_start + t * (self.range_end - self.range_start)
    }

    fn invert(&self, pixel: f64) -> f64 {
        let range_span = self.range_end - self.range_start;
        if range_span.abs() < f64::EPSILON {
            return self.domain_min;
        }

        let t = (pixel - self.range_start) / range_span;

        let pow_min = self.pow_transform(self.domain_min);
        let pow_max = self.pow_transform(self.domain_max);
        let pow_val = pow_min + t * (pow_max - pow_min);

        self.pow_inverse(pow_val)
    }

    fn ticks(&self, options: &TickOptions) -> Vec<Tick> {
        let span = self.domain_max - self.domain_min;

        let step = options
            .step_size
            .unwrap_or_else(|| nice_step(span.abs(), options.count));

        if step <= 0.0 {
            return vec![];
        }

        let start = (self.domain_min / step).ceil() * step;

        let mut ticks = Vec::new();
        let epsilon = step * 0.0001;

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
        self.exponent = other.exponent;
        self.clamp = other.clamp;
    }

    fn clone_box(&self) -> Box<dyn Scale> {
        Box::new(self.clone())
    }
}

impl ContinuousScale for PowScale {
    fn nice(&mut self) {
        let (nice_min, nice_max) = nice_bounds(self.domain_min, self.domain_max);
        self.domain_min = nice_min;
        self.domain_max = nice_max;
    }

    fn is_clamped(&self) -> bool {
        self.clamp
    }

    fn set_clamp(&mut self, clamp: bool) {
        self.clamp = clamp;
    }
}

impl ScaleExt for PowScale {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pow_scale_new() {
        let scale = PowScale::new();
        assert_eq!(scale.domain(), (0.0, 1.0));
        assert_eq!(scale.range(), (0.0, 1.0));
        assert_eq!(scale.exponent(), 1.0);
    }

    #[test]
    fn test_pow_scale_linear() {
        // Exponent 1 should behave like linear
        let scale = PowScale::new()
            .with_domain(0.0, 100.0)
            .with_range(0.0, 500.0);

        assert!((scale.scale(0.0) - 0.0).abs() < 0.01);
        assert!((scale.scale(50.0) - 250.0).abs() < 0.01);
        assert!((scale.scale(100.0) - 500.0).abs() < 0.01);
    }

    #[test]
    fn test_pow_scale_sqrt() {
        let scale = PowScale::sqrt()
            .with_domain(0.0, 100.0)
            .with_range(0.0, 100.0);

        // sqrt(0) = 0
        assert!((scale.scale(0.0) - 0.0).abs() < 0.01);

        // sqrt(25) / sqrt(100) = 5/10 = 0.5 -> 50
        assert!((scale.scale(25.0) - 50.0).abs() < 0.01);

        // sqrt(100) = 10 -> 100
        assert!((scale.scale(100.0) - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_pow_scale_square() {
        let scale = PowScale::square()
            .with_domain(0.0, 10.0)
            .with_range(0.0, 100.0);

        // 5^2 / 10^2 = 25/100 = 0.25 -> 25
        assert!((scale.scale(5.0) - 25.0).abs() < 0.01);
    }

    #[test]
    fn test_pow_scale_invert() {
        let scale = PowScale::sqrt()
            .with_domain(0.0, 100.0)
            .with_range(0.0, 100.0);

        assert!((scale.invert(0.0) - 0.0).abs() < 0.01);
        assert!((scale.invert(50.0) - 25.0).abs() < 0.01);
        assert!((scale.invert(100.0) - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_pow_scale_negative_values() {
        let scale = PowScale::square()
            .with_domain(-10.0, 10.0)
            .with_range(0.0, 100.0);

        // Should handle negative values via signed power
        let val_neg = scale.scale(-5.0);
        let val_pos = scale.scale(5.0);

        // Both should be equidistant from center (but in different directions)
        let center = scale.scale(0.0);
        assert!((center - val_neg).abs() - (val_pos - center).abs() < 1.0);
    }

    #[test]
    fn test_pow_scale_clamp() {
        let scale = PowScale::sqrt()
            .with_domain(0.0, 100.0)
            .with_range(0.0, 100.0)
            .with_clamp(true);

        assert!((scale.scale(-10.0) - 0.0).abs() < 0.01);
        assert!((scale.scale(200.0) - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_pow_scale_nice() {
        let mut scale = PowScale::sqrt().with_domain(3.2, 97.8);

        scale.nice();

        assert_eq!(scale.domain(), (0.0, 100.0));
    }

    #[test]
    fn test_pow_scale_ticks() {
        let scale = PowScale::sqrt()
            .with_domain(0.0, 100.0)
            .with_range(0.0, 500.0);

        let ticks = scale.ticks(&TickOptions::default());

        assert!(!ticks.is_empty());
    }

    #[test]
    fn test_pow_scale_clone_box() {
        let scale = PowScale::sqrt()
            .with_domain(0.0, 100.0)
            .with_range(0.0, 500.0);

        let boxed: Box<dyn Scale> = scale.clone_box();
        assert_eq!(boxed.scale_type(), "pow");
    }

    #[test]
    fn test_pow_roundtrip() {
        let scale = PowScale::sqrt()
            .with_domain(0.0, 100.0)
            .with_range(0.0, 100.0);

        for &value in &[0.0, 25.0, 50.0, 75.0, 100.0] {
            let pixel = scale.scale(value);
            let roundtrip = scale.invert(pixel);
            assert!((roundtrip - value).abs() < 0.01);
        }
    }

    #[test]
    fn test_pow_cubic() {
        let scale = PowScale::cubic();
        assert_eq!(scale.exponent(), 3.0);
    }
}

//! Logarithmic scale implementation

use super::traits::{ContinuousScale, Scale, ScaleExt, Tick, TickOptions};
use super::utils::format_number;

/// Logarithmic scale for exponential data
///
/// Maps a continuous input domain to a continuous output range using
/// logarithmic interpolation. Useful for data spanning multiple orders
/// of magnitude.
///
/// # Example
/// ```
/// use makepad_d3::scale::{Scale, LogScale, ScaleExt};
///
/// let scale = LogScale::new()
///     .with_domain(1.0, 1000.0)
///     .with_range(0.0, 300.0);
///
/// assert!((scale.scale(1.0) - 0.0).abs() < 0.01);
/// assert!((scale.scale(10.0) - 100.0).abs() < 0.01);
/// assert!((scale.scale(100.0) - 200.0).abs() < 0.01);
/// assert!((scale.scale(1000.0) - 300.0).abs() < 0.01);
/// ```
#[derive(Clone, Debug)]
pub struct LogScale {
    domain_min: f64,
    domain_max: f64,
    range_start: f64,
    range_end: f64,
    base: f64,
    clamp: bool,
}

impl LogScale {
    /// Create a new log scale with default domain [1, 10] and range [0, 1]
    pub fn new() -> Self {
        Self {
            domain_min: 1.0,
            domain_max: 10.0,
            range_start: 0.0,
            range_end: 1.0,
            base: 10.0,
            clamp: false,
        }
    }

    /// Create a log scale with base 2
    pub fn base2() -> Self {
        Self::new().with_base(2.0)
    }

    /// Create a log scale with natural log (base e)
    pub fn ln() -> Self {
        Self::new().with_base(std::f64::consts::E)
    }

    /// Set the logarithm base
    pub fn with_base(mut self, base: f64) -> Self {
        self.base = base.max(1.001); // Ensure base > 1
        self
    }

    /// Enable clamping
    pub fn with_clamp(mut self, clamp: bool) -> Self {
        self.clamp = clamp;
        self
    }

    /// Get the current base
    pub fn base(&self) -> f64 {
        self.base
    }

    /// Calculate log with the configured base
    fn log(&self, x: f64) -> f64 {
        if x <= 0.0 {
            f64::NEG_INFINITY
        } else {
            x.ln() / self.base.ln()
        }
    }

    /// Calculate power with the configured base
    fn pow(&self, x: f64) -> f64 {
        self.base.powf(x)
    }

    /// Format a value for tick labels
    fn format_tick(&self, value: f64) -> String {
        if value >= 1e6 || value < 1e-3 {
            format!("{:.0e}", value)
        } else if value >= 1000.0 {
            format!("{:.0}", value)
        } else if value >= 1.0 {
            format_number(value)
        } else {
            format!("{:.3}", value)
        }
    }
}

impl Default for LogScale {
    fn default() -> Self {
        Self::new()
    }
}

impl Scale for LogScale {
    fn scale_type(&self) -> &'static str {
        "log"
    }

    fn set_domain(&mut self, min: f64, max: f64) {
        // Ensure domain is positive for log scale
        self.domain_min = min.max(f64::EPSILON);
        self.domain_max = max.max(f64::EPSILON);
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
            value.max(f64::EPSILON)
        };

        let log_min = self.log(self.domain_min);
        let log_max = self.log(self.domain_max);
        let log_val = self.log(value);

        if (log_max - log_min).abs() < f64::EPSILON {
            return self.range_start;
        }

        let t = (log_val - log_min) / (log_max - log_min);
        self.range_start + t * (self.range_end - self.range_start)
    }

    fn invert(&self, pixel: f64) -> f64 {
        let range_span = self.range_end - self.range_start;
        if range_span.abs() < f64::EPSILON {
            return self.domain_min;
        }

        let t = (pixel - self.range_start) / range_span;
        let log_min = self.log(self.domain_min);
        let log_max = self.log(self.domain_max);

        self.pow(log_min + t * (log_max - log_min))
    }

    fn ticks(&self, options: &TickOptions) -> Vec<Tick> {
        let mut ticks = Vec::new();

        let log_min = self.log(self.domain_min).floor() as i32;
        let log_max = self.log(self.domain_max).ceil() as i32;

        // Generate ticks at powers of base
        for exp in log_min..=log_max {
            let value = self.pow(exp as f64);
            if value >= self.domain_min && value <= self.domain_max {
                let pos = self.scale(value);
                ticks.push(Tick::new(value, self.format_tick(value)).with_position(pos));
            }
            if ticks.len() >= options.max_count {
                break;
            }
        }

        // If we need more ticks, add intermediate values
        if ticks.len() < options.min_count && log_max - log_min < 3 {
            let mut extra_ticks = Vec::new();
            for exp in log_min..log_max {
                let base_val = self.pow(exp as f64);
                for mult in [2.0, 5.0] {
                    let value = base_val * mult;
                    if value > self.domain_min && value < self.domain_max {
                        let pos = self.scale(value);
                        extra_ticks
                            .push(Tick::new(value, self.format_tick(value)).with_position(pos));
                    }
                }
            }
            ticks.extend(extra_ticks);
            ticks.sort_by(|a, b| a.value.partial_cmp(&b.value).unwrap());
        }

        ticks
    }

    fn copy_from(&mut self, other: &Self) {
        self.domain_min = other.domain_min;
        self.domain_max = other.domain_max;
        self.range_start = other.range_start;
        self.range_end = other.range_end;
        self.base = other.base;
        self.clamp = other.clamp;
    }

    fn clone_box(&self) -> Box<dyn Scale> {
        Box::new(self.clone())
    }
}

impl ContinuousScale for LogScale {
    fn nice(&mut self) {
        self.domain_min = self.pow(self.log(self.domain_min).floor());
        self.domain_max = self.pow(self.log(self.domain_max).ceil());
    }

    fn is_clamped(&self) -> bool {
        self.clamp
    }

    fn set_clamp(&mut self, clamp: bool) {
        self.clamp = clamp;
    }
}

impl ScaleExt for LogScale {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_scale_new() {
        let scale = LogScale::new();
        assert_eq!(scale.domain(), (1.0, 10.0));
        assert_eq!(scale.range(), (0.0, 1.0));
        assert_eq!(scale.base(), 10.0);
    }

    #[test]
    fn test_log_scale_basic() {
        let scale = LogScale::new()
            .with_domain(1.0, 1000.0)
            .with_range(0.0, 300.0);

        assert!((scale.scale(1.0) - 0.0).abs() < 0.01);
        assert!((scale.scale(10.0) - 100.0).abs() < 0.01);
        assert!((scale.scale(100.0) - 200.0).abs() < 0.01);
        assert!((scale.scale(1000.0) - 300.0).abs() < 0.01);
    }

    #[test]
    fn test_log_scale_invert() {
        let scale = LogScale::new()
            .with_domain(1.0, 1000.0)
            .with_range(0.0, 300.0);

        assert!((scale.invert(0.0) - 1.0).abs() < 0.01);
        assert!((scale.invert(100.0) - 10.0).abs() < 0.1);
        assert!((scale.invert(200.0) - 100.0).abs() < 1.0);
        assert!((scale.invert(300.0) - 1000.0).abs() < 1.0);
    }

    #[test]
    fn test_log_scale_base2() {
        let scale = LogScale::base2()
            .with_domain(1.0, 8.0)
            .with_range(0.0, 300.0);

        assert!((scale.scale(1.0) - 0.0).abs() < 0.01);
        assert!((scale.scale(2.0) - 100.0).abs() < 0.01);
        assert!((scale.scale(4.0) - 200.0).abs() < 0.01);
        assert!((scale.scale(8.0) - 300.0).abs() < 0.01);
    }

    #[test]
    fn test_log_scale_nice() {
        let mut scale = LogScale::new().with_domain(3.0, 750.0);

        scale.nice();

        assert_eq!(scale.domain(), (1.0, 1000.0));
    }

    #[test]
    fn test_log_scale_clamp() {
        let scale = LogScale::new()
            .with_domain(10.0, 1000.0)
            .with_range(0.0, 200.0)
            .with_clamp(true);

        // Below domain min should clamp
        assert!((scale.scale(1.0) - 0.0).abs() < 0.01);
        // Above domain max should clamp
        assert!((scale.scale(10000.0) - 200.0).abs() < 0.01);
    }

    #[test]
    fn test_log_scale_ticks() {
        let scale = LogScale::new()
            .with_domain(1.0, 1000.0)
            .with_range(0.0, 300.0);

        let ticks = scale.ticks(&TickOptions::default());

        assert!(!ticks.is_empty());

        // Should have ticks at 1, 10, 100, 1000
        let values: Vec<f64> = ticks.iter().map(|t| t.value).collect();
        assert!(values.contains(&1.0));
        assert!(values.contains(&10.0));
        assert!(values.contains(&100.0));
        assert!(values.contains(&1000.0));
    }

    #[test]
    fn test_log_scale_clone_box() {
        let scale = LogScale::new()
            .with_domain(1.0, 100.0)
            .with_range(0.0, 200.0);

        let boxed: Box<dyn Scale> = scale.clone_box();
        assert_eq!(boxed.scale_type(), "log");
    }

    #[test]
    fn test_log_scale_with_base() {
        let scale = LogScale::new().with_base(2.0);
        assert_eq!(scale.base(), 2.0);
    }

    #[test]
    fn test_log_roundtrip() {
        let scale = LogScale::new()
            .with_domain(1.0, 1000.0)
            .with_range(0.0, 300.0);

        for &value in &[1.0, 10.0, 100.0, 500.0, 1000.0] {
            let pixel = scale.scale(value);
            let roundtrip = scale.invert(pixel);
            assert!((roundtrip - value).abs() / value < 0.01);
        }
    }
}

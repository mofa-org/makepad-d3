//! Linear scale implementation

use super::traits::{ContinuousScale, Scale, ScaleExt, Tick, TickOptions};
use super::utils::{format_number, nice_bounds, nice_step};

/// Linear scale for continuous numeric data
///
/// Maps a continuous input domain to a continuous output range using
/// linear interpolation.
///
/// # Example
/// ```
/// use makepad_d3::scale::{Scale, LinearScale, ScaleExt};
///
/// let scale = LinearScale::new()
///     .with_domain(0.0, 100.0)
///     .with_range(0.0, 500.0);
///
/// assert_eq!(scale.scale(0.0), 0.0);
/// assert_eq!(scale.scale(50.0), 250.0);
/// assert_eq!(scale.scale(100.0), 500.0);
/// ```
#[derive(Clone, Debug)]
pub struct LinearScale {
    domain_min: f64,
    domain_max: f64,
    range_start: f64,
    range_end: f64,
    clamp: bool,
    nice: bool,
}

impl LinearScale {
    /// Create a new linear scale with default domain [0, 1] and range [0, 1]
    pub fn new() -> Self {
        Self {
            domain_min: 0.0,
            domain_max: 1.0,
            range_start: 0.0,
            range_end: 1.0,
            clamp: false,
            nice: false,
        }
    }

    /// Enable nice domain bounds
    pub fn with_nice(mut self, nice: bool) -> Self {
        self.nice = nice;
        if nice {
            let (nice_min, nice_max) = nice_bounds(self.domain_min, self.domain_max);
            self.domain_min = nice_min;
            self.domain_max = nice_max;
        }
        self
    }

    /// Enable clamping
    pub fn with_clamp(mut self, clamp: bool) -> Self {
        self.clamp = clamp;
        self
    }

    /// Extend domain to start at zero (for bar charts)
    pub fn with_zero(mut self) -> Self {
        if self.domain_min > 0.0 {
            self.domain_min = 0.0;
        }
        if self.domain_max < 0.0 {
            self.domain_max = 0.0;
        }
        self
    }

    /// Create from data extent
    pub fn from_extent(min: f64, max: f64) -> Self {
        Self::new().with_domain(min, max)
    }
}

impl Default for LinearScale {
    fn default() -> Self {
        Self::new()
    }
}

impl Scale for LinearScale {
    fn scale_type(&self) -> &'static str {
        "linear"
    }

    fn set_domain(&mut self, min: f64, max: f64) {
        self.domain_min = min;
        self.domain_max = max;

        if self.nice {
            let (nice_min, nice_max) = nice_bounds(min, max);
            self.domain_min = nice_min;
            self.domain_max = nice_max;
        }
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

        let t = self.normalize(value);
        self.range_start + t * (self.range_end - self.range_start)
    }

    fn invert(&self, pixel: f64) -> f64 {
        let range_span = self.range_end - self.range_start;
        if range_span.abs() < f64::EPSILON {
            return self.domain_min;
        }

        let t = (pixel - self.range_start) / range_span;
        self.domain_min + t * (self.domain_max - self.domain_min)
    }

    fn ticks(&self, options: &TickOptions) -> Vec<Tick> {
        let span = self.domain_max - self.domain_min;

        // Determine step size
        let step = options
            .step_size
            .unwrap_or_else(|| nice_step(span.abs(), options.count));

        if step <= 0.0 {
            return vec![];
        }

        // Calculate starting tick
        let start = (self.domain_min / step).ceil() * step;

        let mut ticks = Vec::new();
        let epsilon = step * 0.0001;

        // Add min bound if requested
        if options.include_bounds && start > self.domain_min + epsilon {
            let pos = self.scale(self.domain_min);
            ticks.push(
                Tick::new(self.domain_min, format_number(self.domain_min)).with_position(pos),
            );
        }

        // Generate ticks
        let mut value = start;
        while value <= self.domain_max + epsilon && ticks.len() < options.max_count {
            // Skip if too close to previous
            let skip = ticks
                .last()
                .map(|t| (t.value - value).abs() < epsilon)
                .unwrap_or(false);

            if !skip {
                let pos = self.scale(value);
                ticks.push(Tick::new(value, format_number(value)).with_position(pos));
            }
            value += step;
        }

        // Add max bound if requested
        if options.include_bounds {
            let last_value = ticks.last().map(|t| t.value).unwrap_or(f64::MIN);
            if (self.domain_max - last_value).abs() > epsilon {
                let pos = self.scale(self.domain_max);
                ticks.push(
                    Tick::new(self.domain_max, format_number(self.domain_max)).with_position(pos),
                );
            }
        }

        ticks
    }

    fn copy_from(&mut self, other: &Self) {
        self.domain_min = other.domain_min;
        self.domain_max = other.domain_max;
        self.range_start = other.range_start;
        self.range_end = other.range_end;
        self.clamp = other.clamp;
        self.nice = other.nice;
    }

    fn clone_box(&self) -> Box<dyn Scale> {
        Box::new(self.clone())
    }
}

impl ContinuousScale for LinearScale {
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

impl ScaleExt for LinearScale {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_scale_new() {
        let scale = LinearScale::new();
        assert_eq!(scale.domain(), (0.0, 1.0));
        assert_eq!(scale.range(), (0.0, 1.0));
    }

    #[test]
    fn test_linear_scale_basic() {
        let scale = LinearScale::new()
            .with_domain(0.0, 100.0)
            .with_range(0.0, 500.0);

        assert_eq!(scale.scale(0.0), 0.0);
        assert_eq!(scale.scale(50.0), 250.0);
        assert_eq!(scale.scale(100.0), 500.0);
    }

    #[test]
    fn test_linear_scale_invert() {
        let scale = LinearScale::new()
            .with_domain(0.0, 100.0)
            .with_range(0.0, 500.0);

        assert_eq!(scale.invert(0.0), 0.0);
        assert_eq!(scale.invert(250.0), 50.0);
        assert_eq!(scale.invert(500.0), 100.0);
    }

    #[test]
    fn test_linear_scale_inverted_range() {
        let scale = LinearScale::new()
            .with_domain(0.0, 100.0)
            .with_range(500.0, 0.0); // Inverted range

        assert_eq!(scale.scale(0.0), 500.0);
        assert_eq!(scale.scale(50.0), 250.0);
        assert_eq!(scale.scale(100.0), 0.0);
    }

    #[test]
    fn test_linear_scale_clamp() {
        let scale = LinearScale::new()
            .with_domain(0.0, 100.0)
            .with_range(0.0, 500.0)
            .with_clamp(true);

        assert_eq!(scale.scale(-50.0), 0.0);
        assert_eq!(scale.scale(150.0), 500.0);
    }

    #[test]
    fn test_linear_scale_no_clamp() {
        let scale = LinearScale::new()
            .with_domain(0.0, 100.0)
            .with_range(0.0, 500.0);

        assert_eq!(scale.scale(-50.0), -250.0);
        assert_eq!(scale.scale(150.0), 750.0);
    }

    #[test]
    fn test_linear_scale_nice() {
        let mut scale = LinearScale::new().with_domain(3.2, 97.8);
        scale.nice();

        assert_eq!(scale.domain(), (0.0, 100.0));
    }

    #[test]
    fn test_linear_scale_with_zero() {
        let scale = LinearScale::new().with_domain(50.0, 100.0).with_zero();

        assert_eq!(scale.domain(), (0.0, 100.0));
    }

    #[test]
    fn test_linear_scale_ticks() {
        let scale = LinearScale::new()
            .with_domain(0.0, 100.0)
            .with_range(0.0, 500.0);

        let ticks = scale.ticks(&TickOptions::new().with_count(10));

        assert!(!ticks.is_empty());
        assert!(ticks.len() <= 15);

        // First tick should be at 0
        assert_eq!(ticks[0].value, 0.0);
        assert_eq!(ticks[0].position, 0.0);
    }

    #[test]
    fn test_linear_scale_ticks_with_bounds() {
        let scale = LinearScale::new()
            .with_domain(3.0, 97.0)
            .with_range(0.0, 500.0);

        let ticks = scale.ticks(&TickOptions::new().with_count(10).with_bounds(true));

        // Should include 3.0 and 97.0 as bounds
        assert_eq!(ticks.first().unwrap().value, 3.0);
        assert_eq!(ticks.last().unwrap().value, 97.0);
    }

    #[test]
    fn test_linear_scale_clone_box() {
        let scale = LinearScale::new()
            .with_domain(0.0, 100.0)
            .with_range(0.0, 500.0);

        let boxed: Box<dyn Scale> = scale.clone_box();
        assert_eq!(boxed.scale(50.0), 250.0);
    }

    #[test]
    fn test_linear_scale_is_inverted() {
        let normal = LinearScale::new().with_range(0.0, 100.0);
        let inverted = LinearScale::new().with_range(100.0, 0.0);

        assert!(!normal.is_inverted());
        assert!(inverted.is_inverted());
    }

    #[test]
    fn test_linear_scale_normalize() {
        let scale = LinearScale::new().with_domain(0.0, 100.0);

        assert_eq!(scale.normalize(0.0), 0.0);
        assert_eq!(scale.normalize(50.0), 0.5);
        assert_eq!(scale.normalize(100.0), 1.0);
    }

    #[test]
    fn test_linear_scale_from_extent() {
        let scale = LinearScale::from_extent(10.0, 90.0);
        assert_eq!(scale.domain(), (10.0, 90.0));
    }
}

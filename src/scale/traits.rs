//! Scale trait definitions

/// Options for tick generation
#[derive(Clone, Debug)]
pub struct TickOptions {
    /// Target number of ticks (approximate)
    pub count: usize,

    /// Maximum ticks to show (hard limit)
    pub max_count: usize,

    /// Minimum ticks to show
    pub min_count: usize,

    /// Include domain bounds as ticks
    pub include_bounds: bool,

    /// Custom step size (overrides count)
    pub step_size: Option<f64>,
}

impl Default for TickOptions {
    fn default() -> Self {
        Self {
            count: 10,
            max_count: 20,
            min_count: 2,
            include_bounds: false,
            step_size: None,
        }
    }
}

impl TickOptions {
    /// Create new tick options with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set target tick count
    pub fn with_count(mut self, count: usize) -> Self {
        self.count = count;
        self
    }

    /// Set maximum tick count
    pub fn with_max_count(mut self, max: usize) -> Self {
        self.max_count = max;
        self
    }

    /// Set minimum tick count
    pub fn with_min_count(mut self, min: usize) -> Self {
        self.min_count = min;
        self
    }

    /// Set custom step size
    pub fn with_step_size(mut self, step: f64) -> Self {
        self.step_size = Some(step);
        self
    }

    /// Include domain bounds as ticks
    pub fn with_bounds(mut self, include: bool) -> Self {
        self.include_bounds = include;
        self
    }
}

/// A tick mark on a scale
#[derive(Clone, Debug, PartialEq)]
pub struct Tick {
    /// The value in the domain
    pub value: f64,

    /// The formatted label
    pub label: String,

    /// Position in pixels (computed from scale)
    pub position: f64,
}

impl Tick {
    /// Create a new tick
    pub fn new(value: f64, label: impl Into<String>) -> Self {
        Self {
            value,
            label: label.into(),
            position: 0.0,
        }
    }

    /// Set the position
    pub fn with_position(mut self, position: f64) -> Self {
        self.position = position;
        self
    }
}

/// Core trait for all scales
///
/// A scale maps values from a domain (input space) to a range (output space).
///
/// # Example
/// ```
/// use makepad_d3::scale::{Scale, LinearScale};
///
/// let mut scale = LinearScale::new();
/// scale.set_domain(0.0, 100.0);
/// scale.set_range(0.0, 500.0);
///
/// assert_eq!(scale.scale(50.0), 250.0);
/// assert_eq!(scale.invert(250.0), 50.0);
/// ```
pub trait Scale: Send + Sync {
    /// Get the scale type identifier
    fn scale_type(&self) -> &'static str;

    /// Set the input domain (data space)
    fn set_domain(&mut self, min: f64, max: f64);

    /// Set the output range (pixel space)
    fn set_range(&mut self, start: f64, end: f64);

    /// Get the current domain bounds
    fn domain(&self) -> (f64, f64);

    /// Get the current range bounds
    fn range(&self) -> (f64, f64);

    /// Map a domain value to range value
    fn scale(&self, value: f64) -> f64;

    /// Map a range value back to domain value
    fn invert(&self, value: f64) -> f64;

    /// Generate tick marks for this scale
    fn ticks(&self, options: &TickOptions) -> Vec<Tick>;

    /// Check if the range is inverted (start > end)
    fn is_inverted(&self) -> bool {
        let (start, end) = self.range();
        start > end
    }

    /// Clamp a value to the domain bounds
    fn clamp_domain(&self, value: f64) -> f64 {
        let (min, max) = self.domain();
        value.clamp(min.min(max), min.max(max))
    }

    /// Clamp a value to the range bounds
    fn clamp_range(&self, value: f64) -> f64 {
        let (start, end) = self.range();
        value.clamp(start.min(end), start.max(end))
    }

    /// Normalize a domain value to [0, 1]
    fn normalize(&self, value: f64) -> f64 {
        let (min, max) = self.domain();
        if (max - min).abs() < f64::EPSILON {
            0.5
        } else {
            (value - min) / (max - min)
        }
    }

    /// Copy configuration from another scale of the same type
    fn copy_from(&mut self, other: &Self)
    where
        Self: Sized;

    /// Clone into a boxed trait object
    fn clone_box(&self) -> Box<dyn Scale>;
}

/// Extension trait for scale configuration (builder pattern)
pub trait ScaleExt: Scale + Sized {
    /// Configure domain and return self (for chaining)
    fn with_domain(mut self, min: f64, max: f64) -> Self {
        self.set_domain(min, max);
        self
    }

    /// Configure range and return self (for chaining)
    fn with_range(mut self, start: f64, end: f64) -> Self {
        self.set_range(start, end);
        self
    }
}

/// Marker trait for continuous scales (linear, log, pow, time)
pub trait ContinuousScale: Scale {
    /// Extend domain to "nice" round values
    fn nice(&mut self);

    /// Check if clamping is enabled
    fn is_clamped(&self) -> bool;

    /// Enable/disable clamping
    fn set_clamp(&mut self, clamp: bool);
}

/// Marker trait for discrete/ordinal scales (category, band, point)
pub trait DiscreteScale: Scale {
    /// Get the bandwidth (space allocated for each item)
    fn bandwidth(&self) -> f64;

    /// Get the step (distance between items)
    fn step(&self) -> f64;

    /// Set padding between items (0-1)
    fn set_padding(&mut self, padding: f64);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tick_options_default() {
        let opts = TickOptions::default();
        assert_eq!(opts.count, 10);
        assert_eq!(opts.max_count, 20);
        assert!(!opts.include_bounds);
    }

    #[test]
    fn test_tick_options_builder() {
        let opts = TickOptions::new()
            .with_count(5)
            .with_step_size(10.0)
            .with_bounds(true);

        assert_eq!(opts.count, 5);
        assert_eq!(opts.step_size, Some(10.0));
        assert!(opts.include_bounds);
    }

    #[test]
    fn test_tick_new() {
        let tick = Tick::new(50.0, "50");
        assert_eq!(tick.value, 50.0);
        assert_eq!(tick.label, "50");
        assert_eq!(tick.position, 0.0);
    }

    #[test]
    fn test_tick_with_position() {
        let tick = Tick::new(50.0, "50").with_position(250.0);
        assert_eq!(tick.position, 250.0);
    }
}

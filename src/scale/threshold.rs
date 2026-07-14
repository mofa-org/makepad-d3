//! Threshold scale implementation
//!
//! Threshold scales map continuous input to discrete output using
//! explicitly specified threshold values. Unlike quantize (equal segments)
//! or quantile (equal counts), threshold scales let you define exact
//! breakpoints.

use super::traits::{Scale, Tick, TickOptions};

/// Scale that maps continuous input to discrete output using explicit thresholds
///
/// Threshold scales are useful when you have meaningful breakpoints in your data.
/// For n threshold values, you get n+1 output buckets.
///
/// # D3.js Equivalent
/// This is equivalent to `d3.scaleThreshold()` in D3.js.
///
/// # Comparison with Other Discretizing Scales
/// - **QuantizeScale**: Equal-sized domain segments (computed from domain extent)
/// - **QuantileScale**: Equal-count segments (computed from data distribution)
/// - **ThresholdScale**: Custom breakpoints (you specify the thresholds)
///
/// # Example
/// ```
/// use makepad_d3::scale::ThresholdScale;
///
/// // Temperature classification with meaningful breakpoints
/// let scale = ThresholdScale::new()
///     .domain(vec![0.0, 20.0, 30.0])  // 3 thresholds = 4 buckets
///     .range(vec!["freezing", "cold", "warm", "hot"]);
///
/// assert_eq!(scale.scale_to_value(-10.0), Some(&"freezing")); // < 0
/// assert_eq!(scale.scale_to_value(10.0), Some(&"cold"));      // 0-20
/// assert_eq!(scale.scale_to_value(25.0), Some(&"warm"));      // 20-30
/// assert_eq!(scale.scale_to_value(35.0), Some(&"hot"));       // >= 30
/// ```
#[derive(Clone, Debug)]
pub struct ThresholdScale<T: Clone> {
    /// Threshold values that separate buckets (should be sorted)
    thresholds: Vec<f64>,
    /// Discrete output values (should have len = thresholds.len() + 1)
    range_values: Vec<T>,
}

impl<T: Clone> ThresholdScale<T> {
    /// Create a new threshold scale with default settings
    pub fn new() -> Self {
        Self {
            thresholds: Vec::new(),
            range_values: Vec::new(),
        }
    }

    /// Set the threshold values (domain)
    ///
    /// The thresholds define the boundaries between buckets.
    /// For n thresholds, you need n+1 range values.
    ///
    /// Values should be in ascending order.
    ///
    /// # Example
    /// ```
    /// use makepad_d3::scale::ThresholdScale;
    ///
    /// // Grade boundaries
    /// let scale: ThresholdScale<&str> = ThresholdScale::new()
    ///     .with_domain(vec![60.0, 70.0, 80.0, 90.0]);  // F, D, C, B, A
    /// ```
    pub fn with_domain(mut self, thresholds: Vec<f64>) -> Self {
        self.thresholds = thresholds;
        // Ensure sorted order
        self.thresholds
            .sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        self
    }

    /// Set the threshold values (deprecated)
    #[deprecated(
        since = "0.2.0",
        note = "Use `with_domain` instead for consistent builder pattern"
    )]
    pub fn domain(self, thresholds: Vec<f64>) -> Self {
        self.with_domain(thresholds)
    }

    /// Set the threshold values
    pub fn set_thresholds(&mut self, thresholds: Vec<f64>) {
        self.thresholds = thresholds;
        self.thresholds
            .sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    }

    /// Get the threshold values
    pub fn thresholds(&self) -> &[f64] {
        &self.thresholds
    }

    /// Set the discrete output range values
    ///
    /// The number of range values should be thresholds.len() + 1.
    ///
    /// # Example
    /// ```
    /// use makepad_d3::scale::ThresholdScale;
    ///
    /// let scale = ThresholdScale::new()
    ///     .with_domain(vec![0.0, 100.0])  // 2 thresholds
    ///     .with_range(vec!["negative", "small", "large"]);  // 3 values
    /// ```
    pub fn with_range(mut self, values: Vec<T>) -> Self {
        self.range_values = values;
        self
    }

    /// Set the discrete output range values (deprecated)
    #[deprecated(
        since = "0.2.0",
        note = "Use `with_range` instead for consistent builder pattern"
    )]
    pub fn range(self, values: Vec<T>) -> Self {
        self.with_range(values)
    }

    /// Set the range values
    pub fn set_range(&mut self, values: Vec<T>) {
        self.range_values = values;
    }

    /// Get the range values
    pub fn range_values(&self) -> &[T] {
        &self.range_values
    }

    /// Get the number of thresholds
    pub fn threshold_count(&self) -> usize {
        self.thresholds.len()
    }

    /// Get the number of output buckets
    pub fn bucket_count(&self) -> usize {
        if self.range_values.is_empty() {
            0
        } else {
            self.range_values.len()
        }
    }

    /// Check if the scale is properly configured
    ///
    /// Returns true if range values count equals thresholds count + 1
    pub fn is_valid(&self) -> bool {
        !self.range_values.is_empty() && self.range_values.len() == self.thresholds.len() + 1
    }

    /// Map a continuous input value to a discrete output value
    ///
    /// Returns `None` if the range is empty.
    pub fn scale_to_value(&self, value: f64) -> Option<&T> {
        if self.range_values.is_empty() {
            return None;
        }

        let index = self.scale_to_index(value);
        self.range_values.get(index)
    }

    /// Map a continuous input value to the index of the output value
    pub fn scale_to_index(&self, value: f64) -> usize {
        if self.range_values.is_empty() {
            return 0;
        }

        // Binary search for the appropriate bucket
        let mut lo = 0;
        let mut hi = self.thresholds.len();

        while lo < hi {
            let mid = (lo + hi) / 2;
            if value < self.thresholds[mid] {
                hi = mid;
            } else {
                lo = mid + 1;
            }
        }

        // Clamp to valid range indices
        lo.min(self.range_values.len().saturating_sub(1))
    }

    /// Get the domain extent [x0, x1] that maps to a specific range index
    ///
    /// Returns the range of input values that map to the given bucket.
    /// For the first bucket, x0 is negative infinity.
    /// For the last bucket, x1 is positive infinity.
    pub fn invert_extent(&self, index: usize) -> (f64, f64) {
        if self.range_values.is_empty() || index >= self.range_values.len() {
            return (f64::NAN, f64::NAN);
        }

        let x0 = if index == 0 {
            f64::NEG_INFINITY
        } else if index - 1 < self.thresholds.len() {
            self.thresholds[index - 1]
        } else {
            f64::INFINITY
        };

        let x1 = if index < self.thresholds.len() {
            self.thresholds[index]
        } else {
            f64::INFINITY
        };

        (x0, x1)
    }

    /// Get the finite domain extent [x0, x1] that maps to a specific range index
    ///
    /// Like `invert_extent` but clamps infinities to the threshold bounds.
    pub fn invert_extent_finite(&self, index: usize) -> Option<(f64, f64)> {
        if self.thresholds.is_empty()
            || self.range_values.is_empty()
            || index >= self.range_values.len()
        {
            return None;
        }

        let min_threshold = self.thresholds[0];
        let max_threshold = self.thresholds[self.thresholds.len() - 1];

        let x0 = if index == 0 {
            min_threshold // Use first threshold as lower bound representation
        } else {
            self.thresholds[index - 1]
        };

        let x1 = if index < self.thresholds.len() {
            self.thresholds[index]
        } else {
            max_threshold // Use last threshold as upper bound representation
        };

        Some((x0, x1))
    }

    /// Find the domain extent for a specific range value
    ///
    /// Returns `None` if the value is not in the range.
    pub fn invert_extent_value(&self, value: &T) -> Option<(f64, f64)>
    where
        T: PartialEq,
    {
        self.range_values
            .iter()
            .position(|v| v == value)
            .map(|i| self.invert_extent(i))
    }
}

impl<T: Clone> Default for ThresholdScale<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// ThresholdScale with f64 range values can implement the Scale trait
impl Scale for ThresholdScale<f64> {
    fn scale_type(&self) -> &'static str {
        "threshold"
    }

    fn set_domain(&mut self, min: f64, max: f64) {
        // For Scale trait compatibility, create a single threshold at midpoint
        self.thresholds = vec![(min + max) / 2.0];
    }

    fn set_range(&mut self, start: f64, end: f64) {
        // For Scale trait compatibility, create a 2-value range
        self.range_values = vec![start, end];
    }

    fn domain(&self) -> (f64, f64) {
        if self.thresholds.is_empty() {
            (0.0, 1.0)
        } else {
            (
                self.thresholds[0],
                self.thresholds[self.thresholds.len() - 1],
            )
        }
    }

    fn range(&self) -> (f64, f64) {
        if self.range_values.is_empty() {
            (0.0, 1.0)
        } else {
            (
                self.range_values[0],
                self.range_values[self.range_values.len() - 1],
            )
        }
    }

    fn scale(&self, value: f64) -> f64 {
        self.scale_to_value(value).copied().unwrap_or(0.0)
    }

    fn invert(&self, value: f64) -> f64 {
        // Find which range value this is and return the midpoint of its extent
        if let Some(index) = self
            .range_values
            .iter()
            .position(|&v| (v - value).abs() < f64::EPSILON)
        {
            let (x0, x1) = self.invert_extent(index);
            if x0.is_infinite() && x1.is_infinite() {
                return f64::NAN;
            }
            if x0.is_infinite() {
                return x1;
            }
            if x1.is_infinite() {
                return x0;
            }
            (x0 + x1) / 2.0
        } else {
            f64::NAN
        }
    }

    fn ticks(&self, options: &TickOptions) -> Vec<Tick> {
        // Generate ticks at threshold values
        let mut ticks = Vec::with_capacity(self.thresholds.len());

        for &threshold in &self.thresholds {
            if ticks.len() < options.max_count {
                ticks.push(
                    Tick::new(threshold, format!("{:.2}", threshold)).with_position(threshold),
                );
            }
        }

        ticks
    }

    fn copy_from(&mut self, other: &Self) {
        self.thresholds = other.thresholds.clone();
        self.range_values = other.range_values.clone();
    }

    fn clone_box(&self) -> Box<dyn Scale> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threshold_scale_new() {
        let scale: ThresholdScale<&str> = ThresholdScale::new();
        assert!(scale.thresholds().is_empty());
        assert!(scale.range_values().is_empty());
    }

    #[test]
    fn test_threshold_scale_basic() {
        let scale = ThresholdScale::new()
            .domain(vec![0.0, 20.0, 30.0])
            .range(vec!["freezing", "cold", "warm", "hot"]);

        assert_eq!(scale.threshold_count(), 3);
        assert_eq!(scale.bucket_count(), 4);
        assert!(scale.is_valid());
    }

    #[test]
    fn test_threshold_scale_mapping() {
        let scale = ThresholdScale::new()
            .domain(vec![0.0, 20.0, 30.0])
            .range(vec!["freezing", "cold", "warm", "hot"]);

        // Below first threshold
        assert_eq!(scale.scale_to_value(-10.0), Some(&"freezing"));
        assert_eq!(scale.scale_to_value(-1.0), Some(&"freezing"));

        // At and above first threshold, below second
        assert_eq!(scale.scale_to_value(0.0), Some(&"cold"));
        assert_eq!(scale.scale_to_value(10.0), Some(&"cold"));
        assert_eq!(scale.scale_to_value(19.9), Some(&"cold"));

        // At and above second threshold, below third
        assert_eq!(scale.scale_to_value(20.0), Some(&"warm"));
        assert_eq!(scale.scale_to_value(25.0), Some(&"warm"));
        assert_eq!(scale.scale_to_value(29.9), Some(&"warm"));

        // At and above third threshold
        assert_eq!(scale.scale_to_value(30.0), Some(&"hot"));
        assert_eq!(scale.scale_to_value(100.0), Some(&"hot"));
    }

    #[test]
    fn test_threshold_scale_single_threshold() {
        let scale = ThresholdScale::new()
            .domain(vec![50.0])
            .range(vec!["below", "above"]);

        assert_eq!(scale.scale_to_value(0.0), Some(&"below"));
        assert_eq!(scale.scale_to_value(49.9), Some(&"below"));
        assert_eq!(scale.scale_to_value(50.0), Some(&"above"));
        assert_eq!(scale.scale_to_value(100.0), Some(&"above"));
    }

    #[test]
    fn test_threshold_scale_to_index() {
        let scale = ThresholdScale::new()
            .domain(vec![10.0, 20.0, 30.0])
            .range(vec!["A", "B", "C", "D"]);

        assert_eq!(scale.scale_to_index(5.0), 0);
        assert_eq!(scale.scale_to_index(10.0), 1);
        assert_eq!(scale.scale_to_index(15.0), 1);
        assert_eq!(scale.scale_to_index(20.0), 2);
        assert_eq!(scale.scale_to_index(25.0), 2);
        assert_eq!(scale.scale_to_index(30.0), 3);
        assert_eq!(scale.scale_to_index(100.0), 3);
    }

    #[test]
    fn test_threshold_scale_invert_extent() {
        let scale = ThresholdScale::new()
            .domain(vec![10.0, 20.0, 30.0])
            .range(vec!["A", "B", "C", "D"]);

        let (x0, x1) = scale.invert_extent(0);
        assert!(x0.is_infinite() && x0.is_sign_negative());
        assert!((x1 - 10.0).abs() < 0.01);

        let (x0, x1) = scale.invert_extent(1);
        assert!((x0 - 10.0).abs() < 0.01);
        assert!((x1 - 20.0).abs() < 0.01);

        let (x0, x1) = scale.invert_extent(2);
        assert!((x0 - 20.0).abs() < 0.01);
        assert!((x1 - 30.0).abs() < 0.01);

        let (x0, x1) = scale.invert_extent(3);
        assert!((x0 - 30.0).abs() < 0.01);
        assert!(x1.is_infinite() && x1.is_sign_positive());
    }

    #[test]
    fn test_threshold_scale_invert_extent_value() {
        let scale = ThresholdScale::new()
            .domain(vec![0.0, 100.0])
            .range(vec!["negative", "small", "large"]);

        let extent = scale.invert_extent_value(&"small");
        assert!(extent.is_some());
        let (x0, x1) = extent.unwrap();
        assert!((x0 - 0.0).abs() < 0.01);
        assert!((x1 - 100.0).abs() < 0.01);

        assert!(scale.invert_extent_value(&"unknown").is_none());
    }

    #[test]
    fn test_threshold_scale_domain_sorted() {
        // Thresholds should be sorted automatically
        let scale = ThresholdScale::new()
            .domain(vec![30.0, 10.0, 20.0])
            .range(vec!["A", "B", "C", "D"]);

        assert_eq!(scale.thresholds(), &[10.0, 20.0, 30.0]);
    }

    #[test]
    fn test_threshold_scale_empty() {
        let scale: ThresholdScale<&str> = ThresholdScale::new();

        assert!(!scale.is_valid());
        assert_eq!(scale.scale_to_value(50.0), None);
    }

    #[test]
    fn test_threshold_scale_no_thresholds() {
        // No thresholds = single bucket containing everything
        let scale = ThresholdScale::new().domain(vec![]).range(vec!["all"]);

        assert!(scale.is_valid());
        assert_eq!(scale.scale_to_value(-100.0), Some(&"all"));
        assert_eq!(scale.scale_to_value(0.0), Some(&"all"));
        assert_eq!(scale.scale_to_value(100.0), Some(&"all"));
    }

    #[test]
    fn test_threshold_scale_grades() {
        // Classic grading scale example
        let scale = ThresholdScale::new()
            .domain(vec![60.0, 70.0, 80.0, 90.0])
            .range(vec!["F", "D", "C", "B", "A"]);

        assert_eq!(scale.scale_to_value(55.0), Some(&"F"));
        assert_eq!(scale.scale_to_value(65.0), Some(&"D"));
        assert_eq!(scale.scale_to_value(75.0), Some(&"C"));
        assert_eq!(scale.scale_to_value(85.0), Some(&"B"));
        assert_eq!(scale.scale_to_value(95.0), Some(&"A"));
    }

    #[test]
    fn test_threshold_scale_with_numbers() {
        let scale = ThresholdScale::new()
            .domain(vec![0.25, 0.5, 0.75])
            .range(vec![0.0, 1.0, 2.0, 3.0]);

        assert_eq!(scale.scale_to_value(0.1), Some(&0.0));
        assert_eq!(scale.scale_to_value(0.3), Some(&1.0));
        assert_eq!(scale.scale_to_value(0.6), Some(&2.0));
        assert_eq!(scale.scale_to_value(0.9), Some(&3.0));
    }

    #[test]
    fn test_threshold_scale_f64_scale_trait() {
        let scale = ThresholdScale::new()
            .domain(vec![10.0, 20.0, 30.0])
            .range(vec![0.0, 1.0, 2.0, 3.0]);

        assert_eq!(scale.scale_type(), "threshold");
        assert_eq!(Scale::domain(&scale), (10.0, 30.0));
        assert_eq!(Scale::range(&scale), (0.0, 3.0));

        assert!((scale.scale(5.0) - 0.0).abs() < 0.01);
        assert!((scale.scale(15.0) - 1.0).abs() < 0.01);
        assert!((scale.scale(25.0) - 2.0).abs() < 0.01);
        assert!((scale.scale(35.0) - 3.0).abs() < 0.01);
    }

    #[test]
    fn test_threshold_scale_clone_box() {
        let scale = ThresholdScale::new()
            .domain(vec![50.0])
            .range(vec![0.0, 1.0]);

        let boxed: Box<dyn Scale> = scale.clone_box();
        assert_eq!(boxed.scale_type(), "threshold");
    }

    #[test]
    fn test_threshold_scale_ticks() {
        let scale = ThresholdScale::new()
            .domain(vec![10.0, 20.0, 30.0])
            .range(vec![0.0, 1.0, 2.0, 3.0]);

        let ticks = scale.ticks(&TickOptions::default());
        assert_eq!(ticks.len(), 3);
        assert!((ticks[0].value - 10.0).abs() < 0.01);
        assert!((ticks[1].value - 20.0).abs() < 0.01);
        assert!((ticks[2].value - 30.0).abs() < 0.01);
    }

    #[test]
    fn test_threshold_scale_colors() {
        // Common use case: choropleth color mapping
        let scale = ThresholdScale::new()
            .domain(vec![10000.0, 50000.0, 100000.0])
            .range(vec!["#ffffcc", "#a1dab4", "#41b6c4", "#225ea8"]);

        assert_eq!(scale.scale_to_value(5000.0), Some(&"#ffffcc"));
        assert_eq!(scale.scale_to_value(25000.0), Some(&"#a1dab4"));
        assert_eq!(scale.scale_to_value(75000.0), Some(&"#41b6c4"));
        assert_eq!(scale.scale_to_value(200000.0), Some(&"#225ea8"));
    }

    #[test]
    fn test_threshold_scale_negative_thresholds() {
        let scale = ThresholdScale::new()
            .domain(vec![-20.0, 0.0, 20.0])
            .range(vec!["very cold", "cold", "mild", "warm"]);

        assert_eq!(scale.scale_to_value(-30.0), Some(&"very cold"));
        assert_eq!(scale.scale_to_value(-10.0), Some(&"cold"));
        assert_eq!(scale.scale_to_value(10.0), Some(&"mild"));
        assert_eq!(scale.scale_to_value(30.0), Some(&"warm"));
    }

    #[test]
    fn test_threshold_scale_mismatched_range() {
        // Range has wrong number of values (should be 4 for 3 thresholds)
        let scale = ThresholdScale::new()
            .domain(vec![10.0, 20.0, 30.0])
            .range(vec!["A", "B"]); // Only 2 values instead of 4

        assert!(!scale.is_valid());
        // Should still work, just clamps to available range
        assert_eq!(scale.scale_to_value(5.0), Some(&"A"));
        assert_eq!(scale.scale_to_value(100.0), Some(&"B"));
    }

    #[test]
    fn test_threshold_scale_at_boundary() {
        let scale = ThresholdScale::new()
            .domain(vec![10.0])
            .range(vec!["below", "at_or_above"]);

        // Values exactly at threshold go to the higher bucket
        assert_eq!(scale.scale_to_value(9.999), Some(&"below"));
        assert_eq!(scale.scale_to_value(10.0), Some(&"at_or_above"));
        assert_eq!(scale.scale_to_value(10.001), Some(&"at_or_above"));
    }

    #[test]
    fn test_threshold_scale_many_thresholds() {
        let thresholds: Vec<f64> = (1..=9).map(|x| x as f64 * 10.0).collect();
        let range_values: Vec<&str> = vec![
            "0-10", "10-20", "20-30", "30-40", "40-50", "50-60", "60-70", "70-80", "80-90", "90+",
        ];

        let scale = ThresholdScale::new().domain(thresholds).range(range_values);

        assert_eq!(scale.threshold_count(), 9);
        assert_eq!(scale.bucket_count(), 10);
        assert!(scale.is_valid());

        assert_eq!(scale.scale_to_value(5.0), Some(&"0-10"));
        assert_eq!(scale.scale_to_value(55.0), Some(&"50-60"));
        assert_eq!(scale.scale_to_value(95.0), Some(&"90+"));
    }
}

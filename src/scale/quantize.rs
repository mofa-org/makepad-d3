//! Quantize scale implementation
//!
//! Quantize scales map a continuous domain to a discrete range by dividing
//! the domain into uniform segments. Each segment maps to one range value.

use super::traits::{Scale, Tick, TickOptions};

/// Scale that maps continuous input to discrete output values
///
/// Quantize scales divide the continuous input domain into uniform segments,
/// each mapping to a discrete output value from the range. This is useful
/// for choropleth maps, heat maps, and other visualizations that need to
/// bucket continuous data into discrete categories.
///
/// # D3.js Equivalent
/// This is equivalent to `d3.scaleQuantize()` in D3.js.
///
/// # Example
/// ```
/// use makepad_d3::scale::QuantizeScale;
///
/// // Create a scale that maps 0-100 to color categories
/// let scale = QuantizeScale::new()
///     .domain(0.0, 100.0)
///     .range(vec!["low", "medium", "high"]);
///
/// // Values are bucketed into equal segments
/// assert_eq!(scale.scale_to_value(10.0), Some(&"low"));      // 0-33.3
/// assert_eq!(scale.scale_to_value(50.0), Some(&"medium"));   // 33.3-66.6
/// assert_eq!(scale.scale_to_value(90.0), Some(&"high"));     // 66.6-100
/// ```
#[derive(Clone, Debug)]
pub struct QuantizeScale<T: Clone> {
    /// Start of input domain
    domain_min: f64,
    /// End of input domain
    domain_max: f64,
    /// Discrete output values
    range_values: Vec<T>,
    /// Computed thresholds between segments
    thresholds: Vec<f64>,
}

impl<T: Clone> QuantizeScale<T> {
    /// Create a new quantize scale with default settings
    pub fn new() -> Self {
        Self {
            domain_min: 0.0,
            domain_max: 1.0,
            range_values: Vec::new(),
            thresholds: Vec::new(),
        }
    }

    /// Set the continuous input domain
    ///
    /// # Example
    /// ```
    /// use makepad_d3::scale::QuantizeScale;
    ///
    /// let scale: QuantizeScale<&str> = QuantizeScale::new()
    ///     .with_domain(0.0, 100.0);
    /// ```
    pub fn with_domain(mut self, min: f64, max: f64) -> Self {
        self.domain_min = min;
        self.domain_max = max;
        self.rescale();
        self
    }

    /// Set the continuous input domain (deprecated)
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
        self.rescale();
    }

    /// Get the domain bounds
    pub fn get_domain(&self) -> (f64, f64) {
        (self.domain_min, self.domain_max)
    }

    /// Set the discrete output range values
    ///
    /// The number of range values determines how many segments
    /// the domain is divided into.
    ///
    /// # Example
    /// ```
    /// use makepad_d3::scale::QuantizeScale;
    ///
    /// let scale = QuantizeScale::new()
    ///     .with_domain(0.0, 100.0)
    ///     .with_range(vec!["cold", "warm", "hot"]);
    /// ```
    pub fn with_range(mut self, values: Vec<T>) -> Self {
        self.range_values = values;
        self.rescale();
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
        self.rescale();
    }

    /// Get the range values
    pub fn range_values(&self) -> &[T] {
        &self.range_values
    }

    /// Get the number of output categories
    pub fn len(&self) -> usize {
        self.range_values.len()
    }

    /// Check if the scale has no range values
    pub fn is_empty(&self) -> bool {
        self.range_values.is_empty()
    }

    /// Get the computed threshold values that separate segments
    ///
    /// For n range values, there are n-1 thresholds.
    ///
    /// # Example
    /// ```
    /// use makepad_d3::scale::QuantizeScale;
    ///
    /// let scale = QuantizeScale::new()
    ///     .domain(0.0, 100.0)
    ///     .range(vec!["A", "B", "C", "D"]);
    ///
    /// // 4 values = 3 thresholds at 25, 50, 75
    /// let thresholds = scale.thresholds();
    /// assert_eq!(thresholds.len(), 3);
    /// ```
    pub fn thresholds(&self) -> &[f64] {
        &self.thresholds
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

        // Binary search for the appropriate threshold
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

        lo
    }

    /// Get the domain extent [x0, x1] that maps to a specific range index
    ///
    /// # Example
    /// ```
    /// use makepad_d3::scale::QuantizeScale;
    ///
    /// let scale = QuantizeScale::new()
    ///     .domain(0.0, 100.0)
    ///     .range(vec!["A", "B", "C", "D"]);
    ///
    /// // Get the extent for category "B" (index 1)
    /// let (x0, x1) = scale.invert_extent(1);
    /// assert!((x0 - 25.0).abs() < 0.01);
    /// assert!((x1 - 50.0).abs() < 0.01);
    /// ```
    pub fn invert_extent(&self, index: usize) -> (f64, f64) {
        if self.range_values.is_empty() || index >= self.range_values.len() {
            return (self.domain_min, self.domain_max);
        }

        let x0 = if index == 0 {
            self.domain_min
        } else {
            self.thresholds[index - 1]
        };

        let x1 = if index >= self.thresholds.len() {
            self.domain_max
        } else {
            self.thresholds[index]
        };

        (x0, x1)
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

    /// Recalculate thresholds when domain or range changes
    fn rescale(&mut self) {
        let n = self.range_values.len();
        if n == 0 {
            self.thresholds.clear();
            return;
        }

        // n range values means n-1 thresholds
        self.thresholds = Vec::with_capacity(n.saturating_sub(1));

        let domain_span = self.domain_max - self.domain_min;
        let step = domain_span / n as f64;

        for i in 1..n {
            self.thresholds.push(self.domain_min + step * i as f64);
        }
    }
}

impl<T: Clone> Default for QuantizeScale<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// QuantizeScale with f64 range values can implement the Scale trait
impl Scale for QuantizeScale<f64> {
    fn scale_type(&self) -> &'static str {
        "quantize"
    }

    fn set_domain(&mut self, min: f64, max: f64) {
        self.domain_min = min;
        self.domain_max = max;
        self.rescale();
    }

    fn set_range(&mut self, start: f64, end: f64) {
        // For Scale trait compatibility, create a default 2-value range
        self.range_values = vec![start, end];
        self.rescale();
    }

    fn domain(&self) -> (f64, f64) {
        (self.domain_min, self.domain_max)
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
            (x0 + x1) / 2.0
        } else {
            self.domain_min
        }
    }

    fn ticks(&self, options: &TickOptions) -> Vec<Tick> {
        // Generate ticks at threshold boundaries
        let mut ticks = Vec::with_capacity(self.thresholds.len() + 2);

        // Include domain bounds if requested
        if options.include_bounds {
            ticks.push(
                Tick::new(self.domain_min, format!("{:.2}", self.domain_min))
                    .with_position(self.domain_min),
            );
        }

        for (i, &threshold) in self.thresholds.iter().enumerate() {
            if ticks.len() < options.max_count {
                ticks.push(
                    Tick::new(threshold, format!("{:.2}", threshold)).with_position(threshold),
                );
            }
            let _ = i; // suppress unused warning
        }

        if options.include_bounds && ticks.len() < options.max_count {
            ticks.push(
                Tick::new(self.domain_max, format!("{:.2}", self.domain_max))
                    .with_position(self.domain_max),
            );
        }

        ticks
    }

    fn copy_from(&mut self, other: &Self) {
        self.domain_min = other.domain_min;
        self.domain_max = other.domain_max;
        self.range_values = other.range_values.clone();
        self.thresholds = other.thresholds.clone();
    }

    fn clone_box(&self) -> Box<dyn Scale> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantize_scale_new() {
        let scale: QuantizeScale<&str> = QuantizeScale::new();
        assert!(scale.is_empty());
        assert_eq!(scale.get_domain(), (0.0, 1.0));
    }

    #[test]
    fn test_quantize_scale_domain_range() {
        let scale = QuantizeScale::new()
            .domain(0.0, 100.0)
            .range(vec!["A", "B", "C", "D"]);

        assert_eq!(scale.get_domain(), (0.0, 100.0));
        assert_eq!(scale.len(), 4);
    }

    #[test]
    fn test_quantize_scale_thresholds() {
        let scale = QuantizeScale::new()
            .domain(0.0, 100.0)
            .range(vec!["A", "B", "C", "D"]);

        let thresholds = scale.thresholds();
        assert_eq!(thresholds.len(), 3);
        assert!((thresholds[0] - 25.0).abs() < 0.01);
        assert!((thresholds[1] - 50.0).abs() < 0.01);
        assert!((thresholds[2] - 75.0).abs() < 0.01);
    }

    #[test]
    fn test_quantize_scale_basic_mapping() {
        let scale = QuantizeScale::new()
            .domain(0.0, 100.0)
            .range(vec!["low", "medium", "high"]);

        // Domain split into 3 equal segments: 0-33.3, 33.3-66.6, 66.6-100
        assert_eq!(scale.scale_to_value(0.0), Some(&"low"));
        assert_eq!(scale.scale_to_value(10.0), Some(&"low"));
        assert_eq!(scale.scale_to_value(33.0), Some(&"low"));
        assert_eq!(scale.scale_to_value(34.0), Some(&"medium"));
        assert_eq!(scale.scale_to_value(50.0), Some(&"medium"));
        assert_eq!(scale.scale_to_value(66.0), Some(&"medium"));
        assert_eq!(scale.scale_to_value(67.0), Some(&"high"));
        assert_eq!(scale.scale_to_value(100.0), Some(&"high"));
    }

    #[test]
    fn test_quantize_scale_to_index() {
        let scale = QuantizeScale::new()
            .domain(0.0, 100.0)
            .range(vec!["A", "B", "C", "D"]);

        assert_eq!(scale.scale_to_index(0.0), 0);
        assert_eq!(scale.scale_to_index(24.0), 0);
        assert_eq!(scale.scale_to_index(25.0), 1);
        assert_eq!(scale.scale_to_index(49.0), 1);
        assert_eq!(scale.scale_to_index(50.0), 2);
        assert_eq!(scale.scale_to_index(74.0), 2);
        assert_eq!(scale.scale_to_index(75.0), 3);
        assert_eq!(scale.scale_to_index(100.0), 3);
    }

    #[test]
    fn test_quantize_scale_invert_extent() {
        let scale = QuantizeScale::new()
            .domain(0.0, 100.0)
            .range(vec!["A", "B", "C", "D"]);

        let (x0, x1) = scale.invert_extent(0);
        assert!((x0 - 0.0).abs() < 0.01);
        assert!((x1 - 25.0).abs() < 0.01);

        let (x0, x1) = scale.invert_extent(1);
        assert!((x0 - 25.0).abs() < 0.01);
        assert!((x1 - 50.0).abs() < 0.01);

        let (x0, x1) = scale.invert_extent(2);
        assert!((x0 - 50.0).abs() < 0.01);
        assert!((x1 - 75.0).abs() < 0.01);

        let (x0, x1) = scale.invert_extent(3);
        assert!((x0 - 75.0).abs() < 0.01);
        assert!((x1 - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_quantize_scale_invert_extent_value() {
        let scale = QuantizeScale::new()
            .domain(0.0, 100.0)
            .range(vec!["A", "B", "C"]);

        let extent = scale.invert_extent_value(&"B");
        assert!(extent.is_some());
        let (x0, x1) = extent.unwrap();
        assert!((x0 - 33.33).abs() < 0.1);
        assert!((x1 - 66.66).abs() < 0.1);

        assert!(scale.invert_extent_value(&"Z").is_none());
    }

    #[test]
    fn test_quantize_scale_edge_values() {
        let scale = QuantizeScale::new()
            .domain(0.0, 100.0)
            .range(vec!["A", "B"]);

        // Two values: threshold at 50
        assert_eq!(scale.scale_to_value(0.0), Some(&"A"));
        assert_eq!(scale.scale_to_value(49.9), Some(&"A"));
        assert_eq!(scale.scale_to_value(50.0), Some(&"B"));
        assert_eq!(scale.scale_to_value(100.0), Some(&"B"));
    }

    #[test]
    fn test_quantize_scale_single_value() {
        let scale = QuantizeScale::new().domain(0.0, 100.0).range(vec!["only"]);

        // Single value: everything maps to it
        assert_eq!(scale.thresholds().len(), 0);
        assert_eq!(scale.scale_to_value(0.0), Some(&"only"));
        assert_eq!(scale.scale_to_value(50.0), Some(&"only"));
        assert_eq!(scale.scale_to_value(100.0), Some(&"only"));
    }

    #[test]
    fn test_quantize_scale_empty_range() {
        let scale: QuantizeScale<&str> = QuantizeScale::new().domain(0.0, 100.0);

        assert!(scale.is_empty());
        assert_eq!(scale.scale_to_value(50.0), None);
    }

    #[test]
    fn test_quantize_scale_negative_domain() {
        let scale = QuantizeScale::new()
            .domain(-100.0, 100.0)
            .range(vec!["negative", "zero", "positive"]);

        assert_eq!(scale.scale_to_value(-100.0), Some(&"negative"));
        assert_eq!(scale.scale_to_value(-50.0), Some(&"negative"));
        assert_eq!(scale.scale_to_value(-33.0), Some(&"zero"));
        assert_eq!(scale.scale_to_value(0.0), Some(&"zero"));
        assert_eq!(scale.scale_to_value(33.0), Some(&"zero"));
        assert_eq!(scale.scale_to_value(34.0), Some(&"positive"));
        assert_eq!(scale.scale_to_value(100.0), Some(&"positive"));
    }

    #[test]
    fn test_quantize_scale_with_numbers() {
        let scale = QuantizeScale::new()
            .domain(0.0, 100.0)
            .range(vec![0.0, 0.25, 0.5, 0.75, 1.0]);

        assert_eq!(scale.scale_to_value(10.0), Some(&0.0));
        assert_eq!(scale.scale_to_value(30.0), Some(&0.25));
        assert_eq!(scale.scale_to_value(50.0), Some(&0.5));
        assert_eq!(scale.scale_to_value(70.0), Some(&0.75));
        assert_eq!(scale.scale_to_value(90.0), Some(&1.0));
    }

    #[test]
    fn test_quantize_scale_f64_scale_trait() {
        let scale = QuantizeScale::new()
            .domain(0.0, 100.0)
            .range(vec![10.0, 20.0, 30.0, 40.0]);

        // Test Scale trait methods - need to use trait method via explicit call
        assert_eq!(scale.scale_type(), "quantize");
        assert_eq!(scale.get_domain(), (0.0, 100.0));
        assert_eq!(Scale::range(&scale), (10.0, 40.0));

        assert!((scale.scale(10.0) - 10.0).abs() < 0.01);
        assert!((scale.scale(30.0) - 20.0).abs() < 0.01);
        assert!((scale.scale(60.0) - 30.0).abs() < 0.01);
        assert!((scale.scale(90.0) - 40.0).abs() < 0.01);
    }

    #[test]
    fn test_quantize_scale_clone_box() {
        let scale = QuantizeScale::new()
            .domain(0.0, 100.0)
            .range(vec![1.0, 2.0, 3.0]);

        let boxed: Box<dyn Scale> = scale.clone_box();
        assert_eq!(boxed.scale_type(), "quantize");
    }

    #[test]
    fn test_quantize_scale_many_values() {
        let scale = QuantizeScale::new()
            .domain(0.0, 100.0)
            .range(vec!["A", "B", "C", "D", "E", "F", "G", "H", "I", "J"]);

        assert_eq!(scale.thresholds().len(), 9);
        assert_eq!(scale.scale_to_value(5.0), Some(&"A"));
        assert_eq!(scale.scale_to_value(15.0), Some(&"B"));
        assert_eq!(scale.scale_to_value(95.0), Some(&"J"));
    }

    #[test]
    fn test_quantize_scale_out_of_bounds() {
        let scale = QuantizeScale::new()
            .domain(0.0, 100.0)
            .range(vec!["A", "B", "C"]);

        // Values below domain map to first category
        assert_eq!(scale.scale_to_value(-50.0), Some(&"A"));
        // Values above domain map to last category
        assert_eq!(scale.scale_to_value(150.0), Some(&"C"));
    }

    #[test]
    fn test_quantize_scale_inverted_domain() {
        let scale = QuantizeScale::new()
            .domain(100.0, 0.0)
            .range(vec!["high", "medium", "low"]);

        // With inverted domain (100 to 0), thresholds are at 66.6 and 33.3
        // But since domain_min > domain_max, the step is negative
        // This creates thresholds in descending order
        let thresholds = scale.thresholds();
        assert_eq!(thresholds.len(), 2);
    }

    #[test]
    fn test_quantize_scale_colors() {
        // Common use case: mapping values to color indices
        let scale = QuantizeScale::new().domain(0.0, 1.0).range(vec![
            "#f7fbff", "#deebf7", "#c6dbef", "#9ecae1", "#6baed6", "#4292c6", "#2171b5", "#084594",
        ]);

        assert_eq!(scale.scale_to_value(0.0), Some(&"#f7fbff"));
        assert_eq!(scale.scale_to_value(0.5), Some(&"#6baed6"));
        assert_eq!(scale.scale_to_value(1.0), Some(&"#084594"));
    }
}

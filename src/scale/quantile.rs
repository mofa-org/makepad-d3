//! Quantile scale implementation
//!
//! Quantile scales map a sampled continuous domain to a discrete range by
//! computing quantiles from the data. Unlike quantize scales which create
//! equal-sized domain segments, quantile scales create segments with equal
//! numbers of data points.

use super::traits::{Scale, Tick, TickOptions};

/// Scale that maps continuous input to discrete output based on data quantiles
///
/// Quantile scales divide the input domain based on the distribution of sample
/// data, ensuring each output category contains approximately the same number
/// of data points. This is useful when you want equal representation in each
/// bucket regardless of the value distribution.
///
/// # D3.js Equivalent
/// This is equivalent to `d3.scaleQuantile()` in D3.js.
///
/// # Difference from QuantizeScale
/// - **QuantizeScale**: Equal-sized domain segments (e.g., 0-25, 25-50, 50-75, 75-100)
/// - **QuantileScale**: Equal-count data segments (based on actual data distribution)
///
/// # Example
/// ```
/// use makepad_d3::scale::QuantileScale;
///
/// // With skewed data, quantile ensures equal representation
/// let scale = QuantileScale::new()
///     .domain(vec![1.0, 2.0, 3.0, 4.0, 5.0, 100.0])
///     .range(vec!["low", "high"]);
///
/// // Threshold is at median (3.5), not midpoint (50.5)
/// assert_eq!(scale.scale_to_value(3.0), Some(&"low"));
/// assert_eq!(scale.scale_to_value(4.0), Some(&"high"));
/// ```
#[derive(Clone, Debug)]
pub struct QuantileScale<T: Clone> {
    /// Sorted domain sample data
    domain_data: Vec<f64>,
    /// Discrete output values
    range_values: Vec<T>,
    /// Computed quantile thresholds
    thresholds: Vec<f64>,
}

impl<T: Clone> QuantileScale<T> {
    /// Create a new quantile scale with default settings
    pub fn new() -> Self {
        Self {
            domain_data: Vec::new(),
            range_values: Vec::new(),
            thresholds: Vec::new(),
        }
    }

    /// Set the domain from sample data
    ///
    /// The data will be sorted internally. NaN values are ignored.
    ///
    /// # Example
    /// ```
    /// use makepad_d3::scale::QuantileScale;
    ///
    /// let scale: QuantileScale<&str> = QuantileScale::new()
    ///     .with_domain(vec![10.0, 20.0, 30.0, 40.0, 50.0]);
    /// ```
    pub fn with_domain(mut self, data: Vec<f64>) -> Self {
        self.set_domain_data(data);
        self
    }

    /// Set the domain from sample data (deprecated)
    #[deprecated(
        since = "0.2.0",
        note = "Use `with_domain` instead for consistent builder pattern"
    )]
    pub fn domain(self, data: Vec<f64>) -> Self {
        self.with_domain(data)
    }

    /// Set the domain data
    pub fn set_domain_data(&mut self, data: Vec<f64>) {
        // Filter out NaN values and sort
        self.domain_data = data.into_iter().filter(|x| !x.is_nan()).collect();
        self.domain_data.sort_by(|a, b| a.partial_cmp(b).unwrap());
        self.rescale();
    }

    /// Get the sorted domain data
    pub fn domain_data(&self) -> &[f64] {
        &self.domain_data
    }

    /// Get the domain extent (min, max)
    pub fn domain_extent(&self) -> (f64, f64) {
        if self.domain_data.is_empty() {
            (0.0, 1.0)
        } else {
            (
                self.domain_data[0],
                self.domain_data[self.domain_data.len() - 1],
            )
        }
    }

    /// Set the discrete output range values
    ///
    /// The number of range values determines how many quantiles
    /// the domain is divided into.
    ///
    /// # Example
    /// ```
    /// use makepad_d3::scale::QuantileScale;
    ///
    /// let scale = QuantileScale::new()
    ///     .with_domain(vec![1.0, 2.0, 3.0, 4.0])
    ///     .with_range(vec!["Q1", "Q2", "Q3", "Q4"]);
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

    /// Get the number of output categories (quantiles)
    pub fn len(&self) -> usize {
        self.range_values.len()
    }

    /// Check if the scale has no range values
    pub fn is_empty(&self) -> bool {
        self.range_values.is_empty()
    }

    /// Get the computed quantile thresholds
    ///
    /// These are the values that separate quantiles, computed from
    /// the actual data distribution.
    ///
    /// # Example
    /// ```
    /// use makepad_d3::scale::QuantileScale;
    ///
    /// let scale = QuantileScale::new()
    ///     .domain(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0])
    ///     .range(vec!["Q1", "Q2", "Q3", "Q4"]);
    ///
    /// // Thresholds divide data into 4 equal parts
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
    /// Returns the range of input values that map to the given quantile.
    pub fn invert_extent(&self, index: usize) -> (f64, f64) {
        if self.domain_data.is_empty() || self.range_values.is_empty() {
            return (f64::NAN, f64::NAN);
        }

        if index >= self.range_values.len() {
            return (f64::NAN, f64::NAN);
        }

        let x0 = if index == 0 {
            self.domain_data[0]
        } else if index - 1 < self.thresholds.len() {
            self.thresholds[index - 1]
        } else {
            self.domain_data[self.domain_data.len() - 1]
        };

        let x1 = if index < self.thresholds.len() {
            self.thresholds[index]
        } else {
            self.domain_data[self.domain_data.len() - 1]
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

    /// Compute the p-th quantile of the sorted data
    fn quantile_of_sorted(&self, p: f64) -> f64 {
        if self.domain_data.is_empty() {
            return f64::NAN;
        }

        if p <= 0.0 || self.domain_data.len() < 2 {
            return self.domain_data[0];
        }

        if p >= 1.0 {
            return self.domain_data[self.domain_data.len() - 1];
        }

        let n = self.domain_data.len();
        let i = (n - 1) as f64 * p;
        let i0 = i.floor() as usize;
        let i1 = i.ceil() as usize;
        let t = i - i0 as f64;

        let v0 = self.domain_data[i0];
        let v1 = self.domain_data[i1.min(n - 1)];

        v0 + t * (v1 - v0)
    }

    /// Recalculate thresholds when domain or range changes
    fn rescale(&mut self) {
        let n = self.range_values.len();
        if n == 0 || self.domain_data.is_empty() {
            self.thresholds.clear();
            return;
        }

        // Compute n-1 thresholds at quantile positions
        self.thresholds = Vec::with_capacity(n.saturating_sub(1));

        for i in 1..n {
            let p = i as f64 / n as f64;
            self.thresholds.push(self.quantile_of_sorted(p));
        }
    }
}

impl<T: Clone> Default for QuantileScale<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// QuantileScale with f64 range values can implement the Scale trait
impl Scale for QuantileScale<f64> {
    fn scale_type(&self) -> &'static str {
        "quantile"
    }

    fn set_domain(&mut self, min: f64, max: f64) {
        // For Scale trait compatibility, create a simple 2-value domain
        self.domain_data = vec![min, max];
        self.domain_data.sort_by(|a, b| a.partial_cmp(b).unwrap());
        self.rescale();
    }

    fn set_range(&mut self, start: f64, end: f64) {
        // For Scale trait compatibility, create a default 2-value range
        self.range_values = vec![start, end];
        self.rescale();
    }

    fn domain(&self) -> (f64, f64) {
        self.domain_extent()
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
            if x0.is_nan() || x1.is_nan() {
                return f64::NAN;
            }
            (x0 + x1) / 2.0
        } else {
            f64::NAN
        }
    }

    fn ticks(&self, options: &TickOptions) -> Vec<Tick> {
        // Generate ticks at threshold boundaries
        let mut ticks = Vec::with_capacity(self.thresholds.len() + 2);

        let (domain_min, domain_max) = self.domain_extent();

        // Include domain bounds if requested
        if options.include_bounds && !domain_min.is_nan() {
            ticks.push(
                Tick::new(domain_min, format!("{:.2}", domain_min)).with_position(domain_min),
            );
        }

        for &threshold in &self.thresholds {
            if ticks.len() < options.max_count {
                ticks.push(
                    Tick::new(threshold, format!("{:.2}", threshold)).with_position(threshold),
                );
            }
        }

        if options.include_bounds && ticks.len() < options.max_count && !domain_max.is_nan() {
            ticks.push(
                Tick::new(domain_max, format!("{:.2}", domain_max)).with_position(domain_max),
            );
        }

        ticks
    }

    fn copy_from(&mut self, other: &Self) {
        self.domain_data = other.domain_data.clone();
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
    fn test_quantile_scale_new() {
        let scale: QuantileScale<&str> = QuantileScale::new();
        assert!(scale.is_empty());
        assert!(scale.domain_data().is_empty());
    }

    #[test]
    fn test_quantile_scale_domain_sorted() {
        let scale: QuantileScale<&str> = QuantileScale::new().domain(vec![5.0, 1.0, 3.0, 2.0, 4.0]);

        // Data should be sorted internally
        assert_eq!(scale.domain_data(), &[1.0, 2.0, 3.0, 4.0, 5.0]);
    }

    #[test]
    fn test_quantile_scale_filters_nan() {
        let scale: QuantileScale<&str> =
            QuantileScale::new().domain(vec![1.0, f64::NAN, 3.0, f64::NAN, 5.0]);

        assert_eq!(scale.domain_data(), &[1.0, 3.0, 5.0]);
    }

    #[test]
    fn test_quantile_scale_basic_mapping() {
        let scale = QuantileScale::new()
            .domain(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0])
            .range(vec!["Q1", "Q2", "Q3", "Q4"]);

        // 8 values divided into 4 quantiles: 2 values each
        // Q1: 1, 2  Q2: 3, 4  Q3: 5, 6  Q4: 7, 8
        assert_eq!(scale.scale_to_value(1.0), Some(&"Q1"));
        assert_eq!(scale.scale_to_value(2.0), Some(&"Q1"));
        assert_eq!(scale.scale_to_value(3.0), Some(&"Q2"));
        assert_eq!(scale.scale_to_value(4.0), Some(&"Q2"));
        assert_eq!(scale.scale_to_value(5.0), Some(&"Q3"));
        assert_eq!(scale.scale_to_value(6.0), Some(&"Q3"));
        assert_eq!(scale.scale_to_value(7.0), Some(&"Q4"));
        assert_eq!(scale.scale_to_value(8.0), Some(&"Q4"));
    }

    #[test]
    fn test_quantile_scale_thresholds() {
        let scale = QuantileScale::new()
            .domain(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0])
            .range(vec!["Q1", "Q2", "Q3", "Q4"]);

        let thresholds = scale.thresholds();
        assert_eq!(thresholds.len(), 3);

        // Thresholds should be at 25%, 50%, 75% quantiles
        // For [1,2,3,4,5,6,7,8]: Q1=2.75, Q2=4.5, Q3=6.25
        assert!((thresholds[0] - 2.75).abs() < 0.01);
        assert!((thresholds[1] - 4.5).abs() < 0.01);
        assert!((thresholds[2] - 6.25).abs() < 0.01);
    }

    #[test]
    fn test_quantile_vs_quantize_skewed_data() {
        // This test demonstrates the key difference between quantile and quantize
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 100.0];

        let quantile_scale = QuantileScale::new()
            .domain(data.clone())
            .range(vec!["low", "high"]);

        // Quantile: threshold is at median (~3.5), so 3 values on each side
        // Values 1,2,3 -> low, values 4,5,100 -> high
        assert_eq!(quantile_scale.scale_to_value(3.0), Some(&"low"));
        assert_eq!(quantile_scale.scale_to_value(4.0), Some(&"high"));

        // Quantize would have threshold at 50.5 (midpoint of 1-100)
        // So 1,2,3,4,5 -> low, only 100 -> high
        // This shows quantile is better for skewed data
    }

    #[test]
    fn test_quantile_scale_two_values() {
        let scale = QuantileScale::new()
            .domain(vec![10.0, 20.0, 30.0, 40.0])
            .range(vec!["A", "B"]);

        // Median is at 25 (between 20 and 30)
        assert_eq!(scale.thresholds().len(), 1);
        assert!((scale.thresholds()[0] - 25.0).abs() < 0.01);

        assert_eq!(scale.scale_to_value(10.0), Some(&"A"));
        assert_eq!(scale.scale_to_value(20.0), Some(&"A"));
        assert_eq!(scale.scale_to_value(30.0), Some(&"B"));
        assert_eq!(scale.scale_to_value(40.0), Some(&"B"));
    }

    #[test]
    fn test_quantile_scale_single_value() {
        let scale = QuantileScale::new()
            .domain(vec![1.0, 2.0, 3.0, 4.0])
            .range(vec!["only"]);

        // Single output: everything maps to it
        assert_eq!(scale.thresholds().len(), 0);
        assert_eq!(scale.scale_to_value(1.0), Some(&"only"));
        assert_eq!(scale.scale_to_value(4.0), Some(&"only"));
    }

    #[test]
    fn test_quantile_scale_empty() {
        let scale: QuantileScale<&str> = QuantileScale::new().range(vec!["A", "B"]);

        assert!(scale.domain_data().is_empty());
        assert!(scale.thresholds().is_empty());
    }

    #[test]
    fn test_quantile_scale_invert_extent() {
        let scale = QuantileScale::new()
            .domain(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0])
            .range(vec!["Q1", "Q2", "Q3", "Q4"]);

        let (x0, x1) = scale.invert_extent(0);
        assert!((x0 - 1.0).abs() < 0.01);
        assert!((x1 - 2.75).abs() < 0.01);

        let (x0, x1) = scale.invert_extent(1);
        assert!((x0 - 2.75).abs() < 0.01);
        assert!((x1 - 4.5).abs() < 0.01);

        let (x0, x1) = scale.invert_extent(3);
        assert!((x0 - 6.25).abs() < 0.01);
        assert!((x1 - 8.0).abs() < 0.01);
    }

    #[test]
    fn test_quantile_scale_invert_extent_value() {
        let scale = QuantileScale::new()
            .domain(vec![0.0, 25.0, 50.0, 75.0, 100.0])
            .range(vec!["A", "B"]);

        let extent = scale.invert_extent_value(&"A");
        assert!(extent.is_some());

        assert!(scale.invert_extent_value(&"Z").is_none());
    }

    #[test]
    fn test_quantile_scale_with_duplicates() {
        let scale = QuantileScale::new()
            .domain(vec![1.0, 1.0, 1.0, 2.0, 2.0, 2.0])
            .range(vec!["low", "high"]);

        // Median is between the 1s and 2s
        assert_eq!(scale.scale_to_value(1.0), Some(&"low"));
        assert_eq!(scale.scale_to_value(2.0), Some(&"high"));
    }

    #[test]
    fn test_quantile_scale_uniform_data() {
        let scale = QuantileScale::new()
            .domain(vec![
                0.0, 10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0,
            ])
            .range(vec!["A", "B", "C", "D", "E"]);

        // 10 uniformly distributed values into 5 quantiles
        // Each quantile should have ~2 values
        assert_eq!(scale.thresholds().len(), 4);

        assert_eq!(scale.scale_to_value(0.0), Some(&"A"));
        assert_eq!(scale.scale_to_value(20.0), Some(&"B"));
        assert_eq!(scale.scale_to_value(40.0), Some(&"C"));
        assert_eq!(scale.scale_to_value(60.0), Some(&"D"));
        assert_eq!(scale.scale_to_value(80.0), Some(&"E"));
    }

    #[test]
    fn test_quantile_scale_f64_scale_trait() {
        let scale = QuantileScale::new()
            .domain(vec![0.0, 25.0, 50.0, 75.0, 100.0])
            .range(vec![10.0, 20.0, 30.0, 40.0, 50.0]);

        assert_eq!(scale.scale_type(), "quantile");
        assert_eq!(scale.domain_extent(), (0.0, 100.0));
        assert_eq!(Scale::range(&scale), (10.0, 50.0));
    }

    #[test]
    fn test_quantile_scale_clone_box() {
        let scale = QuantileScale::new()
            .domain(vec![1.0, 2.0, 3.0])
            .range(vec![0.0, 1.0]);

        let boxed: Box<dyn Scale> = scale.clone_box();
        assert_eq!(boxed.scale_type(), "quantile");
    }

    #[test]
    fn test_quantile_scale_out_of_range() {
        let scale = QuantileScale::new()
            .domain(vec![10.0, 20.0, 30.0, 40.0])
            .range(vec!["low", "high"]);

        // Values below domain map to first category
        assert_eq!(scale.scale_to_value(0.0), Some(&"low"));
        // Values above domain map to last category
        assert_eq!(scale.scale_to_value(100.0), Some(&"high"));
    }

    #[test]
    fn test_quantile_scale_percentiles() {
        // Common use case: mapping to percentile ranges
        let data: Vec<f64> = (1..=100).map(|x| x as f64).collect();

        let scale = QuantileScale::new()
            .domain(data)
            .range(vec!["0-25%", "25-50%", "50-75%", "75-100%"]);

        assert_eq!(scale.scale_to_value(10.0), Some(&"0-25%"));
        assert_eq!(scale.scale_to_value(30.0), Some(&"25-50%"));
        assert_eq!(scale.scale_to_value(60.0), Some(&"50-75%"));
        assert_eq!(scale.scale_to_value(90.0), Some(&"75-100%"));
    }

    #[test]
    fn test_quantile_scale_colors() {
        let data: Vec<f64> = (0..100).map(|x| x as f64).collect();

        let scale = QuantileScale::new().domain(data).range(vec![
            "#f7fbff", "#deebf7", "#c6dbef", "#9ecae1", "#6baed6", "#4292c6", "#2171b5", "#084594",
        ]);

        assert_eq!(scale.scale_to_value(5.0), Some(&"#f7fbff"));
        assert_eq!(scale.scale_to_value(95.0), Some(&"#084594"));
    }
}

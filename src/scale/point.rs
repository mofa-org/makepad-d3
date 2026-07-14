//! Point scale implementation
//!
//! Point scales are a variant of band scales with zero bandwidth.
//! They are useful for scatter plots and dot plots with discrete categories.

use super::traits::{DiscreteScale, Scale, Tick, TickOptions};

/// Scale for mapping discrete domain to evenly spaced points
///
/// Point scales are like band scales but with a bandwidth of zero.
/// They map discrete values to evenly spaced points within the range,
/// useful for scatter plots and ordinal axes.
///
/// # D3.js Equivalent
/// This is equivalent to `d3.scalePoint()` in D3.js.
///
/// # Example
/// ```
/// use makepad_d3::scale::{Scale, PointScale};
///
/// let scale = PointScale::new()
///     .domain(vec!["A", "B", "C", "D"])
///     .range(0.0, 300.0);
///
/// // Points are evenly distributed
/// assert!((scale.scale_category("A").unwrap() - 0.0).abs() < 1.0);
/// assert!((scale.scale_category("B").unwrap() - 100.0).abs() < 1.0);
/// assert!((scale.scale_category("C").unwrap() - 200.0).abs() < 1.0);
/// assert!((scale.scale_category("D").unwrap() - 300.0).abs() < 1.0);
/// ```
#[derive(Clone, Debug)]
pub struct PointScale {
    /// The discrete domain values
    domain_values: Vec<String>,
    /// Start of output range
    range_start: f64,
    /// End of output range
    range_end: f64,
    /// Padding at the edges as fraction of step (0-1)
    padding: f64,
    /// Alignment within outer padding (0-1)
    align: f64,
    /// Whether to round output values to integers
    round: bool,
    /// Cached computed step value
    cached_step: f64,
}

impl PointScale {
    /// Create a new point scale with default settings
    pub fn new() -> Self {
        let mut scale = Self {
            domain_values: Vec::new(),
            range_start: 0.0,
            range_end: 1.0,
            padding: 0.0,
            align: 0.5,
            round: false,
            cached_step: 0.0,
        };
        scale.rescale();
        scale
    }

    /// Set the domain (discrete categories)
    ///
    /// # Example
    /// ```
    /// use makepad_d3::scale::PointScale;
    ///
    /// let scale = PointScale::new()
    ///     .with_domain(vec!["Low", "Medium", "High"]);
    /// ```
    pub fn with_domain<S: Into<String>>(mut self, values: impl IntoIterator<Item = S>) -> Self {
        self.domain_values = values.into_iter().map(Into::into).collect();
        self.rescale();
        self
    }

    /// Set the domain (discrete categories)
    #[deprecated(
        since = "0.2.0",
        note = "Use `with_domain` instead for consistent builder pattern"
    )]
    pub fn domain<S: Into<String>>(self, values: impl IntoIterator<Item = S>) -> Self {
        self.with_domain(values)
    }

    /// Set the domain from a vector of strings
    pub fn set_domain_values(&mut self, values: Vec<String>) {
        self.domain_values = values;
        self.rescale();
    }

    /// Get the domain values
    pub fn domain_values(&self) -> &[String] {
        &self.domain_values
    }

    /// Set the output range
    ///
    /// # Example
    /// ```
    /// use makepad_d3::scale::PointScale;
    ///
    /// let scale = PointScale::new()
    ///     .with_domain(vec!["A", "B", "C"])
    ///     .with_range(0.0, 300.0);
    /// ```
    pub fn with_range(mut self, start: f64, end: f64) -> Self {
        self.range_start = start;
        self.range_end = end;
        self.rescale();
        self
    }

    /// Set the output range
    #[deprecated(
        since = "0.2.0",
        note = "Use `with_range` instead for consistent builder pattern"
    )]
    pub fn range(self, start: f64, end: f64) -> Self {
        self.with_range(start, end)
    }

    /// Set the outer padding as a fraction of the step size
    ///
    /// Padding of 0.5 means half a step of padding at each end,
    /// effectively centering the points within the range.
    ///
    /// # Example
    /// ```
    /// use makepad_d3::scale::PointScale;
    ///
    /// let scale = PointScale::new()
    ///     .with_domain(vec!["A", "B", "C"])
    ///     .with_range(0.0, 300.0)
    ///     .with_padding(0.5);  // Center points with half-step padding
    /// ```
    pub fn with_padding(mut self, padding: f64) -> Self {
        self.padding = padding.clamp(0.0, 1.0);
        self.rescale();
        self
    }

    /// Set the outer padding as a fraction of the step size
    #[deprecated(
        since = "0.2.0",
        note = "Use `with_padding` instead for consistent builder pattern"
    )]
    pub fn padding(self, padding: f64) -> Self {
        self.with_padding(padding)
    }

    /// Set the alignment within outer padding
    ///
    /// Alignment of 0 means points are left-aligned, 1 means right-aligned,
    /// and 0.5 (default) means centered.
    pub fn with_align(mut self, align: f64) -> Self {
        self.align = align.clamp(0.0, 1.0);
        self.rescale();
        self
    }

    /// Set the alignment within outer padding
    #[deprecated(
        since = "0.2.0",
        note = "Use `with_align` instead for consistent builder pattern"
    )]
    pub fn align(self, align: f64) -> Self {
        self.with_align(align)
    }

    /// Enable or disable rounding to pixel boundaries
    ///
    /// When enabled, positions are rounded to integers for crisper rendering.
    pub fn with_round(mut self, round: bool) -> Self {
        self.round = round;
        self.rescale();
        self
    }

    /// Enable or disable rounding to pixel boundaries
    #[deprecated(
        since = "0.2.0",
        note = "Use `with_round` instead for consistent builder pattern"
    )]
    pub fn round(self, round: bool) -> Self {
        self.with_round(round)
    }

    /// Get the number of points
    pub fn len(&self) -> usize {
        self.domain_values.len()
    }

    /// Check if the scale has no points
    pub fn is_empty(&self) -> bool {
        self.domain_values.is_empty()
    }

    /// Get the index for a domain value
    pub fn index_of(&self, value: &str) -> Option<usize> {
        self.domain_values.iter().position(|v| v == value)
    }

    /// Get the domain value at an index
    pub fn value_at(&self, index: usize) -> Option<&str> {
        self.domain_values.get(index).map(String::as_str)
    }

    /// Get the pixel position for a category by name
    ///
    /// Returns `None` if the category is not in the domain.
    pub fn scale_category(&self, value: &str) -> Option<f64> {
        self.index_of(value).map(|i| self.scale_index(i))
    }

    /// Get the pixel position for a category by index
    pub fn scale_index(&self, index: usize) -> f64 {
        if self.domain_values.is_empty() {
            return self.range_start;
        }

        if index >= self.domain_values.len() {
            return self.range_start;
        }

        // Calculate start position with padding and alignment
        let start = self.range_start + self.padding * self.cached_step * self.align * 2.0;
        let pos = start + index as f64 * self.cached_step;

        if self.round {
            pos.round()
        } else {
            pos
        }
    }

    /// Get the category at a pixel position (nearest point)
    pub fn invert_index(&self, pixel: f64) -> Option<usize> {
        if self.domain_values.is_empty() || self.cached_step == 0.0 {
            return if self.domain_values.is_empty() {
                None
            } else {
                Some(0)
            };
        }

        let start = self.range_start + self.padding * self.cached_step * self.align * 2.0;
        let relative = pixel - start;

        // Find nearest point
        let index = (relative / self.cached_step + 0.5).floor() as i64;
        let clamped = index.clamp(0, (self.domain_values.len() - 1) as i64) as usize;
        Some(clamped)
    }

    /// Get the category name at a pixel position (nearest point)
    pub fn invert(&self, pixel: f64) -> Option<&str> {
        self.invert_index(pixel)
            .and_then(|i| self.domain_values.get(i).map(String::as_str))
    }

    /// Recalculate cached values when scale parameters change
    fn rescale(&mut self) {
        let n = self.domain_values.len();
        if n == 0 {
            self.cached_step = 0.0;
            return;
        }

        let range = (self.range_end - self.range_start).abs();

        if n == 1 {
            // Single point goes at the center (considering alignment)
            self.cached_step = range;
            return;
        }

        // D3's formula for point scale:
        // step = range / (n - 1 + padding * 2)
        let divisor = (n - 1) as f64 + self.padding * 2.0;
        if divisor <= 0.0 {
            self.cached_step = 0.0;
            return;
        }

        self.cached_step = range / divisor;

        if self.round {
            self.cached_step = self.cached_step.floor();
        }
    }
}

impl Default for PointScale {
    fn default() -> Self {
        Self::new()
    }
}

impl Scale for PointScale {
    fn scale_type(&self) -> &'static str {
        "point"
    }

    fn set_domain(&mut self, _min: f64, _max: f64) {
        // Point scale uses discrete domain, not numeric
        // Use set_domain_values() or domain() instead
    }

    fn set_range(&mut self, start: f64, end: f64) {
        self.range_start = start;
        self.range_end = end;
        self.rescale();
    }

    fn domain(&self) -> (f64, f64) {
        (0.0, (self.domain_values.len().saturating_sub(1)) as f64)
    }

    fn range(&self) -> (f64, f64) {
        (self.range_start, self.range_end)
    }

    fn scale(&self, value: f64) -> f64 {
        self.scale_index(value.round().max(0.0) as usize)
    }

    fn invert(&self, pixel: f64) -> f64 {
        self.invert_index(pixel).unwrap_or(0) as f64
    }

    fn ticks(&self, options: &TickOptions) -> Vec<Tick> {
        let mut ticks = Vec::with_capacity(self.domain_values.len());

        // Calculate step for skipping labels if too many
        let step = if self.domain_values.len() > options.max_count && options.max_count > 0 {
            (self.domain_values.len() as f64 / options.max_count as f64).ceil() as usize
        } else {
            1
        };

        for (i, label) in self.domain_values.iter().enumerate() {
            if i % step == 0 {
                let pos = self.scale_index(i);
                ticks.push(Tick::new(i as f64, label.clone()).with_position(pos));
            }
        }

        ticks
    }

    fn copy_from(&mut self, other: &Self) {
        self.domain_values = other.domain_values.clone();
        self.range_start = other.range_start;
        self.range_end = other.range_end;
        self.padding = other.padding;
        self.align = other.align;
        self.round = other.round;
        self.rescale();
    }

    fn clone_box(&self) -> Box<dyn Scale> {
        Box::new(self.clone())
    }
}

impl DiscreteScale for PointScale {
    /// Point scales have zero bandwidth by definition
    fn bandwidth(&self) -> f64 {
        0.0
    }

    fn step(&self) -> f64 {
        self.cached_step
    }

    fn set_padding(&mut self, padding: f64) {
        self.padding = padding.clamp(0.0, 1.0);
        self.rescale();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_scale_new() {
        let scale = PointScale::new();
        assert!(scale.is_empty());
        assert_eq!(scale.bandwidth(), 0.0);
    }

    #[test]
    fn test_point_scale_domain() {
        let scale = PointScale::new().domain(vec!["A", "B", "C", "D"]);

        assert_eq!(scale.len(), 4);
        assert_eq!(scale.index_of("A"), Some(0));
        assert_eq!(scale.index_of("B"), Some(1));
        assert_eq!(scale.index_of("C"), Some(2));
        assert_eq!(scale.index_of("D"), Some(3));
        assert_eq!(scale.index_of("E"), None);
    }

    #[test]
    fn test_point_scale_basic_mapping() {
        let scale = PointScale::new()
            .domain(vec!["A", "B", "C", "D"])
            .range(0.0, 300.0);

        // Without padding: step = 300/3 = 100
        // Points at: 0, 100, 200, 300
        assert!((scale.step() - 100.0).abs() < 0.01);
        assert!((scale.scale_index(0) - 0.0).abs() < 0.01);
        assert!((scale.scale_index(1) - 100.0).abs() < 0.01);
        assert!((scale.scale_index(2) - 200.0).abs() < 0.01);
        assert!((scale.scale_index(3) - 300.0).abs() < 0.01);
    }

    #[test]
    fn test_point_scale_bandwidth_is_zero() {
        let scale = PointScale::new()
            .domain(vec!["A", "B", "C"])
            .range(0.0, 200.0);

        // Point scales always have zero bandwidth
        assert_eq!(scale.bandwidth(), 0.0);
    }

    #[test]
    fn test_point_scale_with_padding() {
        let scale = PointScale::new()
            .domain(vec!["A", "B", "C"])
            .range(0.0, 200.0)
            .padding(0.5);

        // With padding=0.5, formula: step = 200 / (3 - 1 + 0.5 * 2) = 200 / 3 = 66.67
        // First point at: 0 + 0.5 * step * 0.5 * 2 = 0.5 * step
        let step = scale.step();
        let first_pos = scale.scale_index(0);

        // First point should be offset from start
        assert!(first_pos > 0.0);
        assert!((first_pos - step * 0.5).abs() < 0.01);
    }

    #[test]
    fn test_point_scale_scale_category() {
        let scale = PointScale::new()
            .domain(vec!["Low", "Medium", "High"])
            .range(0.0, 200.0);

        assert!(scale.scale_category("Low").is_some());
        assert!(scale.scale_category("Medium").is_some());
        assert!(scale.scale_category("High").is_some());
        assert!(scale.scale_category("VeryHigh").is_none());
    }

    #[test]
    fn test_point_scale_invert() {
        let scale = PointScale::new()
            .domain(vec!["A", "B", "C", "D"])
            .range(0.0, 300.0);

        // Points at 0, 100, 200, 300
        // Invert finds nearest point
        assert_eq!(scale.invert(0.0), Some("A"));
        assert_eq!(scale.invert(40.0), Some("A"));
        assert_eq!(scale.invert(60.0), Some("B"));
        assert_eq!(scale.invert(100.0), Some("B"));
        assert_eq!(scale.invert(140.0), Some("B"));
        // At exactly midpoint (150), rounds to C (next point)
        assert_eq!(scale.invert(150.0), Some("C"));
        assert_eq!(scale.invert(160.0), Some("C"));
        assert_eq!(scale.invert(300.0), Some("D"));
    }

    #[test]
    fn test_point_scale_round() {
        let scale = PointScale::new()
            .domain(vec!["A", "B", "C"])
            .range(0.0, 100.0)
            .round(true);

        // With rounding, step should be an integer
        let step = scale.step();
        assert_eq!(step, step.floor());

        // Positions should be integers
        for i in 0..3 {
            let pos = scale.scale_index(i);
            assert_eq!(pos, pos.round());
        }
    }

    #[test]
    fn test_point_scale_ticks() {
        let scale = PointScale::new()
            .domain(vec!["A", "B", "C", "D"])
            .range(0.0, 300.0);

        let ticks = scale.ticks(&TickOptions::default());

        assert_eq!(ticks.len(), 4);
        assert_eq!(ticks[0].label, "A");
        assert_eq!(ticks[3].label, "D");

        // Tick positions should match scale positions
        assert!((ticks[0].position - 0.0).abs() < 0.01);
        assert!((ticks[1].position - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_point_scale_ticks_max_count() {
        let scale = PointScale::new()
            .domain(vec!["A", "B", "C", "D", "E", "F", "G", "H", "I", "J"])
            .range(0.0, 500.0);

        let ticks = scale.ticks(&TickOptions::new().with_max_count(5));
        assert!(ticks.len() <= 5);
    }

    #[test]
    fn test_point_scale_clone_box() {
        let scale = PointScale::new().domain(vec!["A", "B"]).range(0.0, 200.0);

        let boxed: Box<dyn Scale> = scale.clone_box();
        assert_eq!(boxed.scale_type(), "point");
    }

    #[test]
    fn test_point_scale_discrete_trait() {
        let scale = PointScale::new()
            .domain(vec!["A", "B", "C"])
            .range(0.0, 200.0);

        // Test DiscreteScale trait methods
        assert_eq!(scale.bandwidth(), 0.0); // Always zero for point scale
        assert!(scale.step() > 0.0);
    }

    #[test]
    fn test_point_scale_empty() {
        let scale = PointScale::new().range(0.0, 100.0);

        assert!(scale.is_empty());
        assert_eq!(scale.bandwidth(), 0.0);
        assert_eq!(scale.step(), 0.0);
        assert_eq!(scale.scale_index(0), 0.0);
    }

    #[test]
    fn test_point_scale_single_item() {
        let scale = PointScale::new().domain(vec!["Only"]).range(0.0, 100.0);

        assert_eq!(scale.len(), 1);
        // Single item - step equals range, point at start
        assert!((scale.scale_index(0) - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_point_scale_single_item_with_padding() {
        let scale = PointScale::new()
            .domain(vec!["Only"])
            .range(0.0, 100.0)
            .padding(0.5)
            .align(0.5);

        // Single item with padding should be centered
        let pos = scale.scale_index(0);
        assert!((pos - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_point_scale_two_items() {
        let scale = PointScale::new().domain(vec!["A", "B"]).range(0.0, 100.0);

        // Two items: step = 100 / 1 = 100
        // Points at 0 and 100
        assert!((scale.step() - 100.0).abs() < 0.01);
        assert!((scale.scale_index(0) - 0.0).abs() < 0.01);
        assert!((scale.scale_index(1) - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_point_scale_align() {
        let scale_left = PointScale::new()
            .domain(vec!["A", "B", "C"])
            .range(0.0, 200.0)
            .padding(0.5)
            .align(0.0);

        let scale_right = PointScale::new()
            .domain(vec!["A", "B", "C"])
            .range(0.0, 200.0)
            .padding(0.5)
            .align(1.0);

        let scale_center = PointScale::new()
            .domain(vec!["A", "B", "C"])
            .range(0.0, 200.0)
            .padding(0.5)
            .align(0.5);

        // Left aligned should start at 0
        assert!((scale_left.scale_index(0) - 0.0).abs() < 0.01);

        // Right aligned - last point at end
        // Center aligned - first and last equidistant from edges
        let center_first = scale_center.scale_index(0);
        let center_last = scale_center.scale_index(2);
        assert!((center_first - (200.0 - center_last)).abs() < 0.01);
    }
}

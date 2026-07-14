//! Band scale implementation
//!
//! Band scales are like ordinal scales except the output range is continuous and numeric.
//! They are useful for bar charts where you need both position and width.

use super::traits::{DiscreteScale, Scale, Tick, TickOptions};

/// Scale for mapping discrete domain to continuous bands
///
/// Band scales are commonly used for bar charts. They divide the continuous
/// range into uniform bands based on the number of values in the discrete domain.
///
/// # D3.js Equivalent
/// This is equivalent to `d3.scaleBand()` in D3.js.
///
/// # Example
/// ```
/// use makepad_d3::scale::{Scale, DiscreteScale, BandScale};
///
/// let scale = BandScale::new()
///     .domain(vec!["A", "B", "C", "D"])
///     .range(0.0, 400.0)
///     .padding(0.1);
///
/// // Get position for category "B"
/// assert!((scale.scale_category("B").unwrap() - 100.0).abs() < 10.0);
///
/// // Get the computed bandwidth
/// let bw = scale.bandwidth();
/// assert!(bw > 0.0);
/// ```
#[derive(Clone, Debug)]
pub struct BandScale {
    /// The discrete domain values
    domain_values: Vec<String>,
    /// Start of output range
    range_start: f64,
    /// End of output range
    range_end: f64,
    /// Padding between bands (0-1)
    padding_inner: f64,
    /// Padding at the edges (0-1)
    padding_outer: f64,
    /// Alignment within outer padding (0-1)
    align: f64,
    /// Whether to round output values to integers
    round: bool,
    /// Cached computed values
    cached_step: f64,
    cached_bandwidth: f64,
}

impl BandScale {
    /// Create a new band scale with default settings
    pub fn new() -> Self {
        let mut scale = Self {
            domain_values: Vec::new(),
            range_start: 0.0,
            range_end: 1.0,
            padding_inner: 0.0,
            padding_outer: 0.0,
            align: 0.5,
            round: false,
            cached_step: 0.0,
            cached_bandwidth: 0.0,
        };
        scale.rescale();
        scale
    }

    /// Set the domain (discrete categories)
    ///
    /// # Example
    /// ```
    /// use makepad_d3::scale::BandScale;
    ///
    /// let scale = BandScale::new()
    ///     .with_domain(vec!["Mon", "Tue", "Wed", "Thu", "Fri"]);
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
    /// use makepad_d3::scale::BandScale;
    ///
    /// let scale = BandScale::new()
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

    /// Set both inner and outer padding uniformly
    ///
    /// Padding is specified as a fraction of the step size (0 to 1).
    /// A padding of 0.1 means 10% of the step is used for padding.
    ///
    /// # Example
    /// ```
    /// use makepad_d3::scale::BandScale;
    ///
    /// let scale = BandScale::new()
    ///     .with_domain(vec!["A", "B", "C"])
    ///     .with_range(0.0, 300.0)
    ///     .with_padding(0.2);  // 20% padding
    /// ```
    pub fn with_padding(mut self, padding: f64) -> Self {
        let p = padding.clamp(0.0, 1.0);
        self.padding_inner = p;
        self.padding_outer = p;
        self.rescale();
        self
    }

    /// Set both inner and outer padding uniformly
    #[deprecated(
        since = "0.2.0",
        note = "Use `with_padding` instead for consistent builder pattern"
    )]
    pub fn padding(self, padding: f64) -> Self {
        self.with_padding(padding)
    }

    /// Set the inner padding between bands
    ///
    /// Inner padding is specified as a fraction of the step (0 to 1).
    pub fn with_padding_inner(mut self, padding: f64) -> Self {
        self.padding_inner = padding.clamp(0.0, 1.0);
        self.rescale();
        self
    }

    /// Set the inner padding between bands
    #[deprecated(
        since = "0.2.0",
        note = "Use `with_padding_inner` instead for consistent builder pattern"
    )]
    pub fn padding_inner(self, padding: f64) -> Self {
        self.with_padding_inner(padding)
    }

    /// Set the outer padding at the edges
    ///
    /// Outer padding is specified as a fraction of the step (0 to 1).
    pub fn with_padding_outer(mut self, padding: f64) -> Self {
        self.padding_outer = padding.clamp(0.0, 1.0);
        self.rescale();
        self
    }

    /// Set the outer padding at the edges
    #[deprecated(
        since = "0.2.0",
        note = "Use `with_padding_outer` instead for consistent builder pattern"
    )]
    pub fn padding_outer(self, padding: f64) -> Self {
        self.with_padding_outer(padding)
    }

    /// Set the alignment within outer padding
    ///
    /// Alignment of 0 means bands are left-aligned, 1 means right-aligned,
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
    /// When enabled, positions and bandwidths are rounded to integers
    /// for crisper rendering.
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

    /// Get the number of bands
    pub fn len(&self) -> usize {
        self.domain_values.len()
    }

    /// Check if the scale has no bands
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
        if self.domain_values.is_empty() || index >= self.domain_values.len() {
            return self.range_start;
        }

        let start = self.range_start + self.padding_outer * self.cached_step * self.align * 2.0;
        let pos = start + index as f64 * self.cached_step;

        if self.round {
            pos.round()
        } else {
            pos
        }
    }

    /// Get the band start position for a category by name
    pub fn band_start(&self, value: &str) -> Option<f64> {
        self.index_of(value).map(|i| self.scale_index(i))
    }

    /// Get the band end position for a category by name
    pub fn band_end(&self, value: &str) -> Option<f64> {
        self.index_of(value)
            .map(|i| self.scale_index(i) + self.cached_bandwidth)
    }

    /// Get the center position of a band by index
    pub fn center(&self, index: usize) -> f64 {
        self.scale_index(index) + self.cached_bandwidth / 2.0
    }

    /// Get the category at a pixel position
    pub fn invert_index(&self, pixel: f64) -> Option<usize> {
        if self.domain_values.is_empty() || self.cached_step == 0.0 {
            return None;
        }

        let start = self.range_start + self.padding_outer * self.cached_step * self.align * 2.0;
        let relative = pixel - start;

        if relative < 0.0 {
            return Some(0);
        }

        let index = (relative / self.cached_step).floor() as usize;
        Some(index.min(self.domain_values.len() - 1))
    }

    /// Get the category name at a pixel position
    pub fn invert(&self, pixel: f64) -> Option<&str> {
        self.invert_index(pixel)
            .and_then(|i| self.domain_values.get(i).map(String::as_str))
    }

    /// Recalculate cached values when scale parameters change
    fn rescale(&mut self) {
        let n = self.domain_values.len();
        if n == 0 {
            self.cached_step = 0.0;
            self.cached_bandwidth = 0.0;
            return;
        }

        let range = (self.range_end - self.range_start).abs();
        let n_f = n as f64;

        // D3's formula:
        // step = range / (n - paddingInner + paddingOuter * 2)
        // bandwidth = step * (1 - paddingInner)
        let divisor = n_f - self.padding_inner + self.padding_outer * 2.0;
        if divisor <= 0.0 {
            self.cached_step = range / n_f;
            self.cached_bandwidth = self.cached_step;
            return;
        }

        self.cached_step = range / divisor;
        self.cached_bandwidth = self.cached_step * (1.0 - self.padding_inner);

        if self.round {
            self.cached_step = self.cached_step.floor();
            self.cached_bandwidth = self.cached_bandwidth.floor();
        }
    }
}

impl Default for BandScale {
    fn default() -> Self {
        Self::new()
    }
}

impl Scale for BandScale {
    fn scale_type(&self) -> &'static str {
        "band"
    }

    fn set_domain(&mut self, _min: f64, _max: f64) {
        // Band scale uses discrete domain, not numeric
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
                // Position tick at center of band
                let pos = self.center(i);
                ticks.push(Tick::new(i as f64, label.clone()).with_position(pos));
            }
        }

        ticks
    }

    fn copy_from(&mut self, other: &Self) {
        self.domain_values = other.domain_values.clone();
        self.range_start = other.range_start;
        self.range_end = other.range_end;
        self.padding_inner = other.padding_inner;
        self.padding_outer = other.padding_outer;
        self.align = other.align;
        self.round = other.round;
        self.rescale();
    }

    fn clone_box(&self) -> Box<dyn Scale> {
        Box::new(self.clone())
    }
}

impl DiscreteScale for BandScale {
    fn bandwidth(&self) -> f64 {
        self.cached_bandwidth
    }

    fn step(&self) -> f64 {
        self.cached_step
    }

    fn set_padding(&mut self, padding: f64) {
        let p = padding.clamp(0.0, 1.0);
        self.padding_inner = p;
        self.padding_outer = p;
        self.rescale();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_band_scale_new() {
        let scale = BandScale::new();
        assert!(scale.is_empty());
        assert_eq!(scale.bandwidth(), 0.0);
    }

    #[test]
    fn test_band_scale_domain() {
        let scale = BandScale::new().domain(vec!["A", "B", "C", "D"]);

        assert_eq!(scale.len(), 4);
        assert_eq!(scale.index_of("A"), Some(0));
        assert_eq!(scale.index_of("B"), Some(1));
        assert_eq!(scale.index_of("C"), Some(2));
        assert_eq!(scale.index_of("D"), Some(3));
        assert_eq!(scale.index_of("E"), None);
    }

    #[test]
    fn test_band_scale_basic_mapping() {
        let scale = BandScale::new()
            .domain(vec!["A", "B", "C", "D"])
            .range(0.0, 400.0);

        // Without padding: step = 400/4 = 100, bandwidth = 100
        assert!((scale.step() - 100.0).abs() < 0.01);
        assert!((scale.bandwidth() - 100.0).abs() < 0.01);

        // Positions at band start
        assert!((scale.scale_index(0) - 0.0).abs() < 0.01);
        assert!((scale.scale_index(1) - 100.0).abs() < 0.01);
        assert!((scale.scale_index(2) - 200.0).abs() < 0.01);
        assert!((scale.scale_index(3) - 300.0).abs() < 0.01);
    }

    #[test]
    fn test_band_scale_with_padding() {
        let scale = BandScale::new()
            .domain(vec!["A", "B", "C", "D"])
            .range(0.0, 400.0)
            .padding(0.2);

        let bandwidth = scale.bandwidth();
        let step = scale.step();

        // Bandwidth should be 80% of step due to 20% inner padding
        assert!((bandwidth / step - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_band_scale_with_inner_padding_only() {
        let scale = BandScale::new()
            .domain(vec!["A", "B", "C", "D"])
            .range(0.0, 400.0)
            .padding_inner(0.5);

        // With 50% inner padding, bandwidth = step * 0.5
        let bandwidth = scale.bandwidth();
        let step = scale.step();
        assert!((bandwidth / step - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_band_scale_scale_category() {
        let scale = BandScale::new()
            .domain(vec!["Mon", "Tue", "Wed", "Thu", "Fri"])
            .range(0.0, 500.0);

        assert!(scale.scale_category("Mon").is_some());
        assert!(scale.scale_category("Wed").is_some());
        assert!(scale.scale_category("Sat").is_none());
    }

    #[test]
    fn test_band_scale_invert() {
        let scale = BandScale::new()
            .domain(vec!["A", "B", "C", "D"])
            .range(0.0, 400.0);

        assert_eq!(scale.invert(50.0), Some("A"));
        assert_eq!(scale.invert(150.0), Some("B"));
        assert_eq!(scale.invert(250.0), Some("C"));
        assert_eq!(scale.invert(350.0), Some("D"));
    }

    #[test]
    fn test_band_scale_round() {
        let scale = BandScale::new()
            .domain(vec!["A", "B", "C"])
            .range(0.0, 100.0)
            .round(true);

        // With rounding, values should be integers
        let step = scale.step();
        let bandwidth = scale.bandwidth();
        assert_eq!(step, step.floor());
        assert_eq!(bandwidth, bandwidth.floor());
    }

    #[test]
    fn test_band_scale_center() {
        let scale = BandScale::new()
            .domain(vec!["A", "B", "C", "D"])
            .range(0.0, 400.0);

        // Center should be at band_start + bandwidth/2
        assert!((scale.center(0) - 50.0).abs() < 0.01);
        assert!((scale.center(1) - 150.0).abs() < 0.01);
    }

    #[test]
    fn test_band_scale_ticks() {
        let scale = BandScale::new()
            .domain(vec!["A", "B", "C", "D"])
            .range(0.0, 400.0);

        let ticks = scale.ticks(&TickOptions::default());

        assert_eq!(ticks.len(), 4);
        assert_eq!(ticks[0].label, "A");
        assert_eq!(ticks[3].label, "D");

        // Ticks should be positioned at band centers
        assert!((ticks[0].position - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_band_scale_ticks_max_count() {
        let scale = BandScale::new()
            .domain(vec!["A", "B", "C", "D", "E", "F", "G", "H", "I", "J"])
            .range(0.0, 500.0);

        let ticks = scale.ticks(&TickOptions::new().with_max_count(5));
        assert!(ticks.len() <= 5);
    }

    #[test]
    fn test_band_scale_clone_box() {
        let scale = BandScale::new().domain(vec!["A", "B"]).range(0.0, 200.0);

        let boxed: Box<dyn Scale> = scale.clone_box();
        assert_eq!(boxed.scale_type(), "band");
    }

    #[test]
    fn test_band_scale_discrete_trait() {
        let scale = BandScale::new()
            .domain(vec!["A", "B", "C"])
            .range(0.0, 300.0)
            .padding(0.1);

        // Test DiscreteScale trait methods
        assert!(scale.bandwidth() > 0.0);
        assert!(scale.step() > 0.0);
        assert!(scale.bandwidth() < scale.step());
    }

    #[test]
    fn test_band_scale_empty() {
        let scale = BandScale::new().range(0.0, 100.0);

        assert!(scale.is_empty());
        assert_eq!(scale.bandwidth(), 0.0);
        assert_eq!(scale.step(), 0.0);
        assert_eq!(scale.scale_index(0), 0.0);
    }

    #[test]
    fn test_band_scale_single_item() {
        let scale = BandScale::new().domain(vec!["Only"]).range(0.0, 100.0);

        assert_eq!(scale.len(), 1);
        // Single item should span the whole range (minus padding)
        assert!((scale.bandwidth() - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_band_scale_outer_padding() {
        let scale = BandScale::new()
            .domain(vec!["A", "B", "C"])
            .range(0.0, 300.0)
            .padding_outer(0.5);

        // With outer padding, first band should not start at 0
        let first_pos = scale.scale_index(0);
        assert!(first_pos > 0.0);
    }
}

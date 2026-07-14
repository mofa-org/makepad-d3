//! Category scale implementation

use super::traits::{DiscreteScale, Scale, Tick, TickOptions};

/// Scale for categorical/discrete data
///
/// Maps discrete categories to continuous bands or points.
/// Used primarily for bar charts and grouped visualizations.
///
/// # Example
/// ```
/// use makepad_d3::scale::{Scale, CategoryScale};
///
/// let scale = CategoryScale::new()
///     .with_labels(vec!["A", "B", "C", "D"])
///     .with_range(0.0, 400.0);
///
/// // With offset=true (default), items are centered in bands
/// assert_eq!(scale.scale(0.0), 50.0);  // Center of first band
/// assert_eq!(scale.scale(1.0), 150.0); // Center of second band
/// ```
#[derive(Clone, Debug)]
pub struct CategoryScale {
    labels: Vec<String>,
    range_start: f64,
    range_end: f64,
    padding_inner: f64,
    padding_outer: f64,
    align: f64,
    offset: bool,
}

impl CategoryScale {
    /// Create a new category scale
    pub fn new() -> Self {
        Self {
            labels: Vec::new(),
            range_start: 0.0,
            range_end: 100.0,
            padding_inner: 0.0,
            padding_outer: 0.0,
            align: 0.5,
            offset: true,
        }
    }

    /// Set category labels
    pub fn with_labels<S: Into<String>>(mut self, labels: impl IntoIterator<Item = S>) -> Self {
        self.labels = labels.into_iter().map(Into::into).collect();
        self
    }

    /// Set range
    pub fn with_range(mut self, start: f64, end: f64) -> Self {
        self.range_start = start;
        self.range_end = end;
        self
    }

    /// Set inner padding (between bands) as fraction 0-1
    pub fn with_padding_inner(mut self, padding: f64) -> Self {
        self.padding_inner = padding.clamp(0.0, 1.0);
        self
    }

    /// Set outer padding (at edges) as fraction 0-1
    pub fn with_padding_outer(mut self, padding: f64) -> Self {
        self.padding_outer = padding.clamp(0.0, 1.0);
        self
    }

    /// Set uniform padding (inner and outer)
    pub fn with_padding(mut self, padding: f64) -> Self {
        self.padding_inner = padding.clamp(0.0, 1.0);
        self.padding_outer = padding.clamp(0.0, 1.0);
        self
    }

    /// Set alignment within step (0-1)
    pub fn with_align(mut self, align: f64) -> Self {
        self.align = align.clamp(0.0, 1.0);
        self
    }

    /// Set offset mode
    ///
    /// When true, items are centered between grid lines (bar charts).
    /// When false, items are placed on grid lines (line charts).
    pub fn with_offset(mut self, offset: bool) -> Self {
        self.offset = offset;
        self
    }

    /// Get number of categories
    pub fn len(&self) -> usize {
        self.labels.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.labels.is_empty()
    }

    /// Get label at index
    pub fn label(&self, index: usize) -> Option<&str> {
        self.labels.get(index).map(String::as_str)
    }

    /// Get all labels
    pub fn labels(&self) -> &[String] {
        &self.labels
    }

    /// Find index for a label
    pub fn index_of(&self, label: &str) -> Option<usize> {
        self.labels.iter().position(|l| l == label)
    }

    /// Get pixel position for category index
    pub fn scale_index(&self, index: usize) -> f64 {
        if self.labels.is_empty() {
            return self.range_start;
        }

        let step = self.step();
        let bandwidth = self.bandwidth();
        let outer_padding = self.padding_outer * step;
        let base = self.range_start + outer_padding + index as f64 * step;

        if self.offset {
            // Center within the band
            base + bandwidth / 2.0
        } else {
            base
        }
    }

    /// Get category index for pixel position
    pub fn invert_index(&self, pixel: f64) -> usize {
        if self.labels.is_empty() {
            return 0;
        }

        let step = self.step();
        if step == 0.0 {
            return 0;
        }

        let outer_padding = self.padding_outer * step;
        let adjusted = pixel - self.range_start - outer_padding;
        let index = (adjusted / step).floor() as i64;
        index.clamp(0, (self.labels.len().saturating_sub(1)) as i64) as usize
    }

    /// Get the band start position for an index
    pub fn band_start(&self, index: usize) -> f64 {
        if self.labels.is_empty() {
            return self.range_start;
        }

        let step = self.step();
        let outer_padding = self.padding_outer * step;
        self.range_start + outer_padding + index as f64 * step
    }

    /// Get the band end position for an index
    pub fn band_end(&self, index: usize) -> f64 {
        self.band_start(index) + self.bandwidth()
    }
}

impl Default for CategoryScale {
    fn default() -> Self {
        Self::new()
    }
}

impl Scale for CategoryScale {
    fn scale_type(&self) -> &'static str {
        "category"
    }

    fn set_domain(&mut self, _min: f64, _max: f64) {
        // Category scale doesn't use numeric domain
        // Labels are set via with_labels()
    }

    fn set_range(&mut self, start: f64, end: f64) {
        self.range_start = start;
        self.range_end = end;
    }

    fn domain(&self) -> (f64, f64) {
        (0.0, (self.labels.len().saturating_sub(1)) as f64)
    }

    fn range(&self) -> (f64, f64) {
        (self.range_start, self.range_end)
    }

    fn scale(&self, value: f64) -> f64 {
        self.scale_index(value.round().max(0.0) as usize)
    }

    fn invert(&self, pixel: f64) -> f64 {
        self.invert_index(pixel) as f64
    }

    fn ticks(&self, options: &TickOptions) -> Vec<Tick> {
        let mut ticks = Vec::with_capacity(self.labels.len());

        // Calculate step for skipping labels if too many
        let step = if self.labels.len() > options.max_count && options.max_count > 0 {
            (self.labels.len() as f64 / options.max_count as f64).ceil() as usize
        } else {
            1
        };

        for (i, label) in self.labels.iter().enumerate() {
            if i % step == 0 {
                let pos = self.scale_index(i);
                ticks.push(Tick::new(i as f64, label.clone()).with_position(pos));
            }
        }

        ticks
    }

    fn copy_from(&mut self, other: &Self) {
        self.labels = other.labels.clone();
        self.range_start = other.range_start;
        self.range_end = other.range_end;
        self.padding_inner = other.padding_inner;
        self.padding_outer = other.padding_outer;
        self.align = other.align;
        self.offset = other.offset;
    }

    fn clone_box(&self) -> Box<dyn Scale> {
        Box::new(self.clone())
    }
}

impl DiscreteScale for CategoryScale {
    fn bandwidth(&self) -> f64 {
        if self.labels.is_empty() {
            return 0.0;
        }

        let step = self.step();
        step * (1.0 - self.padding_inner)
    }

    fn step(&self) -> f64 {
        if self.labels.is_empty() {
            return 0.0;
        }

        let n = self.labels.len() as f64;
        let range = (self.range_end - self.range_start).abs();

        // Total space = n * step + 2 * outer_padding * step
        // n * step * (1 + 2 * outer_padding / n) = range
        // But we also have inner padding between bands
        // Effective: range = (n - 1) * inner_padding * step + n * bandwidth + 2 * outer_padding * step
        // bandwidth = step * (1 - inner_padding)
        // range = (n - 1) * inner_padding * step + n * step * (1 - inner_padding) + 2 * outer_padding * step
        // range = step * ((n - 1) * inner_padding + n * (1 - inner_padding) + 2 * outer_padding)
        // range = step * ((n - 1) * inner_padding + n - n * inner_padding + 2 * outer_padding)
        // range = step * (n - inner_padding + 2 * outer_padding)

        let divisor = n - self.padding_inner + 2.0 * self.padding_outer;
        if divisor <= 0.0 {
            return range / n;
        }

        range / divisor
    }

    fn set_padding(&mut self, padding: f64) {
        self.padding_inner = padding.clamp(0.0, 1.0);
        self.padding_outer = padding.clamp(0.0, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_scale_new() {
        let scale = CategoryScale::new();
        assert!(scale.is_empty());
        assert_eq!(scale.range(), (0.0, 100.0));
    }

    #[test]
    fn test_category_scale_with_labels() {
        let scale = CategoryScale::new().with_labels(vec!["A", "B", "C", "D"]);

        assert_eq!(scale.len(), 4);
        assert_eq!(scale.label(0), Some("A"));
        assert_eq!(scale.label(3), Some("D"));
        assert_eq!(scale.label(4), None);
    }

    #[test]
    fn test_category_scale_basic() {
        let scale = CategoryScale::new()
            .with_labels(vec!["A", "B", "C", "D"])
            .with_range(0.0, 400.0);

        assert_eq!(scale.len(), 4);
        // Without padding: step = 400 / 4 = 100, bandwidth = 100
        assert!((scale.step() - 100.0).abs() < 0.01);
        assert!((scale.bandwidth() - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_category_scale_with_offset() {
        let scale = CategoryScale::new()
            .with_labels(vec!["A", "B", "C", "D"])
            .with_range(0.0, 400.0)
            .with_offset(true);

        // Items should be centered in bands
        assert!((scale.scale_index(0) - 50.0).abs() < 0.01);
        assert!((scale.scale_index(1) - 150.0).abs() < 0.01);
        assert!((scale.scale_index(2) - 250.0).abs() < 0.01);
        assert!((scale.scale_index(3) - 350.0).abs() < 0.01);
    }

    #[test]
    fn test_category_scale_without_offset() {
        let scale = CategoryScale::new()
            .with_labels(vec!["A", "B", "C", "D"])
            .with_range(0.0, 400.0)
            .with_offset(false);

        // Items should be at start of bands
        assert!((scale.scale_index(0) - 0.0).abs() < 0.01);
        assert!((scale.scale_index(1) - 100.0).abs() < 0.01);
        assert!((scale.scale_index(2) - 200.0).abs() < 0.01);
        assert!((scale.scale_index(3) - 300.0).abs() < 0.01);
    }

    #[test]
    fn test_category_scale_invert() {
        let scale = CategoryScale::new()
            .with_labels(vec!["A", "B", "C", "D"])
            .with_range(0.0, 400.0);

        assert_eq!(scale.invert_index(50.0), 0);
        assert_eq!(scale.invert_index(150.0), 1);
        assert_eq!(scale.invert_index(250.0), 2);
        assert_eq!(scale.invert_index(350.0), 3);
    }

    #[test]
    fn test_category_scale_with_padding() {
        let scale = CategoryScale::new()
            .with_labels(vec!["A", "B", "C", "D"])
            .with_range(0.0, 400.0)
            .with_padding_inner(0.2);

        let bandwidth = scale.bandwidth();
        let step = scale.step();

        // Bandwidth should be 80% of step
        assert!((bandwidth / step - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_category_scale_index_of() {
        let scale = CategoryScale::new().with_labels(vec!["Apple", "Banana", "Cherry"]);

        assert_eq!(scale.index_of("Apple"), Some(0));
        assert_eq!(scale.index_of("Banana"), Some(1));
        assert_eq!(scale.index_of("Cherry"), Some(2));
        assert_eq!(scale.index_of("Date"), None);
    }

    #[test]
    fn test_category_scale_ticks() {
        let scale = CategoryScale::new()
            .with_labels(vec!["A", "B", "C", "D"])
            .with_range(0.0, 400.0);

        let ticks = scale.ticks(&TickOptions::default());

        assert_eq!(ticks.len(), 4);
        assert_eq!(ticks[0].label, "A");
        assert_eq!(ticks[1].label, "B");
        assert_eq!(ticks[2].label, "C");
        assert_eq!(ticks[3].label, "D");
    }

    #[test]
    fn test_category_scale_ticks_max_count() {
        let scale = CategoryScale::new()
            .with_labels(vec!["A", "B", "C", "D", "E", "F", "G", "H", "I", "J"])
            .with_range(0.0, 500.0);

        let ticks = scale.ticks(&TickOptions::new().with_max_count(5));

        assert!(ticks.len() <= 5);
    }

    #[test]
    fn test_category_scale_empty() {
        let scale = CategoryScale::new();

        assert_eq!(scale.bandwidth(), 0.0);
        assert_eq!(scale.step(), 0.0);
        assert_eq!(scale.scale_index(0), 0.0);
    }

    #[test]
    fn test_category_scale_clone_box() {
        let scale = CategoryScale::new()
            .with_labels(vec!["A", "B"])
            .with_range(0.0, 200.0);

        let boxed: Box<dyn Scale> = scale.clone_box();
        assert_eq!(boxed.scale_type(), "category");
    }

    #[test]
    fn test_category_scale_band_positions() {
        let scale = CategoryScale::new()
            .with_labels(vec!["A", "B", "C", "D"])
            .with_range(0.0, 400.0);

        assert!((scale.band_start(0) - 0.0).abs() < 0.01);
        assert!((scale.band_end(0) - 100.0).abs() < 0.01);
        assert!((scale.band_start(1) - 100.0).abs() < 0.01);
    }
}

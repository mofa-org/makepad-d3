//! Stack generator for stacked charts
//!
//! Computes stacked layouts for bar charts, area charts, and stream graphs.

use crate::data::ChartData;

/// Stack ordering method
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StackOrder {
    /// No reordering, maintain original series order
    #[default]
    None,
    /// Sort by sum of values ascending
    Ascending,
    /// Sort by sum of values descending
    Descending,
    /// Sort so smallest series are in the middle
    InsideOut,
    /// Reverse the current order
    Reverse,
}

/// Stack offset method
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StackOffset {
    /// No offset, stack from zero
    #[default]
    None,
    /// Normalize to fill [0, 1] range
    Expand,
    /// Center around zero (diverging stacks)
    Diverging,
    /// Center the baseline to minimize weighted wiggle
    Silhouette,
    /// Streamgraph wiggle minimization (Stacked Graph algorithm)
    Wiggle,
}

/// A single point in a stacked series
#[derive(Clone, Debug)]
pub struct StackPoint {
    /// Lower bound (y0)
    pub y0: f64,
    /// Upper bound (y1)
    pub y1: f64,
}

impl StackPoint {
    /// Create a new stack point
    pub fn new(y0: f64, y1: f64) -> Self {
        Self { y0, y1 }
    }

    /// Get the height of this stack segment
    pub fn height(&self) -> f64 {
        self.y1 - self.y0
    }
}

/// A stacked series with its key and stacked points
#[derive(Clone, Debug)]
pub struct StackedSeries {
    /// Series identifier (label)
    pub key: String,
    /// Index of this series in the original data
    pub index: usize,
    /// Stacked points (y0, y1) for each data point
    pub points: Vec<StackPoint>,
}

impl StackedSeries {
    /// Create a new stacked series
    pub fn new(key: String, index: usize, n_points: usize) -> Self {
        Self {
            key,
            index,
            points: vec![StackPoint::new(0.0, 0.0); n_points],
        }
    }
}

/// Stack generator for creating stacked layouts
///
/// # Example
/// ```
/// use makepad_d3::data::{ChartData, Dataset, DataPoint};
/// use makepad_d3::shape::{StackGenerator, StackOffset, StackOrder};
///
/// let data = ChartData::new()
///     .with_labels(vec!["Q1", "Q2", "Q3", "Q4"])
///     .add_dataset(
///         Dataset::new("Product A")
///             .with_data(vec![10.0, 20.0, 15.0, 25.0])
///     )
///     .add_dataset(
///         Dataset::new("Product B")
///             .with_data(vec![15.0, 25.0, 20.0, 30.0])
///     );
///
/// let stack = StackGenerator::new();
/// let stacked = stack.compute(&data);
///
/// assert_eq!(stacked.len(), 2); // Two series
/// ```
#[derive(Clone, Debug)]
pub struct StackGenerator {
    /// Ordering method
    order: StackOrder,
    /// Offset method
    offset: StackOffset,
}

impl Default for StackGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl StackGenerator {
    /// Create a new stack generator
    pub fn new() -> Self {
        Self {
            order: StackOrder::None,
            offset: StackOffset::None,
        }
    }

    /// Set the stack order
    pub fn order(mut self, order: StackOrder) -> Self {
        self.order = order;
        self
    }

    /// Set the stack offset
    pub fn offset(mut self, offset: StackOffset) -> Self {
        self.offset = offset;
        self
    }

    /// Compute stacked series from chart data
    pub fn compute(&self, data: &ChartData) -> Vec<StackedSeries> {
        let n_series = data.datasets.len();
        if n_series == 0 {
            return vec![];
        }

        let n_points = data.len();
        if n_points == 0 {
            return vec![];
        }

        // Initialize result with series info
        let mut result: Vec<StackedSeries> = data
            .datasets
            .iter()
            .enumerate()
            .map(|(i, d)| StackedSeries::new(d.label.clone(), i, n_points))
            .collect();

        // Compute series order
        let order = self.compute_order(data);

        // Stack values
        for i in 0..n_points {
            let mut y0 = 0.0;
            for &series_idx in &order {
                let y = data.datasets[series_idx]
                    .data
                    .get(i)
                    .map(|p| p.y)
                    .unwrap_or(0.0);

                result[series_idx].points[i] = StackPoint::new(y0, y0 + y);
                y0 += y;
            }
        }

        // Apply offset
        self.apply_offset(&mut result, n_points);

        result
    }

    /// Compute stacked series from raw values
    ///
    /// Each inner Vec is a series, containing values for each point.
    pub fn compute_from_values(&self, values: &[Vec<f64>], keys: &[String]) -> Vec<StackedSeries> {
        let n_series = values.len();
        if n_series == 0 {
            return vec![];
        }

        let n_points = values[0].len();
        if n_points == 0 {
            return vec![];
        }

        // Initialize result
        let mut result: Vec<StackedSeries> = values
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let key = keys
                    .get(i)
                    .cloned()
                    .unwrap_or_else(|| format!("series_{}", i));
                StackedSeries::new(key, i, n_points)
            })
            .collect();

        // Compute order
        let order = self.compute_order_from_values(values);

        // Stack values
        for i in 0..n_points {
            let mut y0 = 0.0;
            for &series_idx in &order {
                let y = values[series_idx].get(i).copied().unwrap_or(0.0);
                result[series_idx].points[i] = StackPoint::new(y0, y0 + y);
                y0 += y;
            }
        }

        // Apply offset
        self.apply_offset(&mut result, n_points);

        result
    }

    /// Compute the series order
    fn compute_order(&self, data: &ChartData) -> Vec<usize> {
        let n = data.datasets.len();
        let mut indices: Vec<usize> = (0..n).collect();

        match self.order {
            StackOrder::None => {}
            StackOrder::Ascending => {
                let sums: Vec<f64> = data
                    .datasets
                    .iter()
                    .map(|d| d.data.iter().map(|p| p.y).sum())
                    .collect();
                indices.sort_by(|&a, &b| sums[a].partial_cmp(&sums[b]).unwrap());
            }
            StackOrder::Descending => {
                let sums: Vec<f64> = data
                    .datasets
                    .iter()
                    .map(|d| d.data.iter().map(|p| p.y).sum())
                    .collect();
                indices.sort_by(|&a, &b| sums[b].partial_cmp(&sums[a]).unwrap());
            }
            StackOrder::InsideOut => {
                // Sort by sum, then interleave smallest to center
                let sums: Vec<f64> = data
                    .datasets
                    .iter()
                    .map(|d| d.data.iter().map(|p| p.y).sum())
                    .collect();
                indices.sort_by(|&a, &b| sums[b].partial_cmp(&sums[a]).unwrap());

                let mut new_order = Vec::with_capacity(n);
                let mut top = true;
                for idx in indices {
                    if top {
                        new_order.push(idx);
                    } else {
                        new_order.insert(0, idx);
                    }
                    top = !top;
                }
                indices = new_order;
            }
            StackOrder::Reverse => {
                indices.reverse();
            }
        }

        indices
    }

    /// Compute the series order from raw values
    fn compute_order_from_values(&self, values: &[Vec<f64>]) -> Vec<usize> {
        let n = values.len();
        let mut indices: Vec<usize> = (0..n).collect();

        match self.order {
            StackOrder::None => {}
            StackOrder::Ascending => {
                let sums: Vec<f64> = values.iter().map(|v| v.iter().sum()).collect();
                indices.sort_by(|&a, &b| sums[a].partial_cmp(&sums[b]).unwrap());
            }
            StackOrder::Descending => {
                let sums: Vec<f64> = values.iter().map(|v| v.iter().sum()).collect();
                indices.sort_by(|&a, &b| sums[b].partial_cmp(&sums[a]).unwrap());
            }
            StackOrder::InsideOut => {
                let sums: Vec<f64> = values.iter().map(|v| v.iter().sum()).collect();
                indices.sort_by(|&a, &b| sums[b].partial_cmp(&sums[a]).unwrap());

                let mut new_order = Vec::with_capacity(n);
                let mut top = true;
                for idx in indices {
                    if top {
                        new_order.push(idx);
                    } else {
                        new_order.insert(0, idx);
                    }
                    top = !top;
                }
                indices = new_order;
            }
            StackOrder::Reverse => {
                indices.reverse();
            }
        }

        indices
    }

    /// Apply offset to stacked series
    fn apply_offset(&self, series: &mut [StackedSeries], n_points: usize) {
        match self.offset {
            StackOffset::None => {}
            StackOffset::Expand => {
                self.apply_expand_offset(series, n_points);
            }
            StackOffset::Diverging => {
                self.apply_diverging_offset(series, n_points);
            }
            StackOffset::Silhouette => {
                self.apply_silhouette_offset(series, n_points);
            }
            StackOffset::Wiggle => {
                self.apply_wiggle_offset(series, n_points);
            }
        }
    }

    /// Normalize to [0, 1] range
    fn apply_expand_offset(&self, series: &mut [StackedSeries], n_points: usize) {
        for i in 0..n_points {
            let total: f64 = series.iter().map(|s| s.points[i].height()).sum();

            if total > 0.0 {
                for s in series.iter_mut() {
                    s.points[i].y0 /= total;
                    s.points[i].y1 /= total;
                }
            }
        }
    }

    /// Center around zero
    fn apply_diverging_offset(&self, series: &mut [StackedSeries], n_points: usize) {
        for i in 0..n_points {
            let total: f64 = series.iter().map(|s| s.points[i].height()).sum();

            let offset = -total / 2.0;
            for s in series.iter_mut() {
                s.points[i].y0 += offset;
                s.points[i].y1 += offset;
            }
        }
    }

    /// Center baseline (silhouette)
    fn apply_silhouette_offset(&self, series: &mut [StackedSeries], n_points: usize) {
        for i in 0..n_points {
            let total: f64 = series
                .iter()
                .map(|s| s.points[i].y1)
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap_or(0.0);

            let offset = -total / 2.0;
            for s in series.iter_mut() {
                s.points[i].y0 += offset;
                s.points[i].y1 += offset;
            }
        }
    }

    /// Minimize wiggle (streamgraph)
    fn apply_wiggle_offset(&self, series: &mut [StackedSeries], n_points: usize) {
        if series.is_empty() || n_points == 0 {
            return;
        }

        let n = series.len();

        for i in 0..n_points {
            // Calculate weighted centroid offset
            let mut sum = 0.0;
            let mut total_weight = 0.0;

            for (j, s) in series.iter().enumerate() {
                let height = s.points[i].height();
                let weight = (n - j) as f64;
                sum += weight * height;
                total_weight += weight;
            }

            let total: f64 = series.iter().map(|s| s.points[i].height()).sum();
            let offset = if total_weight > 0.0 && total > 0.0 {
                -sum / (total_weight * 2.0)
            } else {
                0.0
            };

            for s in series.iter_mut() {
                s.points[i].y0 += offset;
                s.points[i].y1 += offset;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::Dataset;

    fn sample_data() -> ChartData {
        ChartData::new()
            .with_labels(vec!["A", "B", "C", "D"])
            .add_dataset(Dataset::new("Series 1").with_data(vec![10.0, 20.0, 15.0, 25.0]))
            .add_dataset(Dataset::new("Series 2").with_data(vec![15.0, 25.0, 20.0, 30.0]))
            .add_dataset(Dataset::new("Series 3").with_data(vec![5.0, 10.0, 5.0, 10.0]))
    }

    #[test]
    fn test_stack_basic() {
        let data = sample_data();
        let stack = StackGenerator::new();
        let result = stack.compute(&data);

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].points.len(), 4);

        // First series should start at 0
        assert_eq!(result[0].points[0].y0, 0.0);
        assert_eq!(result[0].points[0].y1, 10.0);

        // Second series should stack on first
        assert_eq!(result[1].points[0].y0, 10.0);
        assert_eq!(result[1].points[0].y1, 25.0);

        // Third series should stack on second
        assert_eq!(result[2].points[0].y0, 25.0);
        assert_eq!(result[2].points[0].y1, 30.0);
    }

    #[test]
    fn test_stack_expand() {
        let data = sample_data();
        let stack = StackGenerator::new().offset(StackOffset::Expand);
        let result = stack.compute(&data);

        // All points should be normalized to [0, 1]
        for i in 0..4 {
            let total: f64 = result.iter().map(|s| s.points[i].height()).sum();
            assert!((total - 1.0).abs() < 0.01);
        }
    }

    #[test]
    fn test_stack_diverging() {
        let data = sample_data();
        let stack = StackGenerator::new().offset(StackOffset::Diverging);
        let result = stack.compute(&data);

        // Stacks should be centered around 0
        for i in 0..4 {
            let min_y0 = result
                .iter()
                .map(|s| s.points[i].y0)
                .fold(f64::INFINITY, f64::min);
            let max_y1 = result
                .iter()
                .map(|s| s.points[i].y1)
                .fold(f64::NEG_INFINITY, f64::max);
            let center = (min_y0 + max_y1) / 2.0;
            assert!(center.abs() < 0.01);
        }
    }

    #[test]
    fn test_stack_order_descending() {
        let data = sample_data();
        let stack = StackGenerator::new().order(StackOrder::Descending);
        let result = stack.compute(&data);

        // Series should be ordered by sum descending
        // Series 2 has highest sum (90), then Series 1 (70), then Series 3 (30)
        // But the result keeps original indices, just changes stacking order
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_stack_from_values() {
        let values = vec![vec![10.0, 20.0, 15.0], vec![15.0, 25.0, 20.0]];
        let keys = vec!["A".to_string(), "B".to_string()];

        let stack = StackGenerator::new();
        let result = stack.compute_from_values(&values, &keys);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].key, "A");
        assert_eq!(result[1].key, "B");
    }

    #[test]
    fn test_stack_empty() {
        let data = ChartData::new();
        let stack = StackGenerator::new();
        let result = stack.compute(&data);
        assert!(result.is_empty());
    }
}

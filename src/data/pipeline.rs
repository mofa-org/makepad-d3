//! Data pipeline for transformations and filtering
//!
//! Provides composable data transformations that can be chained together
//! to process data before rendering.
//!
//! # Example
//!
//! ```
//! use makepad_d3::data::{DataPipeline, DataPoint, Transform};
//!
//! let pipeline = DataPipeline::new()
//!     .filter(|p| p.y > 0.0)
//!     .map(|p| DataPoint::from_y(p.y * 2.0))
//!     .window(100);
//!
//! let data = vec![
//!     DataPoint::from_y(-10.0),
//!     DataPoint::from_y(50.0),
//!     DataPoint::from_y(100.0),
//! ];
//!
//! let result = pipeline.apply(&data);
//! assert_eq!(result.len(), 2); // -10 filtered out
//! assert_eq!(result[0].y, 100.0); // 50 * 2
//! ```

use super::DataPoint;

/// Transform operation types
pub enum Transform {
    /// Filter points by predicate
    Filter(Box<dyn Fn(&DataPoint) -> bool + Send + Sync>),
    /// Map points to new values
    Map(Box<dyn Fn(&DataPoint) -> DataPoint + Send + Sync>),
    /// Take only the last N points (sliding window)
    Window(usize),
    /// Skip first N points
    Skip(usize),
    /// Take first N points
    Take(usize),
    /// Sample every Nth point
    Sample(usize),
    /// Smooth using moving average
    MovingAverage(usize),
    /// Clamp Y values to range
    ClampY { min: f64, max: f64 },
    /// Scale Y values by factor
    ScaleY(f64),
    /// Offset Y values by amount
    OffsetY(f64),
    /// Normalize Y to 0-1 range
    NormalizeY,
    /// Remove NaN/Inf values
    RemoveInvalid,
    /// Sort by X value
    SortByX,
    /// Sort by Y value
    SortByY,
    /// Reverse order
    Reverse,
    /// Deduplicate consecutive equal Y values
    Dedupe,
}

impl std::fmt::Debug for Transform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Transform::Filter(_) => write!(f, "Filter(...)"),
            Transform::Map(_) => write!(f, "Map(...)"),
            Transform::Window(n) => write!(f, "Window({})", n),
            Transform::Skip(n) => write!(f, "Skip({})", n),
            Transform::Take(n) => write!(f, "Take({})", n),
            Transform::Sample(n) => write!(f, "Sample({})", n),
            Transform::MovingAverage(n) => write!(f, "MovingAverage({})", n),
            Transform::ClampY { min, max } => write!(f, "ClampY({}, {})", min, max),
            Transform::ScaleY(s) => write!(f, "ScaleY({})", s),
            Transform::OffsetY(o) => write!(f, "OffsetY({})", o),
            Transform::NormalizeY => write!(f, "NormalizeY"),
            Transform::RemoveInvalid => write!(f, "RemoveInvalid"),
            Transform::SortByX => write!(f, "SortByX"),
            Transform::SortByY => write!(f, "SortByY"),
            Transform::Reverse => write!(f, "Reverse"),
            Transform::Dedupe => write!(f, "Dedupe"),
        }
    }
}

/// Data transformation pipeline
///
/// Chains multiple transformations that are applied in sequence to data.
#[derive(Debug, Default)]
pub struct DataPipeline {
    transforms: Vec<Transform>,
}

impl DataPipeline {
    /// Create a new empty pipeline
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a filter transform
    pub fn filter<F>(mut self, predicate: F) -> Self
    where
        F: Fn(&DataPoint) -> bool + Send + Sync + 'static,
    {
        self.transforms.push(Transform::Filter(Box::new(predicate)));
        self
    }

    /// Add a map transform
    pub fn map<F>(mut self, mapper: F) -> Self
    where
        F: Fn(&DataPoint) -> DataPoint + Send + Sync + 'static,
    {
        self.transforms.push(Transform::Map(Box::new(mapper)));
        self
    }

    /// Add a sliding window (take last N points)
    pub fn window(mut self, size: usize) -> Self {
        self.transforms.push(Transform::Window(size));
        self
    }

    /// Skip first N points
    pub fn skip(mut self, count: usize) -> Self {
        self.transforms.push(Transform::Skip(count));
        self
    }

    /// Take first N points
    pub fn take(mut self, count: usize) -> Self {
        self.transforms.push(Transform::Take(count));
        self
    }

    /// Sample every Nth point
    pub fn sample(mut self, n: usize) -> Self {
        self.transforms.push(Transform::Sample(n.max(1)));
        self
    }

    /// Smooth with moving average
    pub fn moving_average(mut self, window: usize) -> Self {
        self.transforms
            .push(Transform::MovingAverage(window.max(1)));
        self
    }

    /// Clamp Y values to range
    pub fn clamp_y(mut self, min: f64, max: f64) -> Self {
        self.transforms.push(Transform::ClampY { min, max });
        self
    }

    /// Scale Y values by factor
    pub fn scale_y(mut self, factor: f64) -> Self {
        self.transforms.push(Transform::ScaleY(factor));
        self
    }

    /// Offset Y values by amount
    pub fn offset_y(mut self, offset: f64) -> Self {
        self.transforms.push(Transform::OffsetY(offset));
        self
    }

    /// Normalize Y to 0-1 range
    pub fn normalize_y(mut self) -> Self {
        self.transforms.push(Transform::NormalizeY);
        self
    }

    /// Remove NaN/Inf values
    pub fn remove_invalid(mut self) -> Self {
        self.transforms.push(Transform::RemoveInvalid);
        self
    }

    /// Sort by X value
    pub fn sort_by_x(mut self) -> Self {
        self.transforms.push(Transform::SortByX);
        self
    }

    /// Sort by Y value
    pub fn sort_by_y(mut self) -> Self {
        self.transforms.push(Transform::SortByY);
        self
    }

    /// Reverse order
    pub fn reverse(mut self) -> Self {
        self.transforms.push(Transform::Reverse);
        self
    }

    /// Deduplicate consecutive equal Y values
    pub fn dedupe(mut self) -> Self {
        self.transforms.push(Transform::Dedupe);
        self
    }

    /// Apply all transforms to data
    pub fn apply(&self, data: &[DataPoint]) -> Vec<DataPoint> {
        let mut result: Vec<DataPoint> = data.to_vec();

        for transform in &self.transforms {
            result = Self::apply_transform(&result, transform);
        }

        result
    }

    /// Apply single transform
    fn apply_transform(data: &[DataPoint], transform: &Transform) -> Vec<DataPoint> {
        match transform {
            Transform::Filter(predicate) => data.iter().filter(|p| predicate(p)).cloned().collect(),
            Transform::Map(mapper) => data.iter().map(|p| mapper(p)).collect(),
            Transform::Window(size) => {
                if data.len() <= *size {
                    data.to_vec()
                } else {
                    data[data.len() - size..].to_vec()
                }
            }
            Transform::Skip(count) => data.iter().skip(*count).cloned().collect(),
            Transform::Take(count) => data.iter().take(*count).cloned().collect(),
            Transform::Sample(n) => data.iter().step_by(*n).cloned().collect(),
            Transform::MovingAverage(window) => Self::apply_moving_average(data, *window),
            Transform::ClampY { min, max } => data
                .iter()
                .map(|p| {
                    let mut point = p.clone();
                    point.y = point.y.clamp(*min, *max);
                    point
                })
                .collect(),
            Transform::ScaleY(factor) => data
                .iter()
                .map(|p| {
                    let mut point = p.clone();
                    point.y *= factor;
                    point
                })
                .collect(),
            Transform::OffsetY(offset) => data
                .iter()
                .map(|p| {
                    let mut point = p.clone();
                    point.y += offset;
                    point
                })
                .collect(),
            Transform::NormalizeY => Self::apply_normalize_y(data),
            Transform::RemoveInvalid => data.iter().filter(|p| p.y.is_finite()).cloned().collect(),
            Transform::SortByX => {
                let mut sorted = data.to_vec();
                sorted.sort_by(|a, b| {
                    let ax = a.x.unwrap_or(0.0);
                    let bx = b.x.unwrap_or(0.0);
                    ax.partial_cmp(&bx).unwrap_or(std::cmp::Ordering::Equal)
                });
                sorted
            }
            Transform::SortByY => {
                let mut sorted = data.to_vec();
                sorted.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal));
                sorted
            }
            Transform::Reverse => data.iter().rev().cloned().collect(),
            Transform::Dedupe => Self::apply_dedupe(data),
        }
    }

    fn apply_moving_average(data: &[DataPoint], window: usize) -> Vec<DataPoint> {
        if data.is_empty() || window == 0 {
            return data.to_vec();
        }

        data.iter()
            .enumerate()
            .map(|(i, p)| {
                let start = i.saturating_sub(window - 1);
                let sum: f64 = data[start..=i].iter().map(|p| p.y).sum();
                let count = (i - start + 1) as f64;
                let mut point = p.clone();
                point.y = sum / count;
                point
            })
            .collect()
    }

    fn apply_normalize_y(data: &[DataPoint]) -> Vec<DataPoint> {
        if data.is_empty() {
            return Vec::new();
        }

        let min = data.iter().map(|p| p.y).fold(f64::INFINITY, f64::min);
        let max = data.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max);
        let range = max - min;

        if range == 0.0 {
            return data
                .iter()
                .map(|p| {
                    let mut point = p.clone();
                    point.y = 0.5;
                    point
                })
                .collect();
        }

        data.iter()
            .map(|p| {
                let mut point = p.clone();
                point.y = (point.y - min) / range;
                point
            })
            .collect()
    }

    fn apply_dedupe(data: &[DataPoint]) -> Vec<DataPoint> {
        if data.is_empty() {
            return Vec::new();
        }

        let mut result = Vec::with_capacity(data.len());
        let mut last_y = f64::NAN;

        for point in data {
            if (point.y - last_y).abs() > f64::EPSILON || last_y.is_nan() {
                result.push(point.clone());
                last_y = point.y;
            }
        }

        result
    }

    /// Get number of transforms
    pub fn len(&self) -> usize {
        self.transforms.len()
    }

    /// Check if pipeline is empty
    pub fn is_empty(&self) -> bool {
        self.transforms.is_empty()
    }

    /// Clear all transforms
    pub fn clear(&mut self) {
        self.transforms.clear();
    }
}

impl Clone for DataPipeline {
    fn clone(&self) -> Self {
        // Note: Due to the boxed closures, we can only clone static transforms
        // This is a limitation - for full cloning, transforms would need to be
        // represented as data rather than closures
        Self {
            transforms: Vec::new(), // Cannot clone closures
        }
    }
}

/// Aggregation functions for data reduction
#[derive(Clone, Copy, Debug)]
pub enum Aggregation {
    /// Sum of values
    Sum,
    /// Average of values
    Mean,
    /// Minimum value
    Min,
    /// Maximum value
    Max,
    /// Count of values
    Count,
    /// First value
    First,
    /// Last value
    Last,
    /// Median value
    Median,
}

impl Aggregation {
    /// Apply aggregation to data
    pub fn apply(&self, data: &[DataPoint]) -> Option<f64> {
        if data.is_empty() {
            return None;
        }

        match self {
            Aggregation::Sum => Some(data.iter().map(|p| p.y).sum()),
            Aggregation::Mean => {
                let sum: f64 = data.iter().map(|p| p.y).sum();
                Some(sum / data.len() as f64)
            }
            Aggregation::Min => data
                .iter()
                .map(|p| p.y)
                .fold(None, |acc, y| Some(acc.map_or(y, |a: f64| a.min(y)))),
            Aggregation::Max => data
                .iter()
                .map(|p| p.y)
                .fold(None, |acc, y| Some(acc.map_or(y, |a: f64| a.max(y)))),
            Aggregation::Count => Some(data.len() as f64),
            Aggregation::First => data.first().map(|p| p.y),
            Aggregation::Last => data.last().map(|p| p.y),
            Aggregation::Median => {
                let mut values: Vec<f64> = data.iter().map(|p| p.y).collect();
                values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                let mid = values.len() / 2;
                if values.len() % 2 == 0 {
                    Some((values[mid - 1] + values[mid]) / 2.0)
                } else {
                    Some(values[mid])
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_data() -> Vec<DataPoint> {
        vec![
            DataPoint::from_y(10.0),
            DataPoint::from_y(20.0),
            DataPoint::from_y(30.0),
            DataPoint::from_y(40.0),
            DataPoint::from_y(50.0),
        ]
    }

    #[test]
    fn test_filter() {
        let pipeline = DataPipeline::new().filter(|p| p.y > 25.0);
        let result = pipeline.apply(&sample_data());

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].y, 30.0);
    }

    #[test]
    fn test_map() {
        let pipeline = DataPipeline::new().map(|p| {
            let mut point = p.clone();
            point.y *= 2.0;
            point
        });
        let result = pipeline.apply(&sample_data());

        assert_eq!(result[0].y, 20.0);
        assert_eq!(result[4].y, 100.0);
    }

    #[test]
    fn test_window() {
        let pipeline = DataPipeline::new().window(3);
        let result = pipeline.apply(&sample_data());

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].y, 30.0);
        assert_eq!(result[2].y, 50.0);
    }

    #[test]
    fn test_skip_take() {
        let pipeline = DataPipeline::new().skip(1).take(3);
        let result = pipeline.apply(&sample_data());

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].y, 20.0);
        assert_eq!(result[2].y, 40.0);
    }

    #[test]
    fn test_sample() {
        let pipeline = DataPipeline::new().sample(2);
        let result = pipeline.apply(&sample_data());

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].y, 10.0);
        assert_eq!(result[1].y, 30.0);
        assert_eq!(result[2].y, 50.0);
    }

    #[test]
    fn test_moving_average() {
        let pipeline = DataPipeline::new().moving_average(3);
        let result = pipeline.apply(&sample_data());

        assert_eq!(result.len(), 5);
        assert!((result[2].y - 20.0).abs() < 0.001); // (10+20+30)/3
        assert!((result[4].y - 40.0).abs() < 0.001); // (30+40+50)/3
    }

    #[test]
    fn test_clamp_y() {
        let pipeline = DataPipeline::new().clamp_y(20.0, 40.0);
        let result = pipeline.apply(&sample_data());

        assert_eq!(result[0].y, 20.0); // Clamped from 10
        assert_eq!(result[4].y, 40.0); // Clamped from 50
    }

    #[test]
    fn test_scale_y() {
        let pipeline = DataPipeline::new().scale_y(0.1);
        let result = pipeline.apply(&sample_data());

        assert_eq!(result[0].y, 1.0);
        assert_eq!(result[4].y, 5.0);
    }

    #[test]
    fn test_offset_y() {
        let pipeline = DataPipeline::new().offset_y(-10.0);
        let result = pipeline.apply(&sample_data());

        assert_eq!(result[0].y, 0.0);
        assert_eq!(result[4].y, 40.0);
    }

    #[test]
    fn test_normalize_y() {
        let pipeline = DataPipeline::new().normalize_y();
        let result = pipeline.apply(&sample_data());

        assert!((result[0].y - 0.0).abs() < 0.001);
        assert!((result[4].y - 1.0).abs() < 0.001);
        assert!((result[2].y - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_remove_invalid() {
        let data = vec![
            DataPoint::from_y(10.0),
            DataPoint::from_y(f64::NAN),
            DataPoint::from_y(30.0),
            DataPoint::from_y(f64::INFINITY),
        ];
        let pipeline = DataPipeline::new().remove_invalid();
        let result = pipeline.apply(&data);

        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_sort_by_y() {
        let data = vec![
            DataPoint::from_y(30.0),
            DataPoint::from_y(10.0),
            DataPoint::from_y(20.0),
        ];
        let pipeline = DataPipeline::new().sort_by_y();
        let result = pipeline.apply(&data);

        assert_eq!(result[0].y, 10.0);
        assert_eq!(result[1].y, 20.0);
        assert_eq!(result[2].y, 30.0);
    }

    #[test]
    fn test_reverse() {
        let pipeline = DataPipeline::new().reverse();
        let result = pipeline.apply(&sample_data());

        assert_eq!(result[0].y, 50.0);
        assert_eq!(result[4].y, 10.0);
    }

    #[test]
    fn test_dedupe() {
        let data = vec![
            DataPoint::from_y(10.0),
            DataPoint::from_y(10.0),
            DataPoint::from_y(20.0),
            DataPoint::from_y(20.0),
            DataPoint::from_y(20.0),
            DataPoint::from_y(30.0),
        ];
        let pipeline = DataPipeline::new().dedupe();
        let result = pipeline.apply(&data);

        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_chained_transforms() {
        let pipeline = DataPipeline::new()
            .filter(|p| p.y > 15.0)
            .scale_y(2.0)
            .window(2);

        let result = pipeline.apply(&sample_data());

        // Filter: [20, 30, 40, 50]
        // Scale: [40, 60, 80, 100]
        // Window(2): [80, 100]
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].y, 80.0);
        assert_eq!(result[1].y, 100.0);
    }

    #[test]
    fn test_aggregation_sum() {
        let result = Aggregation::Sum.apply(&sample_data());
        assert_eq!(result, Some(150.0));
    }

    #[test]
    fn test_aggregation_mean() {
        let result = Aggregation::Mean.apply(&sample_data());
        assert_eq!(result, Some(30.0));
    }

    #[test]
    fn test_aggregation_min_max() {
        assert_eq!(Aggregation::Min.apply(&sample_data()), Some(10.0));
        assert_eq!(Aggregation::Max.apply(&sample_data()), Some(50.0));
    }

    #[test]
    fn test_aggregation_median() {
        let result = Aggregation::Median.apply(&sample_data());
        assert_eq!(result, Some(30.0));

        let even_data = vec![
            DataPoint::from_y(10.0),
            DataPoint::from_y(20.0),
            DataPoint::from_y(30.0),
            DataPoint::from_y(40.0),
        ];
        let result = Aggregation::Median.apply(&even_data);
        assert_eq!(result, Some(25.0));
    }
}

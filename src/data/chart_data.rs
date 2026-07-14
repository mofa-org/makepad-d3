//! Chart data container

use super::{DataPoint, Dataset};
use crate::error::D3Error;
use serde::{Deserialize, Serialize};

/// Container for all chart data
///
/// # Example
/// ```
/// use makepad_d3::data::{ChartData, Dataset};
///
/// let data = ChartData::new()
///     .with_labels(vec!["Jan", "Feb", "Mar", "Apr"])
///     .add_dataset(Dataset::new("Revenue").with_data(vec![100.0, 200.0, 150.0, 300.0]))
///     .add_dataset(Dataset::new("Expenses").with_data(vec![80.0, 120.0, 100.0, 180.0]));
/// ```
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ChartData {
    /// Category labels (x-axis for bar charts, legend for pie)
    pub labels: Vec<String>,

    /// Datasets to render
    pub datasets: Vec<Dataset>,
}

impl ChartData {
    /// Create new empty chart data
    pub fn new() -> Self {
        Self::default()
    }

    /// Set category labels
    pub fn with_labels<S: Into<String>>(mut self, labels: impl IntoIterator<Item = S>) -> Self {
        self.labels = labels.into_iter().map(Into::into).collect();
        self
    }

    /// Add a dataset
    pub fn add_dataset(mut self, dataset: Dataset) -> Self {
        self.datasets.push(dataset);
        self
    }

    /// Set datasets
    pub fn with_datasets(mut self, datasets: Vec<Dataset>) -> Self {
        self.datasets = datasets;
        self
    }

    /// Get Y extent across all visible datasets
    pub fn y_extent(&self) -> Option<(f64, f64)> {
        let mut min = f64::MAX;
        let mut max = f64::MIN;
        let mut found = false;

        for dataset in &self.datasets {
            if dataset.hidden {
                continue;
            }
            if let Some((d_min, d_max)) = dataset.y_extent() {
                min = min.min(d_min);
                max = max.max(d_max);
                found = true;
            }
        }

        if found {
            Some((min, max))
        } else {
            None
        }
    }

    /// Get X extent across all visible datasets
    pub fn x_extent(&self) -> Option<(f64, f64)> {
        let mut min = f64::MAX;
        let mut max = f64::MIN;
        let mut found = false;

        for dataset in &self.datasets {
            if dataset.hidden {
                continue;
            }
            if let Some((d_min, d_max)) = dataset.x_extent() {
                min = min.min(d_min);
                max = max.max(d_max);
                found = true;
            }
        }

        if found {
            Some((min, max))
        } else {
            None
        }
    }

    /// Get total Y value (for pie charts)
    pub fn total(&self) -> f64 {
        self.datasets
            .first()
            .map(|d| d.data.iter().filter(|p| p.y > 0.0).map(|p| p.y).sum())
            .unwrap_or(0.0)
    }

    /// Get number of data points in first dataset
    pub fn len(&self) -> usize {
        self.datasets.first().map(|d| d.len()).unwrap_or(0)
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.datasets.is_empty() || self.datasets.iter().all(|d| d.is_empty())
    }

    /// Get number of visible datasets
    pub fn visible_count(&self) -> usize {
        self.datasets.iter().filter(|d| !d.hidden).count()
    }

    /// Toggle dataset visibility
    pub fn toggle_dataset(&mut self, index: usize) {
        if let Some(dataset) = self.datasets.get_mut(index) {
            dataset.hidden = !dataset.hidden;
        }
    }

    /// Get a mutable reference to a dataset
    pub fn dataset_mut(&mut self, index: usize) -> Option<&mut Dataset> {
        self.datasets.get_mut(index)
    }

    /// Get a reference to a dataset
    pub fn dataset(&self, index: usize) -> Option<&Dataset> {
        self.datasets.get(index)
    }

    /// Validate data consistency
    pub fn validate(&self) -> Result<(), D3Error> {
        // Check labels match data length
        if !self.labels.is_empty() && !self.datasets.is_empty() {
            let expected_len = self.labels.len();
            for (i, dataset) in self.datasets.iter().enumerate() {
                if !dataset.data.is_empty() && dataset.data.len() != expected_len {
                    return Err(D3Error::invalid_data(format!(
                        "Dataset {} has {} points, expected {} (labels count)",
                        i,
                        dataset.data.len(),
                        expected_len
                    )));
                }
            }
        }
        Ok(())
    }

    /// Get maximum number of data points across all datasets
    pub fn max_points(&self) -> usize {
        self.datasets.iter().map(|d| d.len()).max().unwrap_or(0)
    }

    /// Check if data has been set
    pub fn has_data(&self) -> bool {
        !self.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chart_data_new() {
        let data = ChartData::new();
        assert!(data.is_empty());
        assert_eq!(data.len(), 0);
    }

    #[test]
    fn test_chart_data_with_labels() {
        let data = ChartData::new().with_labels(vec!["A", "B", "C"]);

        assert_eq!(data.labels.len(), 3);
        assert_eq!(data.labels[0], "A");
    }

    #[test]
    fn test_chart_data_add_dataset() {
        let data =
            ChartData::new().add_dataset(Dataset::new("Test").with_data(vec![1.0, 2.0, 3.0]));

        assert_eq!(data.datasets.len(), 1);
        assert_eq!(data.len(), 3);
    }

    #[test]
    fn test_chart_data_y_extent() {
        let data = ChartData::new()
            .add_dataset(Dataset::new("A").with_data(vec![0.0, 50.0]))
            .add_dataset(Dataset::new("B").with_data(vec![-10.0, 30.0]));

        let (min, max) = data.y_extent().unwrap();
        assert_eq!(min, -10.0);
        assert_eq!(max, 50.0);
    }

    #[test]
    fn test_chart_data_y_extent_skips_hidden() {
        let data = ChartData::new()
            .add_dataset(Dataset::new("visible").with_data(vec![0.0, 50.0]))
            .add_dataset(
                Dataset::new("hidden")
                    .with_data(vec![-100.0, 100.0])
                    .with_hidden(true),
            );

        let (min, max) = data.y_extent().unwrap();
        assert_eq!(min, 0.0);
        assert_eq!(max, 50.0);
    }

    #[test]
    fn test_chart_data_total() {
        let data =
            ChartData::new().add_dataset(Dataset::new("Pie").with_data(vec![30.0, 20.0, 50.0]));

        assert!((data.total() - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_chart_data_toggle_dataset() {
        let mut data = ChartData::new().add_dataset(Dataset::new("Test").with_data(vec![1.0]));

        assert!(!data.datasets[0].hidden);
        data.toggle_dataset(0);
        assert!(data.datasets[0].hidden);
        data.toggle_dataset(0);
        assert!(!data.datasets[0].hidden);
    }

    #[test]
    fn test_chart_data_validation() {
        let data = ChartData::new()
            .with_labels(vec!["A", "B", "C"])
            .add_dataset(Dataset::new("test").with_data(vec![1.0, 2.0])); // Wrong length!

        assert!(data.validate().is_err());
    }

    #[test]
    fn test_chart_data_validation_ok() {
        let data = ChartData::new()
            .with_labels(vec!["A", "B", "C"])
            .add_dataset(Dataset::new("test").with_data(vec![1.0, 2.0, 3.0]));

        assert!(data.validate().is_ok());
    }

    #[test]
    fn test_serde_roundtrip() {
        let original = ChartData::new()
            .with_labels(vec!["A", "B"])
            .add_dataset(Dataset::new("test").with_data(vec![1.0, 2.0]));

        let json = serde_json::to_string(&original).unwrap();
        let parsed: ChartData = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.labels, original.labels);
        assert_eq!(parsed.datasets.len(), original.datasets.len());
    }

    #[test]
    fn test_visible_count() {
        let data = ChartData::new()
            .add_dataset(Dataset::new("A"))
            .add_dataset(Dataset::new("B").with_hidden(true))
            .add_dataset(Dataset::new("C"));

        assert_eq!(data.visible_count(), 2);
    }
}

//! Observable dataset with change notifications
//!
//! Provides a reactive dataset wrapper that notifies listeners when data changes.
//!
//! # Example
//!
//! ```
//! use makepad_d3::data::{ObservableDataset, DataPoint, DataChange};
//!
//! let mut dataset = ObservableDataset::new("Revenue");
//!
//! // Push data and check for changes
//! dataset.push(DataPoint::from_y(100.0));
//!
//! if let Some(change) = dataset.poll_change() {
//!     match change {
//!         DataChange::Append { .. } => println!("Data appended!"),
//!         _ => {}
//!     }
//! }
//! ```

use super::{Color, DataPoint, Dataset, PointStyle};
use std::collections::VecDeque;

/// Types of changes to the dataset
#[derive(Clone, Debug)]
pub enum DataChange {
    /// Data point(s) appended
    Append { start_index: usize, count: usize },
    /// Data point(s) updated
    Update { index: usize, count: usize },
    /// Data point(s) removed
    Remove { index: usize, count: usize },
    /// All data replaced
    Replace { old_count: usize, new_count: usize },
    /// Data cleared
    Clear { old_count: usize },
    /// Style changed
    StyleChange,
    /// Visibility changed
    VisibilityChange { hidden: bool },
}

/// Observable dataset that tracks and reports changes
///
/// Wraps a `Dataset` and maintains a queue of changes that can be polled
/// to trigger chart updates.
#[derive(Clone, Debug)]
pub struct ObservableDataset {
    /// Inner dataset
    inner: Dataset,
    /// Pending changes
    changes: VecDeque<DataChange>,
    /// Change counter (incremented on each change)
    version: u64,
    /// Whether to coalesce changes
    coalesce: bool,
}

impl ObservableDataset {
    /// Create a new observable dataset
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            inner: Dataset::new(label),
            changes: VecDeque::new(),
            version: 0,
            coalesce: false,
        }
    }

    /// Create from existing dataset
    pub fn from_dataset(dataset: Dataset) -> Self {
        Self {
            inner: dataset,
            changes: VecDeque::new(),
            version: 0,
            coalesce: false,
        }
    }

    /// Enable change coalescing (combine multiple changes into one)
    pub fn with_coalescing(mut self, coalesce: bool) -> Self {
        self.coalesce = coalesce;
        self
    }

    /// Get the inner dataset reference
    pub fn dataset(&self) -> &Dataset {
        &self.inner
    }

    /// Get mutable dataset reference (changes not tracked)
    pub fn dataset_mut(&mut self) -> &mut Dataset {
        &mut self.inner
    }

    /// Get current version number
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Check if there are pending changes
    pub fn has_changes(&self) -> bool {
        !self.changes.is_empty()
    }

    /// Poll for the next change
    pub fn poll_change(&mut self) -> Option<DataChange> {
        self.changes.pop_front()
    }

    /// Get all pending changes and clear queue
    pub fn drain_changes(&mut self) -> Vec<DataChange> {
        self.changes.drain(..).collect()
    }

    /// Clear pending changes without processing
    pub fn clear_changes(&mut self) {
        self.changes.clear();
    }

    // ========== Data modification methods ==========

    /// Push a single data point
    pub fn push(&mut self, point: DataPoint) {
        let start_index = self.inner.data.len();
        self.inner.data.push(point);
        self.record_change(DataChange::Append {
            start_index,
            count: 1,
        });
    }

    /// Push multiple data points
    pub fn push_many(&mut self, points: impl IntoIterator<Item = DataPoint>) {
        let start_index = self.inner.data.len();
        let points: Vec<_> = points.into_iter().collect();
        let count = points.len();
        self.inner.data.extend(points);
        if count > 0 {
            self.record_change(DataChange::Append { start_index, count });
        }
    }

    /// Set a data point at index
    pub fn set(&mut self, index: usize, point: DataPoint) {
        if index < self.inner.data.len() {
            self.inner.data[index] = point;
            self.record_change(DataChange::Update { index, count: 1 });
        }
    }

    /// Update multiple data points starting at index
    pub fn update_range(&mut self, index: usize, points: impl IntoIterator<Item = DataPoint>) {
        let points: Vec<_> = points.into_iter().collect();
        let count = points.len();
        for (i, point) in points.into_iter().enumerate() {
            if index + i < self.inner.data.len() {
                self.inner.data[index + i] = point;
            }
        }
        if count > 0 {
            self.record_change(DataChange::Update { index, count });
        }
    }

    /// Remove data point at index
    pub fn remove(&mut self, index: usize) {
        if index < self.inner.data.len() {
            self.inner.data.remove(index);
            self.record_change(DataChange::Remove { index, count: 1 });
        }
    }

    /// Remove range of data points
    pub fn remove_range(&mut self, index: usize, count: usize) {
        let actual_count = count.min(self.inner.data.len().saturating_sub(index));
        if actual_count > 0 {
            self.inner.data.drain(index..index + actual_count);
            self.record_change(DataChange::Remove {
                index,
                count: actual_count,
            });
        }
    }

    /// Replace all data
    pub fn replace(&mut self, points: impl IntoIterator<Item = DataPoint>) {
        let old_count = self.inner.data.len();
        self.inner.data = points.into_iter().collect();
        let new_count = self.inner.data.len();
        self.record_change(DataChange::Replace {
            old_count,
            new_count,
        });
    }

    /// Replace with y-values only
    pub fn replace_y_values(&mut self, values: impl IntoIterator<Item = f64>) {
        let old_count = self.inner.data.len();
        self.inner.data = values.into_iter().map(DataPoint::from_y).collect();
        let new_count = self.inner.data.len();
        self.record_change(DataChange::Replace {
            old_count,
            new_count,
        });
    }

    /// Replace with (x, y) pairs
    pub fn replace_xy_values(&mut self, values: impl IntoIterator<Item = (f64, f64)>) {
        let old_count = self.inner.data.len();
        self.inner.data = values.into_iter().map(DataPoint::from).collect();
        let new_count = self.inner.data.len();
        self.record_change(DataChange::Replace {
            old_count,
            new_count,
        });
    }

    /// Clear all data
    pub fn clear(&mut self) {
        let old_count = self.inner.data.len();
        self.inner.data.clear();
        self.record_change(DataChange::Clear { old_count });
    }

    /// Trim to maximum number of points (removes from front)
    pub fn trim_to(&mut self, max_points: usize) {
        if self.inner.data.len() > max_points {
            let remove_count = self.inner.data.len() - max_points;
            self.inner.data.drain(0..remove_count);
            self.record_change(DataChange::Remove {
                index: 0,
                count: remove_count,
            });
        }
    }

    // ========== Style modification methods ==========

    /// Set label
    pub fn set_label(&mut self, label: impl Into<String>) {
        self.inner.label = label.into();
        self.record_change(DataChange::StyleChange);
    }

    /// Set background color
    pub fn set_color(&mut self, color: Color) {
        self.inner.background_color = Some(color);
        self.record_change(DataChange::StyleChange);
    }

    /// Set border color
    pub fn set_border_color(&mut self, color: Color) {
        self.inner.border_color = Some(color);
        self.record_change(DataChange::StyleChange);
    }

    /// Set hidden state
    pub fn set_hidden(&mut self, hidden: bool) {
        if self.inner.hidden != hidden {
            self.inner.hidden = hidden;
            self.record_change(DataChange::VisibilityChange { hidden });
        }
    }

    /// Toggle visibility
    pub fn toggle_visibility(&mut self) {
        self.inner.hidden = !self.inner.hidden;
        self.record_change(DataChange::VisibilityChange {
            hidden: self.inner.hidden,
        });
    }

    /// Set line tension
    pub fn set_tension(&mut self, tension: f64) {
        self.inner.tension = tension.clamp(0.0, 1.0);
        self.record_change(DataChange::StyleChange);
    }

    /// Set point radius
    pub fn set_point_radius(&mut self, radius: f64) {
        self.inner.point_radius = radius;
        self.record_change(DataChange::StyleChange);
    }

    /// Set point style
    pub fn set_point_style(&mut self, style: PointStyle) {
        self.inner.point_style = style;
        self.record_change(DataChange::StyleChange);
    }

    /// Set fill enabled
    pub fn set_fill(&mut self, fill: bool) {
        self.inner.fill = fill;
        self.record_change(DataChange::StyleChange);
    }

    // ========== Accessor methods ==========

    /// Get label
    pub fn label(&self) -> &str {
        &self.inner.label
    }

    /// Get data points
    pub fn data(&self) -> &[DataPoint] {
        &self.inner.data
    }

    /// Get data point at index
    pub fn get(&self, index: usize) -> Option<&DataPoint> {
        self.inner.data.get(index)
    }

    /// Get number of data points
    pub fn len(&self) -> usize {
        self.inner.data.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.inner.data.is_empty()
    }

    /// Check if hidden
    pub fn is_hidden(&self) -> bool {
        self.inner.hidden
    }

    /// Get Y extent
    pub fn y_extent(&self) -> Option<(f64, f64)> {
        self.inner.y_extent()
    }

    /// Get X extent
    pub fn x_extent(&self) -> Option<(f64, f64)> {
        self.inner.x_extent()
    }

    // ========== Internal methods ==========

    fn record_change(&mut self, change: DataChange) {
        self.version += 1;

        if self.coalesce && !self.changes.is_empty() {
            // Try to coalesce with last change
            if let Some(last) = self.changes.back_mut() {
                if Self::can_coalesce(last, &change) {
                    Self::coalesce_change(last, change);
                    return;
                }
            }
        }

        self.changes.push_back(change);
    }

    fn can_coalesce(existing: &DataChange, new: &DataChange) -> bool {
        matches!(
            (existing, new),
            (DataChange::Append { .. }, DataChange::Append { .. })
                | (DataChange::StyleChange, DataChange::StyleChange)
        )
    }

    fn coalesce_change(existing: &mut DataChange, new: DataChange) {
        match (existing, new) {
            (DataChange::Append { count: c1, .. }, DataChange::Append { count: c2, .. }) => {
                *c1 += c2;
            }
            _ => {}
        }
    }
}

impl Default for ObservableDataset {
    fn default() -> Self {
        Self::new("")
    }
}

impl From<Dataset> for ObservableDataset {
    fn from(dataset: Dataset) -> Self {
        Self::from_dataset(dataset)
    }
}

impl From<ObservableDataset> for Dataset {
    fn from(observable: ObservableDataset) -> Self {
        observable.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observable_push() {
        let mut ds = ObservableDataset::new("Test");
        ds.push(DataPoint::from_y(10.0));

        assert_eq!(ds.len(), 1);
        assert!(ds.has_changes());

        let change = ds.poll_change().unwrap();
        assert!(matches!(
            change,
            DataChange::Append {
                start_index: 0,
                count: 1
            }
        ));
    }

    #[test]
    fn test_observable_push_many() {
        let mut ds = ObservableDataset::new("Test");
        ds.push_many(vec![
            DataPoint::from_y(10.0),
            DataPoint::from_y(20.0),
            DataPoint::from_y(30.0),
        ]);

        assert_eq!(ds.len(), 3);

        let change = ds.poll_change().unwrap();
        assert!(matches!(
            change,
            DataChange::Append {
                start_index: 0,
                count: 3
            }
        ));
    }

    #[test]
    fn test_observable_set() {
        let mut ds = ObservableDataset::new("Test");
        ds.push(DataPoint::from_y(10.0));
        ds.clear_changes();

        ds.set(0, DataPoint::from_y(100.0));

        assert_eq!(ds.data()[0].y, 100.0);
        let change = ds.poll_change().unwrap();
        assert!(matches!(change, DataChange::Update { index: 0, count: 1 }));
    }

    #[test]
    fn test_observable_remove() {
        let mut ds = ObservableDataset::new("Test");
        ds.push_many(vec![
            DataPoint::from_y(10.0),
            DataPoint::from_y(20.0),
            DataPoint::from_y(30.0),
        ]);
        ds.clear_changes();

        ds.remove(1);

        assert_eq!(ds.len(), 2);
        assert_eq!(ds.data()[0].y, 10.0);
        assert_eq!(ds.data()[1].y, 30.0);

        let change = ds.poll_change().unwrap();
        assert!(matches!(change, DataChange::Remove { index: 1, count: 1 }));
    }

    #[test]
    fn test_observable_replace() {
        let mut ds = ObservableDataset::new("Test");
        ds.push_many(vec![DataPoint::from_y(10.0), DataPoint::from_y(20.0)]);
        ds.clear_changes();

        ds.replace(vec![DataPoint::from_y(100.0)]);

        assert_eq!(ds.len(), 1);
        let change = ds.poll_change().unwrap();
        assert!(matches!(
            change,
            DataChange::Replace {
                old_count: 2,
                new_count: 1
            }
        ));
    }

    #[test]
    fn test_observable_clear() {
        let mut ds = ObservableDataset::new("Test");
        ds.push_many(vec![DataPoint::from_y(10.0), DataPoint::from_y(20.0)]);
        ds.clear_changes();

        ds.clear();

        assert!(ds.is_empty());
        let change = ds.poll_change().unwrap();
        assert!(matches!(change, DataChange::Clear { old_count: 2 }));
    }

    #[test]
    fn test_observable_version() {
        let mut ds = ObservableDataset::new("Test");
        assert_eq!(ds.version(), 0);

        ds.push(DataPoint::from_y(10.0));
        assert_eq!(ds.version(), 1);

        ds.push(DataPoint::from_y(20.0));
        assert_eq!(ds.version(), 2);
    }

    #[test]
    fn test_observable_coalescing() {
        let mut ds = ObservableDataset::new("Test").with_coalescing(true);

        ds.push(DataPoint::from_y(10.0));
        ds.push(DataPoint::from_y(20.0));
        ds.push(DataPoint::from_y(30.0));

        // Should be coalesced into single append
        let changes: Vec<_> = ds.drain_changes();
        assert_eq!(changes.len(), 1);
        assert!(matches!(changes[0], DataChange::Append { count: 3, .. }));
    }

    #[test]
    fn test_observable_visibility() {
        let mut ds = ObservableDataset::new("Test");
        ds.clear_changes();

        ds.set_hidden(true);
        assert!(ds.is_hidden());

        let change = ds.poll_change().unwrap();
        assert!(matches!(
            change,
            DataChange::VisibilityChange { hidden: true }
        ));
    }

    #[test]
    fn test_observable_trim() {
        let mut ds = ObservableDataset::new("Test");
        ds.push_many((0..10).map(|i| DataPoint::from_y(i as f64)));
        ds.clear_changes();

        ds.trim_to(5);

        assert_eq!(ds.len(), 5);
        assert_eq!(ds.data()[0].y, 5.0); // First 5 removed
    }

    #[test]
    fn test_from_dataset() {
        let dataset = Dataset::new("Original").with_data(vec![10.0, 20.0, 30.0]);

        let observable = ObservableDataset::from_dataset(dataset);

        assert_eq!(observable.label(), "Original");
        assert_eq!(observable.len(), 3);
    }
}

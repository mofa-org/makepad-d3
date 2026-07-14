//! Pie layout generator
//!
//! Computes pie slice angles from data values for use with the arc generator.

use std::cmp::Ordering;
use std::f64::consts::TAU;

/// A computed pie slice with angle information
#[derive(Clone, Debug)]
pub struct PieSlice<T> {
    /// The original data value
    pub data: T,
    /// The numeric value used for sizing
    pub value: f64,
    /// Index in the original data array
    pub index: usize,
    /// Start angle in radians
    pub start_angle: f64,
    /// End angle in radians
    pub end_angle: f64,
    /// Padding angle between this slice and adjacent slices
    pub pad_angle: f64,
}

impl<T> PieSlice<T> {
    /// Get the angular span of this slice
    pub fn angle(&self) -> f64 {
        self.end_angle - self.start_angle
    }
}

/// Sort order for pie slices
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PieSort {
    /// No sorting, maintain original order
    #[default]
    None,
    /// Sort by value ascending (smallest first)
    ValueAscending,
    /// Sort by value descending (largest first)
    ValueDescending,
    /// Sort by index ascending
    IndexAscending,
    /// Sort by index descending
    IndexDescending,
}

/// Pie layout generator
///
/// Computes pie/donut slice angles from numeric data values.
///
/// # Example
/// ```
/// use makepad_d3::shape::PieLayout;
///
/// let values = vec![10.0, 20.0, 30.0, 40.0];
/// let pie = PieLayout::new();
/// let slices = pie.compute(&values);
///
/// assert_eq!(slices.len(), 4);
/// // First slice starts at 0
/// assert!((slices[0].start_angle - 0.0).abs() < 0.01);
/// ```
#[derive(Clone, Debug)]
pub struct PieLayout {
    /// Start angle for the entire pie
    start_angle: f64,
    /// End angle for the entire pie
    end_angle: f64,
    /// Padding angle between slices
    pad_angle: f64,
    /// Sort order
    sort: PieSort,
}

impl Default for PieLayout {
    fn default() -> Self {
        Self::new()
    }
}

impl PieLayout {
    /// Create a new pie layout with default settings
    pub fn new() -> Self {
        Self {
            start_angle: 0.0,
            end_angle: TAU,
            pad_angle: 0.0,
            sort: PieSort::None,
        }
    }

    /// Set the start angle
    pub fn start_angle(mut self, angle: f64) -> Self {
        self.start_angle = angle;
        self
    }

    /// Set the end angle
    pub fn end_angle(mut self, angle: f64) -> Self {
        self.end_angle = angle;
        self
    }

    /// Set the pad angle between slices
    pub fn pad_angle(mut self, angle: f64) -> Self {
        self.pad_angle = angle.max(0.0);
        self
    }

    /// Set the sort order
    pub fn sort(mut self, order: PieSort) -> Self {
        self.sort = order;
        self
    }

    /// Compute pie slices from values
    pub fn compute(&self, values: &[f64]) -> Vec<PieSlice<f64>> {
        self.compute_with_data(values, |&v| v)
    }

    /// Compute pie slices from data with a value accessor
    pub fn compute_with_data<T, F>(&self, data: &[T], value_fn: F) -> Vec<PieSlice<T>>
    where
        T: Clone,
        F: Fn(&T) -> f64,
    {
        if data.is_empty() {
            return vec![];
        }

        // Extract values
        let values: Vec<f64> = data.iter().map(&value_fn).collect();

        // Calculate total (only positive values)
        let total: f64 = values.iter().filter(|&&v| v > 0.0).sum();

        if total <= 0.0 {
            return vec![];
        }

        // Calculate available angle after padding
        let range = self.end_angle - self.start_angle;
        let n = values.iter().filter(|&&v| v > 0.0).count();
        let total_pad = self.pad_angle * n as f64;
        let value_range = (range - total_pad).max(0.0);

        // Create indexed values for sorting
        let mut indexed: Vec<(usize, f64, T)> = data
            .iter()
            .enumerate()
            .map(|(i, d)| (i, values[i], d.clone()))
            .collect();

        // Apply sorting
        match self.sort {
            PieSort::None => {}
            PieSort::ValueAscending => {
                indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));
            }
            PieSort::ValueDescending => {
                indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
            }
            PieSort::IndexAscending => {
                indexed.sort_by_key(|a| a.0);
            }
            PieSort::IndexDescending => {
                indexed.sort_by(|a, b| b.0.cmp(&a.0));
            }
        }

        // Generate slices
        let mut slices = Vec::with_capacity(data.len());
        let mut angle = self.start_angle;

        for (index, value, data) in indexed {
            let slice_angle = if value > 0.0 {
                (value / total) * value_range
            } else {
                0.0
            };

            slices.push(PieSlice {
                data,
                value,
                index,
                start_angle: angle,
                end_angle: angle + slice_angle,
                pad_angle: self.pad_angle,
            });

            angle += slice_angle + self.pad_angle;
        }

        slices
    }

    /// Create a half-pie (semicircle) layout
    pub fn half() -> Self {
        Self::new().start_angle(0.0).end_angle(std::f64::consts::PI)
    }

    /// Create a three-quarter pie layout
    pub fn three_quarter() -> Self {
        Self::new()
            .start_angle(0.0)
            .end_angle(std::f64::consts::PI * 1.5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pie_layout_basic() {
        let values = vec![1.0, 1.0, 1.0, 1.0];
        let pie = PieLayout::new();
        let slices = pie.compute(&values);

        assert_eq!(slices.len(), 4);

        // Each slice should be 1/4 of the total
        let quarter = TAU / 4.0;
        for slice in &slices {
            assert!((slice.angle() - quarter).abs() < 0.01);
        }
    }

    #[test]
    fn test_pie_layout_different_values() {
        let values = vec![10.0, 20.0, 30.0, 40.0];
        let pie = PieLayout::new();
        let slices = pie.compute(&values);

        // Total is 100, so slices should be 10%, 20%, 30%, 40%
        assert!((slices[0].angle() - TAU * 0.1).abs() < 0.01);
        assert!((slices[1].angle() - TAU * 0.2).abs() < 0.01);
        assert!((slices[2].angle() - TAU * 0.3).abs() < 0.01);
        assert!((slices[3].angle() - TAU * 0.4).abs() < 0.01);
    }

    #[test]
    fn test_pie_layout_with_padding() {
        let values = vec![1.0, 1.0, 1.0, 1.0];
        let pie = PieLayout::new().pad_angle(0.1);
        let slices = pie.compute(&values);

        assert_eq!(slices.len(), 4);

        // Total padding is 0.4, so value range is TAU - 0.4
        let value_range = TAU - 0.4;
        let expected_angle = value_range / 4.0;

        for slice in &slices {
            assert!((slice.angle() - expected_angle).abs() < 0.01);
        }
    }

    #[test]
    fn test_pie_layout_sorted() {
        let values = vec![30.0, 10.0, 40.0, 20.0];
        let pie = PieLayout::new().sort(PieSort::ValueDescending);
        let slices = pie.compute(&values);

        // Should be sorted by value descending
        assert_eq!(slices[0].value, 40.0);
        assert_eq!(slices[1].value, 30.0);
        assert_eq!(slices[2].value, 20.0);
        assert_eq!(slices[3].value, 10.0);
    }

    #[test]
    fn test_pie_layout_custom_angles() {
        let values = vec![1.0, 1.0];
        let pie = PieLayout::new()
            .start_angle(0.0)
            .end_angle(std::f64::consts::PI);
        let slices = pie.compute(&values);

        assert_eq!(slices.len(), 2);

        // Half pie, so each slice is PI/2
        let expected = std::f64::consts::PI / 2.0;
        assert!((slices[0].angle() - expected).abs() < 0.01);
        assert!((slices[1].angle() - expected).abs() < 0.01);
    }

    #[test]
    fn test_pie_layout_empty() {
        let pie = PieLayout::new();
        let slices = pie.compute(&[]);
        assert!(slices.is_empty());
    }

    #[test]
    fn test_pie_layout_zero_values() {
        let values = vec![0.0, 0.0, 0.0];
        let pie = PieLayout::new();
        let slices = pie.compute(&values);
        assert!(slices.is_empty()); // All zeros = no slices
    }

    #[test]
    fn test_pie_layout_with_data() {
        #[derive(Clone)]
        struct Item {
            name: &'static str,
            count: f64,
        }

        let items = vec![
            Item {
                name: "A",
                count: 10.0,
            },
            Item {
                name: "B",
                count: 20.0,
            },
            Item {
                name: "C",
                count: 30.0,
            },
        ];

        let pie = PieLayout::new();
        let slices = pie.compute_with_data(&items, |item| item.count);

        assert_eq!(slices.len(), 3);
        assert_eq!(slices[0].data.name, "A");
        assert_eq!(slices[1].data.name, "B");
        assert_eq!(slices[2].data.name, "C");
    }

    #[test]
    fn test_pie_half() {
        let values = vec![1.0, 1.0];
        let pie = PieLayout::half();
        let slices = pie.compute(&values);

        let total_angle: f64 = slices.iter().map(|s| s.angle()).sum();
        assert!((total_angle - std::f64::consts::PI).abs() < 0.01);
    }
}

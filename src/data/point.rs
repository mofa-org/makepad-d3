//! Data point representation

use serde::{Deserialize, Serialize};

/// A single data point in a chart
///
/// # Example
/// ```
/// use makepad_d3::data::DataPoint;
///
/// // Simple y-value point
/// let p1 = DataPoint::from_y(42.0);
///
/// // X-Y coordinate point
/// let p2 = DataPoint::new(1.0, 2.0);
///
/// // Bubble chart point with radius
/// let p3 = DataPoint::bubble(1.0, 2.0, 10.0);
/// ```
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DataPoint {
    /// X coordinate value (None = use index)
    pub x: Option<f64>,

    /// Y coordinate value (primary value)
    pub y: f64,

    /// Y minimum for floating bars/ranges
    pub y_min: Option<f64>,

    /// Radius for bubble charts
    pub r: Option<f64>,

    /// Display label
    pub label: Option<String>,

    /// Additional metadata (for tooltips)
    pub meta: Option<String>,
}

impl DataPoint {
    /// Create a new data point with x and y values
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            x: Some(x),
            y,
            ..Default::default()
        }
    }

    /// Create from y value only (x will be index)
    pub fn from_y(y: f64) -> Self {
        Self {
            y,
            ..Default::default()
        }
    }

    /// Create a floating/range data point
    pub fn range(y_min: f64, y_max: f64) -> Self {
        Self {
            y: y_max,
            y_min: Some(y_min),
            ..Default::default()
        }
    }

    /// Create a bubble data point
    pub fn bubble(x: f64, y: f64, r: f64) -> Self {
        Self {
            x: Some(x),
            y,
            r: Some(r),
            ..Default::default()
        }
    }

    /// Builder: set label
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Builder: set meta
    pub fn with_meta(mut self, meta: impl Into<String>) -> Self {
        self.meta = Some(meta.into());
        self
    }

    /// Get effective X value (or index if None)
    pub fn x_or(&self, index: usize) -> f64 {
        self.x.unwrap_or(index as f64)
    }

    /// Get Y range (y_min to y)
    pub fn y_range(&self) -> (f64, f64) {
        (self.y_min.unwrap_or(0.0), self.y)
    }

    /// Check if this point has valid finite values
    pub fn is_valid(&self) -> bool {
        self.y.is_finite() && self.x.map(|x| x.is_finite()).unwrap_or(true)
    }
}

// Convenience conversions
impl From<f64> for DataPoint {
    fn from(y: f64) -> Self {
        Self::from_y(y)
    }
}

impl From<(f64, f64)> for DataPoint {
    fn from((x, y): (f64, f64)) -> Self {
        Self::new(x, y)
    }
}

impl From<(f64, f64, f64)> for DataPoint {
    fn from((x, y, r): (f64, f64, f64)) -> Self {
        Self::bubble(x, y, r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_point_from_y() {
        let point = DataPoint::from_y(42.0);
        assert_eq!(point.y, 42.0);
        assert_eq!(point.x, None);
    }

    #[test]
    fn test_data_point_new() {
        let point = DataPoint::new(1.0, 2.0);
        assert_eq!(point.x, Some(1.0));
        assert_eq!(point.y, 2.0);
    }

    #[test]
    fn test_data_point_bubble() {
        let point = DataPoint::bubble(1.0, 2.0, 3.0);
        assert_eq!(point.x, Some(1.0));
        assert_eq!(point.y, 2.0);
        assert_eq!(point.r, Some(3.0));
    }

    #[test]
    fn test_data_point_range() {
        let point = DataPoint::range(-5.0, 10.0);
        assert_eq!(point.y_min, Some(-5.0));
        assert_eq!(point.y, 10.0);
        assert_eq!(point.y_range(), (-5.0, 10.0));
    }

    #[test]
    fn test_data_point_conversions() {
        let p1: DataPoint = 42.0.into();
        assert_eq!(p1.y, 42.0);

        let p2: DataPoint = (1.0, 2.0).into();
        assert_eq!(p2.x, Some(1.0));
        assert_eq!(p2.y, 2.0);

        let p3: DataPoint = (1.0, 2.0, 3.0).into();
        assert_eq!(p3.r, Some(3.0));
    }

    #[test]
    fn test_x_or() {
        let p1 = DataPoint::from_y(10.0);
        assert_eq!(p1.x_or(5), 5.0);

        let p2 = DataPoint::new(3.0, 10.0);
        assert_eq!(p2.x_or(5), 3.0);
    }

    #[test]
    fn test_with_label() {
        let point = DataPoint::from_y(10.0).with_label("Test");
        assert_eq!(point.label, Some("Test".to_string()));
    }

    #[test]
    fn test_is_valid() {
        assert!(DataPoint::new(1.0, 2.0).is_valid());
        assert!(!DataPoint::new(f64::NAN, 2.0).is_valid());
        assert!(!DataPoint::from_y(f64::INFINITY).is_valid());
    }

    #[test]
    fn test_serde_roundtrip() {
        let original = DataPoint::new(1.0, 2.0).with_label("test");
        let json = serde_json::to_string(&original).unwrap();
        let parsed: DataPoint = serde_json::from_str(&json).unwrap();
        assert_eq!(original, parsed);
    }
}

//! Dataset representation

use super::DataPoint;
use serde::{Deserialize, Serialize};

/// Point marker styles for scatter/line charts
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PointStyle {
    #[default]
    Circle,
    Square,
    Triangle,
    Diamond,
    Cross,
    Star,
    None,
}

/// RGBA color representation for serialization
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    /// Create a new color from RGBA values (0.0 - 1.0)
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Create a color from RGB values with full opacity
    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    /// Create a color from hex value (0xRRGGBB)
    pub fn from_hex(hex: u32) -> Self {
        let r = ((hex >> 16) & 0xFF) as f32 / 255.0;
        let g = ((hex >> 8) & 0xFF) as f32 / 255.0;
        let b = (hex & 0xFF) as f32 / 255.0;
        Self { r, g, b, a: 1.0 }
    }

    /// Create a color from hex value with alpha (0xRRGGBBAA)
    pub fn from_hex_alpha(hex: u32) -> Self {
        let r = ((hex >> 24) & 0xFF) as f32 / 255.0;
        let g = ((hex >> 16) & 0xFF) as f32 / 255.0;
        let b = ((hex >> 8) & 0xFF) as f32 / 255.0;
        let a = (hex & 0xFF) as f32 / 255.0;
        Self { r, g, b, a }
    }

    /// Standard colors
    pub const RED: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const GREEN: Color = Color {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };
    pub const BLUE: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };
    pub const BLACK: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const WHITE: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    pub const TRANSPARENT: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };
}

/// A dataset containing multiple data points with styling
///
/// # Example
/// ```
/// use makepad_d3::data::{Dataset, PointStyle};
///
/// let dataset = Dataset::new("Revenue")
///     .with_data(vec![10.0, 20.0, 30.0, 40.0])
///     .with_hex_color(0x4285F4)
///     .with_tension(0.4);
/// ```
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Dataset {
    /// Display label for this dataset
    pub label: String,

    /// Data points
    pub data: Vec<DataPoint>,

    /// Background/fill color
    pub background_color: Option<Color>,

    /// Border/stroke color
    pub border_color: Option<Color>,

    /// Border width in pixels
    pub border_width: f64,

    /// Whether this dataset is hidden
    pub hidden: bool,

    // Line chart options
    /// Fill area under line
    pub fill: bool,

    /// Line tension (0 = straight, 0.4 = smooth)
    pub tension: f64,

    // Point options
    /// Point radius in pixels
    pub point_radius: f64,

    /// Point style
    pub point_style: PointStyle,

    // Bar chart options
    /// Bar width as fraction of category width (0-1)
    pub bar_percent: f64,

    /// Bar border radius
    pub bar_radius: f64,
}

impl Dataset {
    /// Create a new dataset with a label
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            border_width: 2.0,
            tension: 0.0,
            point_radius: 3.0,
            point_style: PointStyle::Circle,
            bar_percent: 0.8,
            ..Default::default()
        }
    }

    /// Set data from y values
    pub fn with_data(mut self, data: impl IntoIterator<Item = f64>) -> Self {
        self.data = data.into_iter().map(DataPoint::from_y).collect();
        self
    }

    /// Set data from (x, y) pairs
    pub fn with_xy_data(mut self, data: impl IntoIterator<Item = (f64, f64)>) -> Self {
        self.data = data.into_iter().map(DataPoint::from).collect();
        self
    }

    /// Set data from DataPoints directly
    pub fn with_points(mut self, data: Vec<DataPoint>) -> Self {
        self.data = data;
        self
    }

    /// Set background color
    pub fn with_color(mut self, color: Color) -> Self {
        self.background_color = Some(color);
        self
    }

    /// Set background color from hex (0xRRGGBB)
    pub fn with_hex_color(mut self, hex: u32) -> Self {
        self.background_color = Some(Color::from_hex(hex));
        self
    }

    /// Set border color
    pub fn with_border_color(mut self, color: Color) -> Self {
        self.border_color = Some(color);
        self
    }

    /// Set border width
    pub fn with_border_width(mut self, width: f64) -> Self {
        self.border_width = width;
        self
    }

    /// Enable area fill
    pub fn with_fill(mut self, fill: bool) -> Self {
        self.fill = fill;
        self
    }

    /// Set line tension
    pub fn with_tension(mut self, tension: f64) -> Self {
        self.tension = tension.clamp(0.0, 1.0);
        self
    }

    /// Set point radius
    pub fn with_point_radius(mut self, radius: f64) -> Self {
        self.point_radius = radius;
        self
    }

    /// Set point style
    pub fn with_point_style(mut self, style: PointStyle) -> Self {
        self.point_style = style;
        self
    }

    /// Set hidden state
    pub fn with_hidden(mut self, hidden: bool) -> Self {
        self.hidden = hidden;
        self
    }

    /// Set bar percent
    pub fn with_bar_percent(mut self, percent: f64) -> Self {
        self.bar_percent = percent.clamp(0.0, 1.0);
        self
    }

    /// Get number of data points
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get Y extent (min, max)
    pub fn y_extent(&self) -> Option<(f64, f64)> {
        if self.data.is_empty() {
            return None;
        }

        let mut min = f64::MAX;
        let mut max = f64::MIN;

        for point in &self.data {
            if !point.y.is_finite() {
                continue;
            }
            if let Some(y_min) = point.y_min {
                if y_min.is_finite() {
                    min = min.min(y_min);
                }
            }
            min = min.min(point.y);
            max = max.max(point.y);
        }

        if min == f64::MAX || max == f64::MIN {
            None
        } else {
            Some((min, max))
        }
    }

    /// Get X extent (min, max)
    pub fn x_extent(&self) -> Option<(f64, f64)> {
        if self.data.is_empty() {
            return None;
        }

        let mut min = f64::MAX;
        let mut max = f64::MIN;

        for (i, point) in self.data.iter().enumerate() {
            let x = point.x_or(i);
            if !x.is_finite() {
                continue;
            }
            min = min.min(x);
            max = max.max(x);
        }

        if min == f64::MAX || max == f64::MIN {
            None
        } else {
            Some((min, max))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dataset_new() {
        let ds = Dataset::new("Test");
        assert_eq!(ds.label, "Test");
        assert!(ds.is_empty());
    }

    #[test]
    fn test_dataset_with_data() {
        let ds = Dataset::new("Test").with_data(vec![10.0, 20.0, 30.0]);

        assert_eq!(ds.len(), 3);
        assert_eq!(ds.data[0].y, 10.0);
        assert_eq!(ds.data[1].y, 20.0);
        assert_eq!(ds.data[2].y, 30.0);
    }

    #[test]
    fn test_dataset_y_extent() {
        let ds = Dataset::new("Test").with_data(vec![10.0, 50.0, 30.0, -5.0]);

        let (min, max) = ds.y_extent().unwrap();
        assert_eq!(min, -5.0);
        assert_eq!(max, 50.0);
    }

    #[test]
    fn test_dataset_y_extent_with_range() {
        let ds = Dataset::new("Test").with_points(vec![
            DataPoint::range(-10.0, 10.0),
            DataPoint::range(5.0, 20.0),
        ]);

        let (min, max) = ds.y_extent().unwrap();
        assert_eq!(min, -10.0);
        assert_eq!(max, 20.0);
    }

    #[test]
    fn test_dataset_x_extent() {
        let ds = Dataset::new("Test").with_xy_data(vec![(1.0, 10.0), (5.0, 20.0), (3.0, 30.0)]);

        let (min, max) = ds.x_extent().unwrap();
        assert_eq!(min, 1.0);
        assert_eq!(max, 5.0);
    }

    #[test]
    fn test_color_from_hex() {
        let c = Color::from_hex(0xFF0000);
        assert!((c.r - 1.0).abs() < 0.01);
        assert!((c.g - 0.0).abs() < 0.01);
        assert!((c.b - 0.0).abs() < 0.01);

        let c2 = Color::from_hex(0x4285F4);
        assert!((c2.r - 0.259).abs() < 0.01);
        assert!((c2.g - 0.522).abs() < 0.01);
        assert!((c2.b - 0.957).abs() < 0.01);
    }

    #[test]
    fn test_dataset_builder_chain() {
        let ds = Dataset::new("Revenue")
            .with_data(vec![10.0, 20.0])
            .with_hex_color(0x4285F4)
            .with_tension(0.4)
            .with_fill(true)
            .with_point_radius(5.0);

        assert_eq!(ds.label, "Revenue");
        assert_eq!(ds.len(), 2);
        assert!(ds.background_color.is_some());
        assert!((ds.tension - 0.4).abs() < 0.001);
        assert!(ds.fill);
        assert!((ds.point_radius - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_dataset_empty_extent() {
        let ds = Dataset::new("Empty");
        assert!(ds.y_extent().is_none());
        assert!(ds.x_extent().is_none());
    }
}

//! Reference line component for chart overlays
//!
//! Provides horizontal and vertical reference lines for marking
//! thresholds, averages, targets, and other significant values.
//!
//! # Example
//!
//! ```
//! use makepad_d3::component::{ReferenceLine, ReferenceLineStyle, ReferenceLineOrientation};
//! use makepad_d3::color::Rgba;
//!
//! // Create a horizontal threshold line
//! let threshold = ReferenceLine::horizontal(75.0, "Target: 75%")
//!     .with_style(ReferenceLineStyle::dashed(Rgba::from_hex(0xFF5722), 2.0));
//!
//! // Create vertical date marker
//! let marker = ReferenceLine::vertical(500.0, "Launch Date");
//! ```

use crate::color::Rgba;
use serde::{Deserialize, Serialize};

/// Orientation of the reference line
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReferenceLineOrientation {
    /// Horizontal line (across the chart width)
    #[default]
    Horizontal,
    /// Vertical line (across the chart height)
    Vertical,
}

/// Line dash style
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineDash {
    /// Solid line
    #[default]
    Solid,
    /// Dashed line
    Dashed,
    /// Dotted line
    Dotted,
    /// Long dash
    LongDash,
    /// Dash-dot pattern
    DashDot,
    /// Double dash
    DoubleDash,
}

impl LineDash {
    /// Get the dash array for this style
    pub fn dash_array(&self) -> Vec<f64> {
        match self {
            LineDash::Solid => Vec::new(),
            LineDash::Dashed => vec![6.0, 4.0],
            LineDash::Dotted => vec![2.0, 2.0],
            LineDash::LongDash => vec![12.0, 4.0],
            LineDash::DashDot => vec![6.0, 2.0, 2.0, 2.0],
            LineDash::DoubleDash => vec![8.0, 2.0, 2.0, 2.0, 2.0, 2.0],
        }
    }
}

/// Label position for reference lines
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum LabelAnchor {
    /// At the start of the line
    Start,
    /// At the center of the line
    #[default]
    Center,
    /// At the end of the line
    End,
    /// Above/left of the line
    Before,
    /// Below/right of the line
    After,
}

/// Style configuration for reference lines
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReferenceLineStyle {
    /// Line color
    #[serde(skip)]
    pub color: Rgba,
    /// Line width
    pub width: f64,
    /// Dash style
    pub dash: LineDash,
    /// Custom dash array (overrides dash if not empty)
    pub dash_array: Vec<f64>,
    /// Line opacity
    pub opacity: f32,
    /// Label background color
    #[serde(skip)]
    pub label_background: Rgba,
    /// Label text color
    #[serde(skip)]
    pub label_color: Rgba,
    /// Label font size
    pub label_font_size: f64,
    /// Label padding
    pub label_padding: f64,
    /// Label border radius
    pub label_border_radius: f64,
    /// Show label
    pub show_label: bool,
    /// Label anchor position
    pub label_anchor: LabelAnchor,
    /// Label offset from line
    pub label_offset: f64,
}

impl Default for ReferenceLineStyle {
    fn default() -> Self {
        Self {
            color: Rgba::new(0.5, 0.5, 0.5, 1.0),
            width: 1.0,
            dash: LineDash::Dashed,
            dash_array: Vec::new(),
            opacity: 1.0,
            label_background: Rgba::new(1.0, 1.0, 1.0, 0.9),
            label_color: Rgba::new(0.3, 0.3, 0.3, 1.0),
            label_font_size: 11.0,
            label_padding: 4.0,
            label_border_radius: 2.0,
            show_label: true,
            label_anchor: LabelAnchor::End,
            label_offset: 8.0,
        }
    }
}

impl ReferenceLineStyle {
    /// Create a solid line style
    pub fn solid(color: Rgba, width: f64) -> Self {
        Self {
            color,
            width,
            dash: LineDash::Solid,
            ..Default::default()
        }
    }

    /// Create a dashed line style
    pub fn dashed(color: Rgba, width: f64) -> Self {
        Self {
            color,
            width,
            dash: LineDash::Dashed,
            ..Default::default()
        }
    }

    /// Create a dotted line style
    pub fn dotted(color: Rgba, width: f64) -> Self {
        Self {
            color,
            width,
            dash: LineDash::Dotted,
            ..Default::default()
        }
    }

    /// Create a threshold/alert style (red, dashed)
    pub fn threshold() -> Self {
        Self {
            color: Rgba::from_hex(0xE53935),
            width: 2.0,
            dash: LineDash::Dashed,
            ..Default::default()
        }
    }

    /// Create a target/goal style (green, solid)
    pub fn target() -> Self {
        Self {
            color: Rgba::from_hex(0x43A047),
            width: 2.0,
            dash: LineDash::Solid,
            ..Default::default()
        }
    }

    /// Create an average/mean style (blue, dashed)
    pub fn average() -> Self {
        Self {
            color: Rgba::from_hex(0x1E88E5),
            width: 1.5,
            dash: LineDash::Dashed,
            ..Default::default()
        }
    }

    /// Create a baseline style (gray, dotted)
    pub fn baseline() -> Self {
        Self {
            color: Rgba::new(0.6, 0.6, 0.6, 1.0),
            width: 1.0,
            dash: LineDash::Dotted,
            ..Default::default()
        }
    }

    /// Set color
    pub fn color(mut self, color: Rgba) -> Self {
        self.color = color;
        self
    }

    /// Set width
    pub fn width(mut self, width: f64) -> Self {
        self.width = width;
        self
    }

    /// Set dash style
    pub fn dash(mut self, dash: LineDash) -> Self {
        self.dash = dash;
        self
    }

    /// Set custom dash array
    pub fn dash_array(mut self, array: Vec<f64>) -> Self {
        self.dash_array = array;
        self
    }

    /// Set opacity
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;
        self
    }

    /// Set label styling
    pub fn label_style(mut self, background: Rgba, text_color: Rgba, font_size: f64) -> Self {
        self.label_background = background;
        self.label_color = text_color;
        self.label_font_size = font_size;
        self
    }

    /// Show or hide label
    pub fn show_label(mut self, show: bool) -> Self {
        self.show_label = show;
        self
    }

    /// Set label anchor
    pub fn label_anchor(mut self, anchor: LabelAnchor) -> Self {
        self.label_anchor = anchor;
        self
    }

    /// Get effective dash array
    pub fn effective_dash_array(&self) -> Vec<f64> {
        if self.dash_array.is_empty() {
            self.dash.dash_array()
        } else {
            self.dash_array.clone()
        }
    }

    /// Get effective color with opacity
    pub fn effective_color(&self) -> Rgba {
        Rgba::new(
            self.color.r,
            self.color.g,
            self.color.b,
            self.color.a * self.opacity,
        )
    }
}

/// A reference line for chart overlays
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReferenceLine {
    /// Unique identifier
    pub id: String,
    /// Orientation
    pub orientation: ReferenceLineOrientation,
    /// Value position (Y for horizontal, X for vertical)
    pub value: f64,
    /// Screen position (computed from value and scale)
    pub position: f64,
    /// Label text
    pub label: String,
    /// Style configuration
    #[serde(skip)]
    pub style: ReferenceLineStyle,
    /// Whether the line is visible
    pub visible: bool,
    /// Whether the line is interactive
    pub interactive: bool,
    /// Start extent (None = chart boundary)
    pub start: Option<f64>,
    /// End extent (None = chart boundary)
    pub end: Option<f64>,
    /// Band width (for range bands instead of single lines)
    pub band_width: Option<f64>,
    /// Band fill color (for bands)
    #[serde(skip)]
    pub band_fill: Option<Rgba>,
}

impl Default for ReferenceLine {
    fn default() -> Self {
        Self {
            id: String::new(),
            orientation: ReferenceLineOrientation::Horizontal,
            value: 0.0,
            position: 0.0,
            label: String::new(),
            style: ReferenceLineStyle::default(),
            visible: true,
            interactive: false,
            start: None,
            end: None,
            band_width: None,
            band_fill: None,
        }
    }
}

impl ReferenceLine {
    /// Create a horizontal reference line
    pub fn horizontal(value: f64, label: impl Into<String>) -> Self {
        Self {
            orientation: ReferenceLineOrientation::Horizontal,
            value,
            position: value,
            label: label.into(),
            ..Default::default()
        }
    }

    /// Create a vertical reference line
    pub fn vertical(value: f64, label: impl Into<String>) -> Self {
        Self {
            orientation: ReferenceLineOrientation::Vertical,
            value,
            position: value,
            label: label.into(),
            ..Default::default()
        }
    }

    /// Create a horizontal band (range)
    pub fn horizontal_band(value: f64, width: f64, label: impl Into<String>, fill: Rgba) -> Self {
        Self {
            orientation: ReferenceLineOrientation::Horizontal,
            value,
            position: value,
            label: label.into(),
            band_width: Some(width),
            band_fill: Some(fill),
            ..Default::default()
        }
    }

    /// Create a vertical band (range)
    pub fn vertical_band(value: f64, width: f64, label: impl Into<String>, fill: Rgba) -> Self {
        Self {
            orientation: ReferenceLineOrientation::Vertical,
            value,
            position: value,
            label: label.into(),
            band_width: Some(width),
            band_fill: Some(fill),
            ..Default::default()
        }
    }

    /// Set ID
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    /// Set style
    pub fn with_style(mut self, style: ReferenceLineStyle) -> Self {
        self.style = style;
        self
    }

    /// Set visibility
    pub fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Make interactive
    pub fn with_interactive(mut self, interactive: bool) -> Self {
        self.interactive = interactive;
        self
    }

    /// Set position (computed from scale)
    pub fn with_position(mut self, position: f64) -> Self {
        self.position = position;
        self
    }

    /// Set extent limits
    pub fn with_extent(mut self, start: f64, end: f64) -> Self {
        self.start = Some(start);
        self.end = Some(end);
        self
    }

    /// Check if this is a band
    pub fn is_band(&self) -> bool {
        self.band_width.is_some()
    }

    /// Check if horizontal
    pub fn is_horizontal(&self) -> bool {
        self.orientation == ReferenceLineOrientation::Horizontal
    }

    /// Check if vertical
    pub fn is_vertical(&self) -> bool {
        self.orientation == ReferenceLineOrientation::Vertical
    }

    /// Get line endpoints given chart bounds
    ///
    /// Returns ((x1, y1), (x2, y2)) for the line segment.
    pub fn endpoints(
        &self,
        chart_x: f64,
        chart_y: f64,
        chart_w: f64,
        chart_h: f64,
    ) -> ((f64, f64), (f64, f64)) {
        match self.orientation {
            ReferenceLineOrientation::Horizontal => {
                let y = self.position;
                let x1 = self.start.unwrap_or(chart_x);
                let x2 = self.end.unwrap_or(chart_x + chart_w);
                ((x1, y), (x2, y))
            }
            ReferenceLineOrientation::Vertical => {
                let x = self.position;
                let y1 = self.start.unwrap_or(chart_y);
                let y2 = self.end.unwrap_or(chart_y + chart_h);
                ((x, y1), (x, y2))
            }
        }
    }

    /// Get band bounds given chart bounds
    ///
    /// Returns (x, y, width, height) for the band rectangle.
    pub fn band_bounds(
        &self,
        chart_x: f64,
        chart_y: f64,
        chart_w: f64,
        chart_h: f64,
    ) -> Option<(f64, f64, f64, f64)> {
        let band_width = self.band_width?;

        Some(match self.orientation {
            ReferenceLineOrientation::Horizontal => {
                let y = self.position - band_width / 2.0;
                let x = self.start.unwrap_or(chart_x);
                let w = self.end.unwrap_or(chart_x + chart_w) - x;
                (x, y, w, band_width)
            }
            ReferenceLineOrientation::Vertical => {
                let x = self.position - band_width / 2.0;
                let y = self.start.unwrap_or(chart_y);
                let h = self.end.unwrap_or(chart_y + chart_h) - y;
                (x, y, band_width, h)
            }
        })
    }

    /// Get label position given chart bounds
    ///
    /// Returns (x, y) for the label position.
    pub fn label_position(
        &self,
        chart_x: f64,
        chart_y: f64,
        chart_w: f64,
        chart_h: f64,
    ) -> (f64, f64) {
        let ((x1, y1), (x2, y2)) = self.endpoints(chart_x, chart_y, chart_w, chart_h);
        let offset = self.style.label_offset;

        match (self.orientation, self.style.label_anchor) {
            (ReferenceLineOrientation::Horizontal, LabelAnchor::Start) => (x1 + offset, y1),
            (ReferenceLineOrientation::Horizontal, LabelAnchor::Center) => ((x1 + x2) / 2.0, y1),
            (ReferenceLineOrientation::Horizontal, LabelAnchor::End) => (x2 - offset, y1),
            (ReferenceLineOrientation::Horizontal, LabelAnchor::Before) => {
                ((x1 + x2) / 2.0, y1 - offset)
            }
            (ReferenceLineOrientation::Horizontal, LabelAnchor::After) => {
                ((x1 + x2) / 2.0, y1 + offset)
            }

            (ReferenceLineOrientation::Vertical, LabelAnchor::Start) => (x1, y1 + offset),
            (ReferenceLineOrientation::Vertical, LabelAnchor::Center) => (x1, (y1 + y2) / 2.0),
            (ReferenceLineOrientation::Vertical, LabelAnchor::End) => (x1, y2 - offset),
            (ReferenceLineOrientation::Vertical, LabelAnchor::Before) => {
                (x1 - offset, (y1 + y2) / 2.0)
            }
            (ReferenceLineOrientation::Vertical, LabelAnchor::After) => {
                (x1 + offset, (y1 + y2) / 2.0)
            }
        }
    }
}

/// Collection of reference lines
#[derive(Clone, Debug, Default)]
pub struct ReferenceLineSet {
    /// All reference lines
    pub lines: Vec<ReferenceLine>,
}

impl ReferenceLineSet {
    /// Create a new empty set
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a reference line
    pub fn add(&mut self, line: ReferenceLine) {
        self.lines.push(line);
    }

    /// Add a horizontal line
    pub fn add_horizontal(&mut self, value: f64, label: impl Into<String>) {
        self.lines.push(ReferenceLine::horizontal(value, label));
    }

    /// Add a vertical line
    pub fn add_vertical(&mut self, value: f64, label: impl Into<String>) {
        self.lines.push(ReferenceLine::vertical(value, label));
    }

    /// Remove by ID
    pub fn remove(&mut self, id: &str) -> Option<ReferenceLine> {
        if let Some(pos) = self.lines.iter().position(|l| l.id == id) {
            Some(self.lines.remove(pos))
        } else {
            None
        }
    }

    /// Find by ID
    pub fn find(&self, id: &str) -> Option<&ReferenceLine> {
        self.lines.iter().find(|l| l.id == id)
    }

    /// Find mutable by ID
    pub fn find_mut(&mut self, id: &str) -> Option<&mut ReferenceLine> {
        self.lines.iter_mut().find(|l| l.id == id)
    }

    /// Get visible lines
    pub fn visible(&self) -> Vec<&ReferenceLine> {
        self.lines.iter().filter(|l| l.visible).collect()
    }

    /// Get horizontal lines
    pub fn horizontal(&self) -> Vec<&ReferenceLine> {
        self.lines.iter().filter(|l| l.is_horizontal()).collect()
    }

    /// Get vertical lines
    pub fn vertical(&self) -> Vec<&ReferenceLine> {
        self.lines.iter().filter(|l| l.is_vertical()).collect()
    }

    /// Clear all lines
    pub fn clear(&mut self) {
        self.lines.clear();
    }

    /// Get count
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Update positions for all lines using a scale function
    pub fn update_positions<F>(&mut self, scale_fn: F)
    where
        F: Fn(f64, ReferenceLineOrientation) -> f64,
    {
        for line in &mut self.lines {
            line.position = scale_fn(line.value, line.orientation);
        }
    }
}

/// Builder for creating reference line sets
pub struct ReferenceLineSetBuilder {
    set: ReferenceLineSet,
}

impl ReferenceLineSetBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            set: ReferenceLineSet::new(),
        }
    }

    /// Add a horizontal threshold line
    pub fn threshold(mut self, value: f64, label: impl Into<String>) -> Self {
        self.set.lines.push(
            ReferenceLine::horizontal(value, label).with_style(ReferenceLineStyle::threshold()),
        );
        self
    }

    /// Add a horizontal target line
    pub fn target(mut self, value: f64, label: impl Into<String>) -> Self {
        self.set
            .lines
            .push(ReferenceLine::horizontal(value, label).with_style(ReferenceLineStyle::target()));
        self
    }

    /// Add an average/mean line
    pub fn average(mut self, value: f64, label: impl Into<String>) -> Self {
        self.set.lines.push(
            ReferenceLine::horizontal(value, label).with_style(ReferenceLineStyle::average()),
        );
        self
    }

    /// Add a baseline line
    pub fn baseline(mut self, value: f64) -> Self {
        self.set.lines.push(
            ReferenceLine::horizontal(value, "")
                .with_style(ReferenceLineStyle::baseline().show_label(false)),
        );
        self
    }

    /// Add a vertical marker
    pub fn vertical_marker(mut self, value: f64, label: impl Into<String>) -> Self {
        self.set.lines.push(ReferenceLine::vertical(value, label));
        self
    }

    /// Add a custom line
    pub fn line(mut self, line: ReferenceLine) -> Self {
        self.set.lines.push(line);
        self
    }

    /// Build the set
    pub fn build(self) -> ReferenceLineSet {
        self.set
    }
}

impl Default for ReferenceLineSetBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reference_line_horizontal() {
        let line = ReferenceLine::horizontal(100.0, "Target");
        assert!(line.is_horizontal());
        assert!(!line.is_vertical());
        assert_eq!(line.value, 100.0);
        assert_eq!(line.label, "Target");
    }

    #[test]
    fn test_reference_line_vertical() {
        let line = ReferenceLine::vertical(50.0, "Marker");
        assert!(line.is_vertical());
        assert!(!line.is_horizontal());
    }

    #[test]
    fn test_reference_line_band() {
        let band =
            ReferenceLine::horizontal_band(75.0, 10.0, "Safe Zone", Rgba::new(0.0, 1.0, 0.0, 0.2));

        assert!(band.is_band());
        assert_eq!(band.band_width, Some(10.0));
    }

    #[test]
    fn test_reference_line_endpoints() {
        let line = ReferenceLine::horizontal(50.0, "").with_position(50.0);
        let ((x1, y1), (x2, y2)) = line.endpoints(0.0, 0.0, 100.0, 100.0);

        assert_eq!(y1, 50.0);
        assert_eq!(y2, 50.0);
        assert_eq!(x1, 0.0);
        assert_eq!(x2, 100.0);
    }

    #[test]
    fn test_reference_line_band_bounds() {
        let band = ReferenceLine::horizontal_band(50.0, 20.0, "", Rgba::RED).with_position(50.0);

        let bounds = band.band_bounds(0.0, 0.0, 100.0, 100.0);
        assert!(bounds.is_some());

        let (x, y, w, h) = bounds.unwrap();
        assert_eq!(x, 0.0);
        assert_eq!(y, 40.0); // 50 - 20/2
        assert_eq!(w, 100.0);
        assert_eq!(h, 20.0);
    }

    #[test]
    fn test_reference_line_style_presets() {
        let threshold = ReferenceLineStyle::threshold();
        assert_eq!(threshold.dash, LineDash::Dashed);

        let target = ReferenceLineStyle::target();
        assert_eq!(target.dash, LineDash::Solid);

        let average = ReferenceLineStyle::average();
        assert_eq!(average.width, 1.5);
    }

    #[test]
    fn test_reference_line_style_builder() {
        let style = ReferenceLineStyle::default()
            .color(Rgba::RED)
            .width(3.0)
            .dash(LineDash::LongDash)
            .opacity(0.8);

        assert_eq!(style.width, 3.0);
        assert_eq!(style.dash, LineDash::LongDash);
    }

    #[test]
    fn test_line_dash_array() {
        assert!(LineDash::Solid.dash_array().is_empty());
        assert!(!LineDash::Dashed.dash_array().is_empty());
        assert_eq!(LineDash::Dotted.dash_array(), vec![2.0, 2.0]);
    }

    #[test]
    fn test_reference_line_set() {
        let mut set = ReferenceLineSet::new();
        set.add(ReferenceLine::horizontal(50.0, "A").with_id("a"));
        set.add(ReferenceLine::vertical(100.0, "B").with_id("b"));

        assert_eq!(set.len(), 2);
        assert_eq!(set.horizontal().len(), 1);
        assert_eq!(set.vertical().len(), 1);
    }

    #[test]
    fn test_reference_line_set_find() {
        let mut set = ReferenceLineSet::new();
        set.add(ReferenceLine::horizontal(50.0, "Test").with_id("test"));

        assert!(set.find("test").is_some());
        assert!(set.find("nonexistent").is_none());
    }

    #[test]
    fn test_reference_line_set_remove() {
        let mut set = ReferenceLineSet::new();
        set.add(ReferenceLine::horizontal(50.0, "A").with_id("a"));
        set.add(ReferenceLine::horizontal(75.0, "B").with_id("b"));

        let removed = set.remove("a");
        assert!(removed.is_some());
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_reference_line_set_builder() {
        let set = ReferenceLineSetBuilder::new()
            .threshold(90.0, "Critical")
            .target(75.0, "Goal")
            .average(60.0, "Mean")
            .baseline(0.0)
            .build();

        assert_eq!(set.len(), 4);
    }

    #[test]
    fn test_reference_line_label_position() {
        let line = ReferenceLine::horizontal(50.0, "Label").with_position(50.0);

        let (lx, ly) = line.label_position(0.0, 0.0, 100.0, 100.0);

        // End anchor by default, so should be near x=100
        assert!(lx > 50.0);
        assert_eq!(ly, 50.0);
    }

    #[test]
    fn test_reference_line_with_extent() {
        let line = ReferenceLine::horizontal(50.0, "")
            .with_position(50.0)
            .with_extent(20.0, 80.0);

        let ((x1, _), (x2, _)) = line.endpoints(0.0, 0.0, 100.0, 100.0);
        assert_eq!(x1, 20.0);
        assert_eq!(x2, 80.0);
    }
}

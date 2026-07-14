//! Crosshair component for cursor tracking in charts
//!
//! Provides vertical and horizontal guide lines that follow the cursor,
//! useful for reading values from charts.
//!
//! # Example
//!
//! ```
//! use makepad_d3::component::{Crosshair, CrosshairMode, CrosshairStyle};
//! use makepad_d3::color::Rgba;
//!
//! let mut crosshair = Crosshair::new()
//!     .mode(CrosshairMode::Both)
//!     .snap_to_data(true)
//!     .show_labels(true);
//!
//! // Update with cursor position
//! crosshair.update(150.0, 200.0);
//!
//! // Get line positions for rendering
//! let (v_line, h_line) = crosshair.get_lines();
//! ```

use crate::color::Rgba;
use serde::{Deserialize, Serialize};

/// Crosshair display mode
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrosshairMode {
    /// Show only vertical line
    Vertical,
    /// Show only horizontal line
    Horizontal,
    /// Show both lines
    #[default]
    Both,
    /// Hide crosshair
    None,
}

/// Line style for crosshair
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrosshairLineStyle {
    /// Solid line
    #[default]
    Solid,
    /// Dashed line
    Dashed,
    /// Dotted line
    Dotted,
    /// Dash-dot pattern
    DashDot,
}

/// Label position relative to the crosshair line
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum LabelPosition {
    /// At the start of the line (top for vertical, left for horizontal)
    Start,
    /// At the end of the line (bottom for vertical, right for horizontal)
    #[default]
    End,
    /// At the cursor position
    AtCursor,
    /// On the axis
    OnAxis,
}

/// Styling for crosshair lines
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CrosshairStyle {
    /// Line color
    #[serde(skip)]
    pub color: Rgba,
    /// Line width
    pub width: f64,
    /// Line style
    pub line_style: CrosshairLineStyle,
    /// Dash array for dashed/dotted styles
    pub dash_array: Vec<f64>,
    /// Line opacity (0.0 to 1.0)
    pub opacity: f32,
}

impl Default for CrosshairStyle {
    fn default() -> Self {
        Self {
            color: Rgba::new(0.5, 0.5, 0.5, 1.0),
            width: 1.0,
            line_style: CrosshairLineStyle::Dashed,
            dash_array: vec![4.0, 4.0],
            opacity: 0.8,
        }
    }
}

impl CrosshairStyle {
    /// Create a solid line style
    pub fn solid(color: Rgba, width: f64) -> Self {
        Self {
            color,
            width,
            line_style: CrosshairLineStyle::Solid,
            dash_array: Vec::new(),
            opacity: 1.0,
        }
    }

    /// Create a dashed line style
    pub fn dashed(color: Rgba, width: f64) -> Self {
        Self {
            color,
            width,
            line_style: CrosshairLineStyle::Dashed,
            dash_array: vec![6.0, 4.0],
            opacity: 0.8,
        }
    }

    /// Create a dotted line style
    pub fn dotted(color: Rgba, width: f64) -> Self {
        Self {
            color,
            width,
            line_style: CrosshairLineStyle::Dotted,
            dash_array: vec![2.0, 2.0],
            opacity: 0.8,
        }
    }

    /// Set color
    pub fn with_color(mut self, color: Rgba) -> Self {
        self.color = color;
        self
    }

    /// Set width
    pub fn with_width(mut self, width: f64) -> Self {
        self.width = width;
        self
    }

    /// Set opacity
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;
        self
    }

    /// Set custom dash array
    pub fn with_dash(mut self, dash: Vec<f64>) -> Self {
        self.dash_array = dash;
        self
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

/// Label configuration for crosshair
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CrosshairLabelConfig {
    /// Whether to show labels
    pub visible: bool,
    /// Label position for vertical line
    pub vertical_position: LabelPosition,
    /// Label position for horizontal line
    pub horizontal_position: LabelPosition,
    /// Background color
    #[serde(skip)]
    pub background: Rgba,
    /// Text color
    #[serde(skip)]
    pub text_color: Rgba,
    /// Font size
    pub font_size: f64,
    /// Padding inside label
    pub padding: f64,
    /// Border radius
    pub border_radius: f64,
    /// Format function result for X axis (stored as formatted value)
    pub x_format: Option<String>,
    /// Format function result for Y axis
    pub y_format: Option<String>,
}

impl Default for CrosshairLabelConfig {
    fn default() -> Self {
        Self {
            visible: true,
            vertical_position: LabelPosition::OnAxis,
            horizontal_position: LabelPosition::OnAxis,
            background: Rgba::new(0.2, 0.2, 0.2, 0.9),
            text_color: Rgba::WHITE,
            font_size: 11.0,
            padding: 4.0,
            border_radius: 2.0,
            x_format: None,
            y_format: None,
        }
    }
}

/// A line in the crosshair
#[derive(Clone, Debug, Default)]
pub struct CrosshairLine {
    /// Starting X coordinate
    pub x1: f64,
    /// Starting Y coordinate
    pub y1: f64,
    /// Ending X coordinate
    pub x2: f64,
    /// Ending Y coordinate
    pub y2: f64,
    /// Whether this line is visible
    pub visible: bool,
    /// Label text
    pub label: Option<String>,
    /// Label position (x, y)
    pub label_position: Option<(f64, f64)>,
}

impl CrosshairLine {
    /// Create a vertical line
    pub fn vertical(x: f64, y1: f64, y2: f64) -> Self {
        Self {
            x1: x,
            y1,
            x2: x,
            y2,
            visible: true,
            label: None,
            label_position: None,
        }
    }

    /// Create a horizontal line
    pub fn horizontal(y: f64, x1: f64, x2: f64) -> Self {
        Self {
            x1,
            y1: y,
            x2,
            y2: y,
            visible: true,
            label: None,
            label_position: None,
        }
    }

    /// Set label
    pub fn with_label(mut self, label: impl Into<String>, x: f64, y: f64) -> Self {
        self.label = Some(label.into());
        self.label_position = Some((x, y));
        self
    }

    /// Check if line is horizontal
    pub fn is_horizontal(&self) -> bool {
        (self.y1 - self.y2).abs() < f64::EPSILON
    }

    /// Check if line is vertical
    pub fn is_vertical(&self) -> bool {
        (self.x1 - self.x2).abs() < f64::EPSILON
    }

    /// Get length of the line
    pub fn length(&self) -> f64 {
        let dx = self.x2 - self.x1;
        let dy = self.y2 - self.y1;
        (dx * dx + dy * dy).sqrt()
    }
}

/// Data point for snapping
#[derive(Clone, Debug)]
pub struct SnapPoint {
    /// X coordinate in screen space
    pub x: f64,
    /// Y coordinate in screen space
    pub y: f64,
    /// X value in data space
    pub x_value: f64,
    /// Y value in data space
    pub y_value: f64,
    /// Series index
    pub series_index: usize,
    /// Point index within series
    pub point_index: usize,
}

/// Crosshair component for cursor tracking
#[derive(Clone, Debug)]
pub struct Crosshair {
    /// Display mode
    pub mode: CrosshairMode,
    /// Style for vertical line
    pub vertical_style: CrosshairStyle,
    /// Style for horizontal line
    pub horizontal_style: CrosshairStyle,
    /// Label configuration
    pub labels: CrosshairLabelConfig,
    /// Chart bounds (x, y, width, height)
    pub bounds: (f64, f64, f64, f64),
    /// Current cursor position
    pub cursor_x: f64,
    /// Current cursor position
    pub cursor_y: f64,
    /// Whether crosshair is active
    pub active: bool,
    /// Snap to nearest data point
    pub snap_to_data: bool,
    /// Snap threshold in pixels
    pub snap_threshold: f64,
    /// Current snapped point (if any)
    pub snapped_point: Option<SnapPoint>,
    /// Available snap points
    snap_points: Vec<SnapPoint>,
}

impl Default for Crosshair {
    fn default() -> Self {
        Self {
            mode: CrosshairMode::Both,
            vertical_style: CrosshairStyle::default(),
            horizontal_style: CrosshairStyle::default(),
            labels: CrosshairLabelConfig::default(),
            bounds: (0.0, 0.0, 100.0, 100.0),
            cursor_x: 0.0,
            cursor_y: 0.0,
            active: false,
            snap_to_data: false,
            snap_threshold: 20.0,
            snapped_point: None,
            snap_points: Vec::new(),
        }
    }
}

impl Crosshair {
    /// Create a new crosshair
    pub fn new() -> Self {
        Self::default()
    }

    /// Set display mode
    pub fn mode(mut self, mode: CrosshairMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set style for vertical line
    pub fn vertical_style(mut self, style: CrosshairStyle) -> Self {
        self.vertical_style = style;
        self
    }

    /// Set style for horizontal line
    pub fn horizontal_style(mut self, style: CrosshairStyle) -> Self {
        self.horizontal_style = style;
        self
    }

    /// Set same style for both lines
    pub fn style(mut self, style: CrosshairStyle) -> Self {
        self.vertical_style = style.clone();
        self.horizontal_style = style;
        self
    }

    /// Set label configuration
    pub fn labels(mut self, labels: CrosshairLabelConfig) -> Self {
        self.labels = labels;
        self
    }

    /// Show or hide labels
    pub fn show_labels(mut self, show: bool) -> Self {
        self.labels.visible = show;
        self
    }

    /// Enable/disable snap to data
    pub fn snap_to_data(mut self, snap: bool) -> Self {
        self.snap_to_data = snap;
        self
    }

    /// Set snap threshold
    pub fn snap_threshold(mut self, threshold: f64) -> Self {
        self.snap_threshold = threshold;
        self
    }

    /// Set chart bounds
    pub fn bounds(mut self, x: f64, y: f64, width: f64, height: f64) -> Self {
        self.bounds = (x, y, width, height);
        self
    }

    /// Set chart bounds from tuple
    pub fn set_bounds(&mut self, bounds: (f64, f64, f64, f64)) {
        self.bounds = bounds;
    }

    /// Set snap points for data snapping
    pub fn set_snap_points(&mut self, points: Vec<SnapPoint>) {
        self.snap_points = points;
    }

    /// Add a snap point
    pub fn add_snap_point(&mut self, point: SnapPoint) {
        self.snap_points.push(point);
    }

    /// Clear snap points
    pub fn clear_snap_points(&mut self) {
        self.snap_points.clear();
    }

    /// Update cursor position
    pub fn update(&mut self, x: f64, y: f64) {
        self.cursor_x = x;
        self.cursor_y = y;

        // Check if within bounds
        let (bx, by, bw, bh) = self.bounds;
        self.active = x >= bx && x <= bx + bw && y >= by && y <= by + bh;

        // Handle snapping
        if self.snap_to_data && self.active {
            self.snapped_point = self.find_nearest_point();
        } else {
            self.snapped_point = None;
        }
    }

    /// Deactivate crosshair
    pub fn deactivate(&mut self) {
        self.active = false;
        self.snapped_point = None;
    }

    /// Check if crosshair is active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Get the effective cursor position (accounting for snap)
    pub fn effective_position(&self) -> (f64, f64) {
        if let Some(ref point) = self.snapped_point {
            (point.x, point.y)
        } else {
            (self.cursor_x, self.cursor_y)
        }
    }

    /// Get vertical and horizontal lines for rendering
    pub fn get_lines(&self) -> (Option<CrosshairLine>, Option<CrosshairLine>) {
        if !self.active {
            return (None, None);
        }

        let (bx, by, bw, bh) = self.bounds;
        let (x, y) = self.effective_position();

        let vertical = match self.mode {
            CrosshairMode::Vertical | CrosshairMode::Both => {
                let mut line = CrosshairLine::vertical(x, by, by + bh);
                if self.labels.visible {
                    let label = self
                        .labels
                        .x_format
                        .clone()
                        .unwrap_or_else(|| format!("{:.1}", x - bx));
                    let label_y = match self.labels.vertical_position {
                        LabelPosition::Start => by,
                        LabelPosition::End => by + bh,
                        LabelPosition::AtCursor => y,
                        LabelPosition::OnAxis => by + bh,
                    };
                    line = line.with_label(label, x, label_y);
                }
                Some(line)
            }
            _ => None,
        };

        let horizontal = match self.mode {
            CrosshairMode::Horizontal | CrosshairMode::Both => {
                let mut line = CrosshairLine::horizontal(y, bx, bx + bw);
                if self.labels.visible {
                    let label = self
                        .labels
                        .y_format
                        .clone()
                        .unwrap_or_else(|| format!("{:.1}", y - by));
                    let label_x = match self.labels.horizontal_position {
                        LabelPosition::Start => bx,
                        LabelPosition::End => bx + bw,
                        LabelPosition::AtCursor => x,
                        LabelPosition::OnAxis => bx,
                    };
                    line = line.with_label(label, label_x, y);
                }
                Some(line)
            }
            _ => None,
        };

        (vertical, horizontal)
    }

    /// Find nearest snap point to cursor
    fn find_nearest_point(&self) -> Option<SnapPoint> {
        if self.snap_points.is_empty() {
            return None;
        }

        let mut nearest: Option<(f64, &SnapPoint)> = None;

        for point in &self.snap_points {
            let dx = point.x - self.cursor_x;
            let dy = point.y - self.cursor_y;
            let dist_sq = dx * dx + dy * dy;

            match nearest {
                Some((min_dist, _)) if dist_sq < min_dist => {
                    nearest = Some((dist_sq, point));
                }
                None => {
                    nearest = Some((dist_sq, point));
                }
                _ => {}
            }
        }

        nearest.and_then(|(dist_sq, point)| {
            if dist_sq.sqrt() <= self.snap_threshold {
                Some(point.clone())
            } else {
                None
            }
        })
    }

    /// Set X axis label format
    pub fn set_x_format(&mut self, format: impl Into<String>) {
        self.labels.x_format = Some(format.into());
    }

    /// Set Y axis label format
    pub fn set_y_format(&mut self, format: impl Into<String>) {
        self.labels.y_format = Some(format.into());
    }

    /// Clear label formats
    pub fn clear_formats(&mut self) {
        self.labels.x_format = None;
        self.labels.y_format = None;
    }
}

/// Builder for creating crosshairs
pub struct CrosshairBuilder {
    crosshair: Crosshair,
}

impl CrosshairBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            crosshair: Crosshair::new(),
        }
    }

    /// Set mode
    pub fn mode(mut self, mode: CrosshairMode) -> Self {
        self.crosshair.mode = mode;
        self
    }

    /// Vertical only mode
    pub fn vertical_only(self) -> Self {
        self.mode(CrosshairMode::Vertical)
    }

    /// Horizontal only mode
    pub fn horizontal_only(self) -> Self {
        self.mode(CrosshairMode::Horizontal)
    }

    /// Both lines
    pub fn both(self) -> Self {
        self.mode(CrosshairMode::Both)
    }

    /// Set color for both lines
    pub fn color(mut self, color: Rgba) -> Self {
        self.crosshair.vertical_style.color = color;
        self.crosshair.horizontal_style.color = color;
        self
    }

    /// Set line width
    pub fn width(mut self, width: f64) -> Self {
        self.crosshair.vertical_style.width = width;
        self.crosshair.horizontal_style.width = width;
        self
    }

    /// Make lines solid
    pub fn solid(mut self) -> Self {
        self.crosshair.vertical_style.line_style = CrosshairLineStyle::Solid;
        self.crosshair.horizontal_style.line_style = CrosshairLineStyle::Solid;
        self
    }

    /// Make lines dashed
    pub fn dashed(mut self) -> Self {
        self.crosshair.vertical_style.line_style = CrosshairLineStyle::Dashed;
        self.crosshair.horizontal_style.line_style = CrosshairLineStyle::Dashed;
        self.crosshair.vertical_style.dash_array = vec![6.0, 4.0];
        self.crosshair.horizontal_style.dash_array = vec![6.0, 4.0];
        self
    }

    /// Enable labels
    pub fn with_labels(mut self) -> Self {
        self.crosshair.labels.visible = true;
        self
    }

    /// Disable labels
    pub fn without_labels(mut self) -> Self {
        self.crosshair.labels.visible = false;
        self
    }

    /// Enable snap to data
    pub fn snap_to_data(mut self, threshold: f64) -> Self {
        self.crosshair.snap_to_data = true;
        self.crosshair.snap_threshold = threshold;
        self
    }

    /// Set bounds
    pub fn bounds(mut self, x: f64, y: f64, width: f64, height: f64) -> Self {
        self.crosshair.bounds = (x, y, width, height);
        self
    }

    /// Build the crosshair
    pub fn build(self) -> Crosshair {
        self.crosshair
    }
}

impl Default for CrosshairBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crosshair_new() {
        let crosshair = Crosshair::new();
        assert_eq!(crosshair.mode, CrosshairMode::Both);
        assert!(!crosshair.active);
    }

    #[test]
    fn test_crosshair_update() {
        let mut crosshair = Crosshair::new().bounds(0.0, 0.0, 100.0, 100.0);

        crosshair.update(50.0, 50.0);
        assert!(crosshair.is_active());

        crosshair.update(150.0, 50.0);
        assert!(!crosshair.is_active());
    }

    #[test]
    fn test_crosshair_get_lines() {
        let mut crosshair = Crosshair::new()
            .bounds(0.0, 0.0, 100.0, 100.0)
            .mode(CrosshairMode::Both);

        crosshair.update(50.0, 50.0);
        let (v, h) = crosshair.get_lines();

        assert!(v.is_some());
        assert!(h.is_some());

        let v_line = v.unwrap();
        assert!(v_line.is_vertical());
        assert!((v_line.x1 - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_crosshair_mode_vertical() {
        let mut crosshair = Crosshair::new()
            .bounds(0.0, 0.0, 100.0, 100.0)
            .mode(CrosshairMode::Vertical);

        crosshair.update(50.0, 50.0);
        let (v, h) = crosshair.get_lines();

        assert!(v.is_some());
        assert!(h.is_none());
    }

    #[test]
    fn test_crosshair_mode_horizontal() {
        let mut crosshair = Crosshair::new()
            .bounds(0.0, 0.0, 100.0, 100.0)
            .mode(CrosshairMode::Horizontal);

        crosshair.update(50.0, 50.0);
        let (v, h) = crosshair.get_lines();

        assert!(v.is_none());
        assert!(h.is_some());
    }

    #[test]
    fn test_crosshair_snap() {
        let mut crosshair = Crosshair::new()
            .bounds(0.0, 0.0, 100.0, 100.0)
            .snap_to_data(true)
            .snap_threshold(10.0);

        crosshair.set_snap_points(vec![
            SnapPoint {
                x: 50.0,
                y: 50.0,
                x_value: 5.0,
                y_value: 5.0,
                series_index: 0,
                point_index: 0,
            },
            SnapPoint {
                x: 80.0,
                y: 80.0,
                x_value: 8.0,
                y_value: 8.0,
                series_index: 0,
                point_index: 1,
            },
        ]);

        // Near first point
        crosshair.update(52.0, 48.0);
        assert!(crosshair.snapped_point.is_some());
        let (x, y) = crosshair.effective_position();
        assert!((x - 50.0).abs() < 0.001);
        assert!((y - 50.0).abs() < 0.001);

        // Far from any point
        crosshair.update(30.0, 30.0);
        assert!(crosshair.snapped_point.is_none());
    }

    #[test]
    fn test_crosshair_deactivate() {
        let mut crosshair = Crosshair::new().bounds(0.0, 0.0, 100.0, 100.0);

        crosshair.update(50.0, 50.0);
        assert!(crosshair.is_active());

        crosshair.deactivate();
        assert!(!crosshair.is_active());
    }

    #[test]
    fn test_crosshair_line_methods() {
        let v_line = CrosshairLine::vertical(50.0, 0.0, 100.0);
        assert!(v_line.is_vertical());
        assert!(!v_line.is_horizontal());
        assert!((v_line.length() - 100.0).abs() < 0.001);

        let h_line = CrosshairLine::horizontal(50.0, 0.0, 100.0);
        assert!(h_line.is_horizontal());
        assert!(!h_line.is_vertical());
    }

    #[test]
    fn test_crosshair_style() {
        let style = CrosshairStyle::dashed(Rgba::from_hex(0x3366FF), 2.0);
        assert_eq!(style.width, 2.0);
        assert_eq!(style.line_style, CrosshairLineStyle::Dashed);
    }

    #[test]
    fn test_crosshair_builder() {
        let crosshair = CrosshairBuilder::new()
            .vertical_only()
            .color(Rgba::RED)
            .width(2.0)
            .solid()
            .with_labels()
            .bounds(10.0, 10.0, 200.0, 200.0)
            .build();

        assert_eq!(crosshair.mode, CrosshairMode::Vertical);
        assert_eq!(crosshair.vertical_style.width, 2.0);
        assert!(crosshair.labels.visible);
    }

    #[test]
    fn test_crosshair_labels() {
        let mut crosshair = Crosshair::new()
            .bounds(0.0, 0.0, 100.0, 100.0)
            .show_labels(true);

        crosshair.set_x_format("X: 50");
        crosshair.set_y_format("Y: 50");

        crosshair.update(50.0, 50.0);
        let (v, h) = crosshair.get_lines();

        assert!(v.unwrap().label.is_some());
        assert!(h.unwrap().label.is_some());
    }
}

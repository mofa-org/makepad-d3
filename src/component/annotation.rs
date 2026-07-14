//! Annotation component for chart labels and callouts
//!
//! Provides configurable annotations for marking specific points,
//! regions, or adding informational labels to charts.
//!
//! # Example
//!
//! ```
//! use makepad_d3::component::{Annotation, AnnotationType, AnnotationStyle};
//! use makepad_d3::color::Rgba;
//!
//! // Create a text annotation
//! let label = Annotation::text(100.0, 50.0, "Peak Value")
//!     .with_background(Rgba::from_hex(0xFFFFCC));
//!
//! // Create a callout with connector
//! let callout = Annotation::callout(150.0, 75.0, 200.0, 50.0, "Important Point")
//!     .with_arrow(true);
//! ```

use crate::color::Rgba;
use serde::{Deserialize, Serialize};

/// Type of annotation
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnnotationType {
    /// Simple text label
    #[default]
    Text,
    /// Text with connector line to a point
    Callout,
    /// Circular marker
    Circle,
    /// Rectangular region
    Rectangle,
    /// Line segment
    Line,
    /// Arrow
    Arrow,
    /// Badge/pill-shaped label
    Badge,
}

/// Text alignment for annotations
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextAlign {
    /// Align to the left
    Left,
    /// Align to the center
    #[default]
    Center,
    /// Align to the right
    Right,
}

/// Vertical alignment for annotations
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerticalAlign {
    /// Align to the top
    Top,
    /// Align to the middle
    #[default]
    Middle,
    /// Align to the bottom
    Bottom,
}

/// Connector line style for callouts
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectorStyle {
    /// Straight line
    #[default]
    Straight,
    /// Elbow connector (horizontal then vertical)
    ElbowHV,
    /// Elbow connector (vertical then horizontal)
    ElbowVH,
    /// Curved line
    Curved,
}

/// Arrow head style
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArrowStyle {
    /// No arrow head
    None,
    /// Simple triangle
    #[default]
    Triangle,
    /// Open arrow (chevron)
    Open,
    /// Stealth arrow
    Stealth,
    /// Circle marker
    Circle,
    /// Diamond marker
    Diamond,
}

/// Styling for annotations
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnnotationStyle {
    /// Fill color for shapes
    #[serde(skip)]
    pub fill: Rgba,
    /// Stroke color
    #[serde(skip)]
    pub stroke: Rgba,
    /// Stroke width
    pub stroke_width: f64,
    /// Text color
    #[serde(skip)]
    pub text_color: Rgba,
    /// Font size
    pub font_size: f64,
    /// Font weight (100-900)
    pub font_weight: u16,
    /// Padding inside the annotation
    pub padding: f64,
    /// Border radius for rectangles/badges
    pub border_radius: f64,
    /// Opacity
    pub opacity: f32,
    /// Shadow offset X
    pub shadow_x: f64,
    /// Shadow offset Y
    pub shadow_y: f64,
    /// Shadow blur
    pub shadow_blur: f64,
    /// Shadow color
    #[serde(skip)]
    pub shadow_color: Rgba,
}

impl Default for AnnotationStyle {
    fn default() -> Self {
        Self {
            fill: Rgba::new(1.0, 1.0, 1.0, 0.95),
            stroke: Rgba::new(0.7, 0.7, 0.7, 1.0),
            stroke_width: 1.0,
            text_color: Rgba::new(0.2, 0.2, 0.2, 1.0),
            font_size: 12.0,
            font_weight: 400,
            padding: 6.0,
            border_radius: 4.0,
            opacity: 1.0,
            shadow_x: 0.0,
            shadow_y: 1.0,
            shadow_blur: 3.0,
            shadow_color: Rgba::new(0.0, 0.0, 0.0, 0.1),
        }
    }
}

impl AnnotationStyle {
    /// Create a minimal style with no background
    pub fn minimal() -> Self {
        Self {
            fill: Rgba::TRANSPARENT,
            stroke: Rgba::TRANSPARENT,
            stroke_width: 0.0,
            shadow_blur: 0.0,
            ..Default::default()
        }
    }

    /// Create a highlighted/emphasis style
    pub fn highlight() -> Self {
        Self {
            fill: Rgba::new(1.0, 0.95, 0.7, 1.0),
            stroke: Rgba::new(1.0, 0.8, 0.2, 1.0),
            stroke_width: 2.0,
            font_weight: 600,
            ..Default::default()
        }
    }

    /// Create a dark themed style
    pub fn dark() -> Self {
        Self {
            fill: Rgba::new(0.2, 0.2, 0.2, 0.95),
            stroke: Rgba::new(0.4, 0.4, 0.4, 1.0),
            text_color: Rgba::WHITE,
            ..Default::default()
        }
    }

    /// Create a colored badge style
    pub fn badge(color: Rgba) -> Self {
        Self {
            fill: color,
            stroke: Rgba::TRANSPARENT,
            stroke_width: 0.0,
            text_color: Rgba::WHITE,
            border_radius: 12.0,
            padding: 8.0,
            font_weight: 500,
            ..Default::default()
        }
    }

    /// Set fill color
    pub fn fill(mut self, color: Rgba) -> Self {
        self.fill = color;
        self
    }

    /// Set stroke
    pub fn stroke(mut self, color: Rgba, width: f64) -> Self {
        self.stroke = color;
        self.stroke_width = width;
        self
    }

    /// Set text color
    pub fn text_color(mut self, color: Rgba) -> Self {
        self.text_color = color;
        self
    }

    /// Set font size
    pub fn font_size(mut self, size: f64) -> Self {
        self.font_size = size;
        self
    }

    /// Set font weight
    pub fn font_weight(mut self, weight: u16) -> Self {
        self.font_weight = weight;
        self
    }

    /// Set padding
    pub fn padding(mut self, padding: f64) -> Self {
        self.padding = padding;
        self
    }

    /// Set border radius
    pub fn border_radius(mut self, radius: f64) -> Self {
        self.border_radius = radius;
        self
    }

    /// Set opacity
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;
        self
    }

    /// Set shadow
    pub fn shadow(mut self, x: f64, y: f64, blur: f64, color: Rgba) -> Self {
        self.shadow_x = x;
        self.shadow_y = y;
        self.shadow_blur = blur;
        self.shadow_color = color;
        self
    }

    /// Disable shadow
    pub fn no_shadow(mut self) -> Self {
        self.shadow_blur = 0.0;
        self
    }
}

/// An annotation element on a chart
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Annotation {
    /// Unique identifier
    pub id: String,
    /// Annotation type
    pub annotation_type: AnnotationType,
    /// X position (primary point)
    pub x: f64,
    /// Y position (primary point)
    pub y: f64,
    /// Secondary X position (for callouts, lines, rectangles)
    pub x2: Option<f64>,
    /// Secondary Y position
    pub y2: Option<f64>,
    /// Text content
    pub text: String,
    /// Text alignment
    pub text_align: TextAlign,
    /// Vertical alignment
    pub vertical_align: VerticalAlign,
    /// Styling
    #[serde(skip)]
    pub style: AnnotationStyle,
    /// Connector style for callouts
    pub connector_style: ConnectorStyle,
    /// Arrow head style
    pub arrow_style: ArrowStyle,
    /// Arrow size
    pub arrow_size: f64,
    /// Width (for rectangles, circles)
    pub width: Option<f64>,
    /// Height (for rectangles)
    pub height: Option<f64>,
    /// Rotation angle in degrees
    pub rotation: f64,
    /// Whether the annotation is visible
    pub visible: bool,
    /// Whether the annotation is interactive
    pub interactive: bool,
}

impl Default for Annotation {
    fn default() -> Self {
        Self {
            id: String::new(),
            annotation_type: AnnotationType::Text,
            x: 0.0,
            y: 0.0,
            x2: None,
            y2: None,
            text: String::new(),
            text_align: TextAlign::Center,
            vertical_align: VerticalAlign::Middle,
            style: AnnotationStyle::default(),
            connector_style: ConnectorStyle::Straight,
            arrow_style: ArrowStyle::None,
            arrow_size: 8.0,
            width: None,
            height: None,
            rotation: 0.0,
            visible: true,
            interactive: false,
        }
    }
}

impl Annotation {
    /// Create a text annotation
    pub fn text(x: f64, y: f64, text: impl Into<String>) -> Self {
        Self {
            annotation_type: AnnotationType::Text,
            x,
            y,
            text: text.into(),
            ..Default::default()
        }
    }

    /// Create a callout annotation (text with connector to point)
    pub fn callout(
        target_x: f64,
        target_y: f64,
        label_x: f64,
        label_y: f64,
        text: impl Into<String>,
    ) -> Self {
        Self {
            annotation_type: AnnotationType::Callout,
            x: target_x,
            y: target_y,
            x2: Some(label_x),
            y2: Some(label_y),
            text: text.into(),
            arrow_style: ArrowStyle::Triangle,
            ..Default::default()
        }
    }

    /// Create a circle marker
    pub fn circle(x: f64, y: f64, radius: f64) -> Self {
        Self {
            annotation_type: AnnotationType::Circle,
            x,
            y,
            width: Some(radius * 2.0),
            ..Default::default()
        }
    }

    /// Create a rectangular region
    pub fn rectangle(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            annotation_type: AnnotationType::Rectangle,
            x,
            y,
            width: Some(width),
            height: Some(height),
            ..Default::default()
        }
    }

    /// Create a line annotation
    pub fn line(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        Self {
            annotation_type: AnnotationType::Line,
            x: x1,
            y: y1,
            x2: Some(x2),
            y2: Some(y2),
            ..Default::default()
        }
    }

    /// Create an arrow annotation
    pub fn arrow(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        Self {
            annotation_type: AnnotationType::Arrow,
            x: x1,
            y: y1,
            x2: Some(x2),
            y2: Some(y2),
            arrow_style: ArrowStyle::Triangle,
            ..Default::default()
        }
    }

    /// Create a badge annotation
    pub fn badge(x: f64, y: f64, text: impl Into<String>, color: Rgba) -> Self {
        Self {
            annotation_type: AnnotationType::Badge,
            x,
            y,
            text: text.into(),
            style: AnnotationStyle::badge(color),
            ..Default::default()
        }
    }

    /// Set ID
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    /// Set style
    pub fn with_style(mut self, style: AnnotationStyle) -> Self {
        self.style = style;
        self
    }

    /// Set background color
    pub fn with_background(mut self, color: Rgba) -> Self {
        self.style.fill = color;
        self
    }

    /// Set text color
    pub fn with_text_color(mut self, color: Rgba) -> Self {
        self.style.text_color = color;
        self
    }

    /// Set text alignment
    pub fn with_text_align(mut self, align: TextAlign) -> Self {
        self.text_align = align;
        self
    }

    /// Set vertical alignment
    pub fn with_vertical_align(mut self, align: VerticalAlign) -> Self {
        self.vertical_align = align;
        self
    }

    /// Enable arrow
    pub fn with_arrow(mut self, enabled: bool) -> Self {
        self.arrow_style = if enabled {
            ArrowStyle::Triangle
        } else {
            ArrowStyle::None
        };
        self
    }

    /// Set arrow style
    pub fn with_arrow_style(mut self, style: ArrowStyle) -> Self {
        self.arrow_style = style;
        self
    }

    /// Set arrow size
    pub fn with_arrow_size(mut self, size: f64) -> Self {
        self.arrow_size = size;
        self
    }

    /// Set connector style
    pub fn with_connector(mut self, style: ConnectorStyle) -> Self {
        self.connector_style = style;
        self
    }

    /// Set rotation
    pub fn with_rotation(mut self, degrees: f64) -> Self {
        self.rotation = degrees;
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

    /// Set font size
    pub fn with_font_size(mut self, size: f64) -> Self {
        self.style.font_size = size;
        self
    }

    /// Set font weight
    pub fn with_font_weight(mut self, weight: u16) -> Self {
        self.style.font_weight = weight;
        self
    }

    /// Get the primary position
    pub fn position(&self) -> (f64, f64) {
        (self.x, self.y)
    }

    /// Get the secondary position (for callouts, lines)
    pub fn secondary_position(&self) -> Option<(f64, f64)> {
        match (self.x2, self.y2) {
            (Some(x), Some(y)) => Some((x, y)),
            _ => None,
        }
    }

    /// Get bounds of the annotation
    ///
    /// Returns (x, y, width, height) based on annotation type.
    pub fn bounds(&self) -> (f64, f64, f64, f64) {
        match self.annotation_type {
            AnnotationType::Rectangle => {
                let w = self.width.unwrap_or(0.0);
                let h = self.height.unwrap_or(0.0);
                (self.x, self.y, w, h)
            }
            AnnotationType::Circle => {
                let d = self.width.unwrap_or(0.0);
                (self.x - d / 2.0, self.y - d / 2.0, d, d)
            }
            AnnotationType::Text | AnnotationType::Badge => {
                // Estimate text bounds
                let w =
                    self.text.len() as f64 * self.style.font_size * 0.6 + self.style.padding * 2.0;
                let h = self.style.font_size + self.style.padding * 2.0;
                (self.x - w / 2.0, self.y - h / 2.0, w, h)
            }
            AnnotationType::Callout => {
                // Return bounds of the label portion
                let label_x = self.x2.unwrap_or(self.x);
                let label_y = self.y2.unwrap_or(self.y);
                let w =
                    self.text.len() as f64 * self.style.font_size * 0.6 + self.style.padding * 2.0;
                let h = self.style.font_size + self.style.padding * 2.0;
                (label_x - w / 2.0, label_y - h / 2.0, w, h)
            }
            AnnotationType::Line | AnnotationType::Arrow => {
                let x2 = self.x2.unwrap_or(self.x);
                let y2 = self.y2.unwrap_or(self.y);
                let min_x = self.x.min(x2);
                let min_y = self.y.min(y2);
                let max_x = self.x.max(x2);
                let max_y = self.y.max(y2);
                (min_x, min_y, max_x - min_x, max_y - min_y)
            }
        }
    }

    /// Check if a point is within the annotation bounds
    pub fn contains(&self, px: f64, py: f64) -> bool {
        let (x, y, w, h) = self.bounds();
        px >= x && px <= x + w && py >= y && py <= y + h
    }

    /// Get connector line points for callout annotations
    ///
    /// Returns Vec of (x, y) points defining the connector path.
    pub fn connector_points(&self) -> Vec<(f64, f64)> {
        if self.annotation_type != AnnotationType::Callout {
            return Vec::new();
        }

        let target = (self.x, self.y);
        let label = (self.x2.unwrap_or(self.x), self.y2.unwrap_or(self.y));

        match self.connector_style {
            ConnectorStyle::Straight => vec![target, label],
            ConnectorStyle::ElbowHV => {
                vec![target, (label.0, target.1), label]
            }
            ConnectorStyle::ElbowVH => {
                vec![target, (target.0, label.1), label]
            }
            ConnectorStyle::Curved => {
                // Simple quadratic bezier-like approximation
                let mid_x = (target.0 + label.0) / 2.0;
                let mid_y = (target.1 + label.1) / 2.0;
                let ctrl = (mid_x, target.1 + (label.1 - target.1) * 0.3);
                vec![target, ctrl, (mid_x, mid_y), label]
            }
        }
    }
}

/// Collection of annotations
#[derive(Clone, Debug, Default)]
pub struct AnnotationLayer {
    /// All annotations in this layer
    pub annotations: Vec<Annotation>,
    /// Layer name
    pub name: String,
    /// Whether the layer is visible
    pub visible: bool,
    /// Layer opacity
    pub opacity: f32,
}

impl AnnotationLayer {
    /// Create a new annotation layer
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            annotations: Vec::new(),
            name: name.into(),
            visible: true,
            opacity: 1.0,
        }
    }

    /// Add an annotation
    pub fn add(&mut self, annotation: Annotation) {
        self.annotations.push(annotation);
    }

    /// Remove annotation by ID
    pub fn remove(&mut self, id: &str) -> Option<Annotation> {
        if let Some(pos) = self.annotations.iter().position(|a| a.id == id) {
            Some(self.annotations.remove(pos))
        } else {
            None
        }
    }

    /// Find annotation by ID
    pub fn find(&self, id: &str) -> Option<&Annotation> {
        self.annotations.iter().find(|a| a.id == id)
    }

    /// Find mutable annotation by ID
    pub fn find_mut(&mut self, id: &str) -> Option<&mut Annotation> {
        self.annotations.iter_mut().find(|a| a.id == id)
    }

    /// Get visible annotations
    pub fn visible_annotations(&self) -> Vec<&Annotation> {
        if !self.visible {
            return Vec::new();
        }
        self.annotations.iter().filter(|a| a.visible).collect()
    }

    /// Find annotation at position
    pub fn find_at(&self, x: f64, y: f64) -> Option<&Annotation> {
        self.annotations
            .iter()
            .filter(|a| a.visible && a.interactive)
            .find(|a| a.contains(x, y))
    }

    /// Get all annotation IDs
    pub fn ids(&self) -> Vec<&str> {
        self.annotations.iter().map(|a| a.id.as_str()).collect()
    }

    /// Clear all annotations
    pub fn clear(&mut self) {
        self.annotations.clear();
    }

    /// Get count
    pub fn len(&self) -> usize {
        self.annotations.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.annotations.is_empty()
    }

    /// Set layer visibility
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Set layer opacity
    pub fn set_opacity(&mut self, opacity: f32) {
        self.opacity = opacity.clamp(0.0, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_annotation_text() {
        let ann = Annotation::text(100.0, 50.0, "Test Label");
        assert_eq!(ann.annotation_type, AnnotationType::Text);
        assert_eq!(ann.x, 100.0);
        assert_eq!(ann.y, 50.0);
        assert_eq!(ann.text, "Test Label");
    }

    #[test]
    fn test_annotation_callout() {
        let ann = Annotation::callout(100.0, 100.0, 150.0, 50.0, "Note");
        assert_eq!(ann.annotation_type, AnnotationType::Callout);
        assert_eq!(ann.x, 100.0);
        assert_eq!(ann.y, 100.0);
        assert_eq!(ann.x2, Some(150.0));
        assert_eq!(ann.y2, Some(50.0));
    }

    #[test]
    fn test_annotation_circle() {
        let ann = Annotation::circle(50.0, 50.0, 10.0);
        assert_eq!(ann.annotation_type, AnnotationType::Circle);
        assert_eq!(ann.width, Some(20.0));
    }

    #[test]
    fn test_annotation_rectangle() {
        let ann = Annotation::rectangle(0.0, 0.0, 100.0, 50.0);
        assert_eq!(ann.annotation_type, AnnotationType::Rectangle);
        let (x, y, w, h) = ann.bounds();
        assert_eq!(w, 100.0);
        assert_eq!(h, 50.0);
    }

    #[test]
    fn test_annotation_line() {
        let ann = Annotation::line(0.0, 0.0, 100.0, 100.0);
        assert_eq!(ann.annotation_type, AnnotationType::Line);
        assert_eq!(ann.secondary_position(), Some((100.0, 100.0)));
    }

    #[test]
    fn test_annotation_arrow() {
        let ann = Annotation::arrow(0.0, 0.0, 50.0, 50.0).with_arrow_style(ArrowStyle::Stealth);
        assert_eq!(ann.annotation_type, AnnotationType::Arrow);
        assert_eq!(ann.arrow_style, ArrowStyle::Stealth);
    }

    #[test]
    fn test_annotation_badge() {
        let ann = Annotation::badge(100.0, 100.0, "New", Rgba::from_hex(0xFF5722));
        assert_eq!(ann.annotation_type, AnnotationType::Badge);
        assert_eq!(ann.text, "New");
    }

    #[test]
    fn test_annotation_with_style() {
        let ann = Annotation::text(0.0, 0.0, "Styled")
            .with_background(Rgba::from_hex(0xFFFF00))
            .with_text_color(Rgba::BLACK)
            .with_font_size(16.0)
            .with_rotation(45.0);

        assert_eq!(ann.style.font_size, 16.0);
        assert_eq!(ann.rotation, 45.0);
    }

    #[test]
    fn test_annotation_contains() {
        let ann = Annotation::rectangle(10.0, 10.0, 50.0, 30.0);

        assert!(ann.contains(20.0, 20.0));
        assert!(ann.contains(10.0, 10.0));
        assert!(!ann.contains(5.0, 5.0));
        assert!(!ann.contains(100.0, 100.0));
    }

    #[test]
    fn test_annotation_connector_points() {
        let ann = Annotation::callout(100.0, 100.0, 150.0, 50.0, "Note")
            .with_connector(ConnectorStyle::Straight);

        let points = ann.connector_points();
        assert_eq!(points.len(), 2);
        assert_eq!(points[0], (100.0, 100.0));
        assert_eq!(points[1], (150.0, 50.0));
    }

    #[test]
    fn test_annotation_connector_elbow() {
        let ann = Annotation::callout(100.0, 100.0, 150.0, 50.0, "Note")
            .with_connector(ConnectorStyle::ElbowHV);

        let points = ann.connector_points();
        assert_eq!(points.len(), 3);
        assert_eq!(points[0], (100.0, 100.0));
        assert_eq!(points[1], (150.0, 100.0));
        assert_eq!(points[2], (150.0, 50.0));
    }

    #[test]
    fn test_annotation_style_default() {
        let style = AnnotationStyle::default();
        assert_eq!(style.font_size, 12.0);
        assert_eq!(style.border_radius, 4.0);
    }

    #[test]
    fn test_annotation_style_builders() {
        let minimal = AnnotationStyle::minimal();
        assert!(minimal.fill.a < 0.01);

        let highlight = AnnotationStyle::highlight();
        assert_eq!(highlight.stroke_width, 2.0);

        let dark = AnnotationStyle::dark();
        assert!(dark.text_color.r > 0.9);
    }

    #[test]
    fn test_annotation_layer() {
        let mut layer = AnnotationLayer::new("Main");

        layer.add(Annotation::text(0.0, 0.0, "A").with_id("ann1"));
        layer.add(Annotation::text(100.0, 100.0, "B").with_id("ann2"));

        assert_eq!(layer.len(), 2);
        assert!(layer.find("ann1").is_some());
        assert!(layer.find("ann3").is_none());
    }

    #[test]
    fn test_annotation_layer_remove() {
        let mut layer = AnnotationLayer::new("Test");
        layer.add(Annotation::text(0.0, 0.0, "A").with_id("a"));
        layer.add(Annotation::text(0.0, 0.0, "B").with_id("b"));

        let removed = layer.remove("a");
        assert!(removed.is_some());
        assert_eq!(layer.len(), 1);
    }

    #[test]
    fn test_annotation_layer_visibility() {
        let mut layer = AnnotationLayer::new("Test");
        layer.add(Annotation::text(0.0, 0.0, "A").with_visible(true));
        layer.add(Annotation::text(0.0, 0.0, "B").with_visible(false));

        let visible = layer.visible_annotations();
        assert_eq!(visible.len(), 1);

        layer.set_visible(false);
        let visible = layer.visible_annotations();
        assert_eq!(visible.len(), 0);
    }

    #[test]
    fn test_annotation_layer_find_at() {
        let mut layer = AnnotationLayer::new("Test");
        layer.add(
            Annotation::rectangle(10.0, 10.0, 50.0, 30.0)
                .with_id("rect")
                .with_interactive(true),
        );

        assert!(layer.find_at(20.0, 20.0).is_some());
        assert!(layer.find_at(200.0, 200.0).is_none());
    }
}

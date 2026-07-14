//! Legend component for data visualization
//!
//! Provides a configurable legend for displaying dataset colors, labels,
//! and interactive toggling of series visibility.
//!
//! # Example
//!
//! ```
//! use makepad_d3::component::{Legend, LegendItem, LegendOrientation};
//! use makepad_d3::color::Rgba;
//!
//! let legend = Legend::new()
//!     .orientation(LegendOrientation::Horizontal)
//!     .add_item("Series A", Rgba::from_hex(0x4285F4))
//!     .add_item("Series B", Rgba::from_hex(0xEA4335))
//!     .add_item("Series C", Rgba::from_hex(0x34A853));
//!
//! // Check visibility
//! assert!(legend.is_visible(0));
//! ```

use crate::color::Rgba;
use serde::{Deserialize, Serialize};

/// Shape of the legend symbol
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum LegendSymbol {
    /// Square/rectangle symbol
    #[default]
    Square,
    /// Circle symbol
    Circle,
    /// Line symbol (for line charts)
    Line,
    /// Dashed line symbol
    DashedLine,
    /// Triangle symbol
    Triangle,
    /// Diamond symbol
    Diamond,
}

/// Orientation of the legend layout
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum LegendOrientation {
    /// Horizontal layout (items side by side)
    #[default]
    Horizontal,
    /// Vertical layout (items stacked)
    Vertical,
}

/// Position of the legend relative to the chart
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum LegendPosition {
    /// Top of the chart
    Top,
    /// Bottom of the chart
    #[default]
    Bottom,
    /// Left side of the chart
    Left,
    /// Right side of the chart
    Right,
    /// Top-left corner
    TopLeft,
    /// Top-right corner
    TopRight,
    /// Bottom-left corner
    BottomLeft,
    /// Bottom-right corner
    BottomRight,
}

/// A single item in the legend
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LegendItem {
    /// Display label
    pub label: String,
    /// Color for the symbol
    #[serde(skip)]
    pub color: Rgba,
    /// Symbol type
    pub symbol: LegendSymbol,
    /// Whether this item is visible/enabled
    pub visible: bool,
    /// Optional data value for display
    pub value: Option<String>,
    /// Optional description/tooltip
    pub description: Option<String>,
}

impl LegendItem {
    /// Create a new legend item
    pub fn new(label: impl Into<String>, color: Rgba) -> Self {
        Self {
            label: label.into(),
            color,
            symbol: LegendSymbol::Square,
            visible: true,
            value: None,
            description: None,
        }
    }

    /// Set the symbol type
    pub fn with_symbol(mut self, symbol: LegendSymbol) -> Self {
        self.symbol = symbol;
        self
    }

    /// Set initial visibility
    pub fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Set an optional value to display
    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Set a description/tooltip
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Toggle visibility
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }
}

impl Default for LegendItem {
    fn default() -> Self {
        Self {
            label: String::new(),
            color: Rgba::BLACK,
            symbol: LegendSymbol::Square,
            visible: true,
            value: None,
            description: None,
        }
    }
}

/// Configuration for legend styling
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LegendStyle {
    /// Symbol size in pixels
    pub symbol_size: f64,
    /// Spacing between items
    pub item_spacing: f64,
    /// Spacing between symbol and label
    pub label_spacing: f64,
    /// Font size for labels
    pub font_size: f64,
    /// Font color for labels
    #[serde(skip)]
    pub font_color: Rgba,
    /// Background color (if any)
    #[serde(skip)]
    pub background: Option<Rgba>,
    /// Border color (if any)
    #[serde(skip)]
    pub border_color: Option<Rgba>,
    /// Border width
    pub border_width: f64,
    /// Padding inside the legend box
    pub padding: f64,
    /// Corner radius for background
    pub corner_radius: f64,
    /// Opacity for disabled items
    pub disabled_opacity: f32,
}

impl Default for LegendStyle {
    fn default() -> Self {
        Self {
            symbol_size: 12.0,
            item_spacing: 20.0,
            label_spacing: 6.0,
            font_size: 12.0,
            font_color: Rgba::rgb(0.2, 0.2, 0.2),
            background: None,
            border_color: None,
            border_width: 1.0,
            padding: 8.0,
            corner_radius: 4.0,
            disabled_opacity: 0.4,
        }
    }
}

/// Interactive legend component
///
/// Manages a collection of legend items with support for toggling,
/// styling, and layout configuration.
#[derive(Clone, Debug, Default)]
pub struct Legend {
    /// Legend items
    pub items: Vec<LegendItem>,
    /// Layout orientation
    pub orientation: LegendOrientation,
    /// Position relative to chart
    pub position: LegendPosition,
    /// Whether legend is interactive (can toggle items)
    pub interactive: bool,
    /// Styling configuration
    pub style: LegendStyle,
    /// Title for the legend
    pub title: Option<String>,
    /// Maximum items per row/column (0 = unlimited)
    pub max_items_per_line: usize,
}

impl Legend {
    /// Create a new empty legend
    pub fn new() -> Self {
        Self::default()
    }

    /// Create from a list of labels and colors
    pub fn from_pairs(pairs: &[(impl AsRef<str>, Rgba)]) -> Self {
        let items = pairs
            .iter()
            .map(|(label, color)| LegendItem::new(label.as_ref(), *color))
            .collect();
        Self {
            items,
            ..Default::default()
        }
    }

    /// Create from labels with automatic colors from a color scale
    pub fn from_labels<F>(labels: &[impl AsRef<str>], color_fn: F) -> Self
    where
        F: Fn(usize) -> Rgba,
    {
        let items = labels
            .iter()
            .enumerate()
            .map(|(i, label)| LegendItem::new(label.as_ref(), color_fn(i)))
            .collect();
        Self {
            items,
            ..Default::default()
        }
    }

    /// Set the orientation
    pub fn orientation(mut self, orientation: LegendOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Set the position
    pub fn position(mut self, position: LegendPosition) -> Self {
        self.position = position;
        self
    }

    /// Set whether the legend is interactive
    pub fn interactive(mut self, interactive: bool) -> Self {
        self.interactive = interactive;
        self
    }

    /// Set the style
    pub fn style(mut self, style: LegendStyle) -> Self {
        self.style = style;
        self
    }

    /// Set a title
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set maximum items per row/column
    pub fn max_items_per_line(mut self, max: usize) -> Self {
        self.max_items_per_line = max;
        self
    }

    /// Add an item to the legend
    pub fn add_item(mut self, label: impl Into<String>, color: Rgba) -> Self {
        self.items.push(LegendItem::new(label, color));
        self
    }

    /// Add an item with a specific symbol
    pub fn add_item_with_symbol(
        mut self,
        label: impl Into<String>,
        color: Rgba,
        symbol: LegendSymbol,
    ) -> Self {
        self.items
            .push(LegendItem::new(label, color).with_symbol(symbol));
        self
    }

    /// Add a pre-built item
    pub fn add(mut self, item: LegendItem) -> Self {
        self.items.push(item);
        self
    }

    /// Push an item mutably
    pub fn push(&mut self, item: LegendItem) {
        self.items.push(item);
    }

    /// Get number of items
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Clear all items
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// Check if an item at index is visible
    pub fn is_visible(&self, index: usize) -> bool {
        self.items.get(index).map_or(false, |item| item.visible)
    }

    /// Toggle visibility of item at index
    pub fn toggle(&mut self, index: usize) {
        if let Some(item) = self.items.get_mut(index) {
            item.toggle();
        }
    }

    /// Set visibility of item at index
    pub fn set_visible(&mut self, index: usize, visible: bool) {
        if let Some(item) = self.items.get_mut(index) {
            item.visible = visible;
        }
    }

    /// Show all items
    pub fn show_all(&mut self) {
        for item in &mut self.items {
            item.visible = true;
        }
    }

    /// Hide all items
    pub fn hide_all(&mut self) {
        for item in &mut self.items {
            item.visible = false;
        }
    }

    /// Get indices of visible items
    pub fn visible_indices(&self) -> Vec<usize> {
        self.items
            .iter()
            .enumerate()
            .filter_map(|(i, item)| if item.visible { Some(i) } else { None })
            .collect()
    }

    /// Get visible items
    pub fn visible_items(&self) -> Vec<&LegendItem> {
        self.items.iter().filter(|item| item.visible).collect()
    }

    /// Get count of visible items
    pub fn visible_count(&self) -> usize {
        self.items.iter().filter(|item| item.visible).count()
    }

    /// Find item by label
    pub fn find_by_label(&self, label: &str) -> Option<usize> {
        self.items.iter().position(|item| item.label == label)
    }

    /// Toggle item by label
    pub fn toggle_by_label(&mut self, label: &str) -> bool {
        if let Some(index) = self.find_by_label(label) {
            self.toggle(index);
            true
        } else {
            false
        }
    }

    /// Calculate layout dimensions
    ///
    /// Returns (width, height) based on current items and style.
    pub fn calculate_size(&self) -> (f64, f64) {
        if self.items.is_empty() {
            return (0.0, 0.0);
        }

        let style = &self.style;
        let item_width = style.symbol_size + style.label_spacing + self.estimate_label_width();
        let item_height = style.symbol_size.max(style.font_size);

        let (rows, cols) = self.calculate_grid();

        let width = cols as f64 * item_width + (cols - 1) as f64 * style.item_spacing;
        let height = rows as f64 * item_height + (rows - 1) as f64 * style.item_spacing;

        // Add padding
        let total_width = width + style.padding * 2.0;
        let total_height = height + style.padding * 2.0;

        // Add title height if present
        let title_height = if self.title.is_some() {
            style.font_size + style.item_spacing
        } else {
            0.0
        };

        (total_width, total_height + title_height)
    }

    /// Calculate grid dimensions (rows, columns)
    fn calculate_grid(&self) -> (usize, usize) {
        let n = self.items.len();
        if n == 0 {
            return (0, 0);
        }

        match self.orientation {
            LegendOrientation::Horizontal => {
                if self.max_items_per_line > 0 {
                    let cols = self.max_items_per_line.min(n);
                    let rows = (n + cols - 1) / cols;
                    (rows, cols)
                } else {
                    (1, n)
                }
            }
            LegendOrientation::Vertical => {
                if self.max_items_per_line > 0 {
                    let rows = self.max_items_per_line.min(n);
                    let cols = (n + rows - 1) / rows;
                    (rows, cols)
                } else {
                    (n, 1)
                }
            }
        }
    }

    /// Estimate average label width (simplified)
    fn estimate_label_width(&self) -> f64 {
        let avg_chars = self.items.iter().map(|i| i.label.len()).sum::<usize>() as f64
            / self.items.len().max(1) as f64;
        avg_chars * self.style.font_size * 0.6 // Rough estimate
    }

    /// Get item at position (for hit testing)
    ///
    /// Returns Some(index) if position is within an item's bounds.
    pub fn item_at_position(&self, x: f64, y: f64, origin_x: f64, origin_y: f64) -> Option<usize> {
        if self.items.is_empty() {
            return None;
        }

        let style = &self.style;
        let item_width = style.symbol_size + style.label_spacing + self.estimate_label_width();
        let item_height = style.symbol_size.max(style.font_size);

        let (rows, cols) = self.calculate_grid();

        // Adjust for padding and title
        let content_x = origin_x + style.padding;
        let title_offset = if self.title.is_some() {
            style.font_size + style.item_spacing
        } else {
            0.0
        };
        let content_y = origin_y + style.padding + title_offset;

        // Check if within content bounds
        let rel_x = x - content_x;
        let rel_y = y - content_y;

        if rel_x < 0.0 || rel_y < 0.0 {
            return None;
        }

        // Find grid cell
        let col = (rel_x / (item_width + style.item_spacing)).floor() as usize;
        let row = (rel_y / (item_height + style.item_spacing)).floor() as usize;

        if col >= cols || row >= rows {
            return None;
        }

        // Calculate item index based on orientation
        let index = match self.orientation {
            LegendOrientation::Horizontal => row * cols + col,
            LegendOrientation::Vertical => col * rows + row,
        };

        if index < self.items.len() {
            Some(index)
        } else {
            None
        }
    }

    /// Get item positions for rendering
    ///
    /// Returns a vector of (x, y, item) for each legend item.
    pub fn get_item_positions(&self, origin_x: f64, origin_y: f64) -> Vec<(f64, f64, &LegendItem)> {
        if self.items.is_empty() {
            return Vec::new();
        }

        let style = &self.style;
        let item_width = style.symbol_size + style.label_spacing + self.estimate_label_width();
        let item_height = style.symbol_size.max(style.font_size);

        let (rows, cols) = self.calculate_grid();

        let content_x = origin_x + style.padding;
        let title_offset = if self.title.is_some() {
            style.font_size + style.item_spacing
        } else {
            0.0
        };
        let content_y = origin_y + style.padding + title_offset;

        self.items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let (row, col) = match self.orientation {
                    LegendOrientation::Horizontal => (i / cols, i % cols),
                    LegendOrientation::Vertical => (i % rows, i / rows),
                };

                let x = content_x + col as f64 * (item_width + style.item_spacing);
                let y = content_y + row as f64 * (item_height + style.item_spacing);

                (x, y, item)
            })
            .collect()
    }
}

/// Builder for creating legends from data
pub struct LegendBuilder {
    legend: Legend,
}

impl LegendBuilder {
    /// Create a new legend builder
    pub fn new() -> Self {
        Self {
            legend: Legend::new(),
        }
    }

    /// Add items from iterator of (label, color) pairs
    pub fn items<I, S>(mut self, items: I) -> Self
    where
        I: IntoIterator<Item = (S, Rgba)>,
        S: Into<String>,
    {
        for (label, color) in items {
            self.legend.items.push(LegendItem::new(label, color));
        }
        self
    }

    /// Set orientation
    pub fn orientation(mut self, orientation: LegendOrientation) -> Self {
        self.legend.orientation = orientation;
        self
    }

    /// Set position
    pub fn position(mut self, position: LegendPosition) -> Self {
        self.legend.position = position;
        self
    }

    /// Make interactive
    pub fn interactive(mut self) -> Self {
        self.legend.interactive = true;
        self
    }

    /// Set title
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.legend.title = Some(title.into());
        self
    }

    /// Set symbol size
    pub fn symbol_size(mut self, size: f64) -> Self {
        self.legend.style.symbol_size = size;
        self
    }

    /// Set item spacing
    pub fn spacing(mut self, spacing: f64) -> Self {
        self.legend.style.item_spacing = spacing;
        self
    }

    /// Set font size
    pub fn font_size(mut self, size: f64) -> Self {
        self.legend.style.font_size = size;
        self
    }

    /// Set font color
    pub fn font_color(mut self, color: Rgba) -> Self {
        self.legend.style.font_color = color;
        self
    }

    /// Set background color
    pub fn background(mut self, color: Rgba) -> Self {
        self.legend.style.background = Some(color);
        self
    }

    /// Set border
    pub fn border(mut self, color: Rgba, width: f64) -> Self {
        self.legend.style.border_color = Some(color);
        self.legend.style.border_width = width;
        self
    }

    /// Build the legend
    pub fn build(self) -> Legend {
        self.legend
    }
}

impl Default for LegendBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_legend_item_new() {
        let item = LegendItem::new("Series A", Rgba::from_hex(0x4285F4));
        assert_eq!(item.label, "Series A");
        assert!(item.visible);
        assert_eq!(item.symbol, LegendSymbol::Square);
    }

    #[test]
    fn test_legend_item_toggle() {
        let mut item = LegendItem::new("Test", Rgba::RED);
        assert!(item.visible);
        item.toggle();
        assert!(!item.visible);
        item.toggle();
        assert!(item.visible);
    }

    #[test]
    fn test_legend_item_with_value() {
        let item = LegendItem::new("Sales", Rgba::BLUE)
            .with_value("$1,234")
            .with_description("Total sales for Q1");

        assert_eq!(item.value, Some("$1,234".to_string()));
        assert_eq!(item.description, Some("Total sales for Q1".to_string()));
    }

    #[test]
    fn test_legend_new() {
        let legend = Legend::new();
        assert!(legend.is_empty());
        assert_eq!(legend.orientation, LegendOrientation::Horizontal);
    }

    #[test]
    fn test_legend_add_items() {
        let legend = Legend::new()
            .add_item("A", Rgba::RED)
            .add_item("B", Rgba::GREEN)
            .add_item("C", Rgba::BLUE);

        assert_eq!(legend.len(), 3);
        assert_eq!(legend.items[0].label, "A");
        assert_eq!(legend.items[1].label, "B");
        assert_eq!(legend.items[2].label, "C");
    }

    #[test]
    fn test_legend_from_pairs() {
        let pairs = vec![
            ("Series 1", Rgba::from_hex(0xFF0000)),
            ("Series 2", Rgba::from_hex(0x00FF00)),
        ];
        let legend = Legend::from_pairs(&pairs);

        assert_eq!(legend.len(), 2);
        assert_eq!(legend.items[0].label, "Series 1");
    }

    #[test]
    fn test_legend_visibility() {
        let mut legend = Legend::new()
            .add_item("A", Rgba::RED)
            .add_item("B", Rgba::GREEN);

        assert!(legend.is_visible(0));
        assert!(legend.is_visible(1));

        legend.toggle(0);
        assert!(!legend.is_visible(0));
        assert!(legend.is_visible(1));

        legend.set_visible(0, true);
        assert!(legend.is_visible(0));
    }

    #[test]
    fn test_legend_show_hide_all() {
        let mut legend = Legend::new()
            .add_item("A", Rgba::RED)
            .add_item("B", Rgba::GREEN)
            .add_item("C", Rgba::BLUE);

        legend.hide_all();
        assert_eq!(legend.visible_count(), 0);

        legend.show_all();
        assert_eq!(legend.visible_count(), 3);
    }

    #[test]
    fn test_legend_visible_indices() {
        let mut legend = Legend::new()
            .add_item("A", Rgba::RED)
            .add_item("B", Rgba::GREEN)
            .add_item("C", Rgba::BLUE);

        legend.toggle(1);
        let visible = legend.visible_indices();
        assert_eq!(visible, vec![0, 2]);
    }

    #[test]
    fn test_legend_find_by_label() {
        let legend = Legend::new()
            .add_item("Series A", Rgba::RED)
            .add_item("Series B", Rgba::GREEN);

        assert_eq!(legend.find_by_label("Series A"), Some(0));
        assert_eq!(legend.find_by_label("Series B"), Some(1));
        assert_eq!(legend.find_by_label("Series C"), None);
    }

    #[test]
    fn test_legend_toggle_by_label() {
        let mut legend = Legend::new()
            .add_item("A", Rgba::RED)
            .add_item("B", Rgba::GREEN);

        assert!(legend.toggle_by_label("A"));
        assert!(!legend.is_visible(0));

        assert!(!legend.toggle_by_label("NonExistent"));
    }

    #[test]
    fn test_legend_orientation() {
        let legend = Legend::new()
            .orientation(LegendOrientation::Vertical)
            .position(LegendPosition::Right);

        assert_eq!(legend.orientation, LegendOrientation::Vertical);
        assert_eq!(legend.position, LegendPosition::Right);
    }

    #[test]
    fn test_legend_calculate_size() {
        let legend = Legend::new()
            .add_item("Series A", Rgba::RED)
            .add_item("Series B", Rgba::GREEN);

        let (width, height) = legend.calculate_size();
        assert!(width > 0.0);
        assert!(height > 0.0);
    }

    #[test]
    fn test_legend_builder() {
        let legend = LegendBuilder::new()
            .items(vec![
                ("A".to_string(), Rgba::RED),
                ("B".to_string(), Rgba::GREEN),
            ])
            .title("Chart Legend")
            .orientation(LegendOrientation::Vertical)
            .interactive()
            .symbol_size(16.0)
            .build();

        assert_eq!(legend.len(), 2);
        assert_eq!(legend.title, Some("Chart Legend".to_string()));
        assert!(legend.interactive);
        assert_eq!(legend.style.symbol_size, 16.0);
    }

    #[test]
    fn test_legend_get_item_positions() {
        let legend = Legend::new()
            .add_item("A", Rgba::RED)
            .add_item("B", Rgba::GREEN)
            .add_item("C", Rgba::BLUE);

        let positions = legend.get_item_positions(0.0, 0.0);
        assert_eq!(positions.len(), 3);

        // First item should be at origin + padding
        let (x, y, _) = positions[0];
        assert!(x >= 0.0);
        assert!(y >= 0.0);
    }

    #[test]
    fn test_legend_style_default() {
        let style = LegendStyle::default();
        assert_eq!(style.symbol_size, 12.0);
        assert_eq!(style.font_size, 12.0);
        assert!(style.background.is_none());
    }
}

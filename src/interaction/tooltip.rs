//! Tooltip content structures for data visualization
//!
//! Provides data structures for building tooltips that display
//! information about data points.

use crate::color::Rgba;
use serde::{Deserialize, Serialize};

/// A single item in a tooltip
///
/// Represents a label-value pair with an optional color indicator.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TooltipItem {
    /// Label for the item (e.g., "Revenue")
    pub label: String,
    /// Formatted value (e.g., "$1,234.56")
    pub value: String,
    /// Optional color indicator
    #[serde(skip)]
    pub color: Option<Rgba>,
}

impl TooltipItem {
    /// Create a new tooltip item
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            color: None,
        }
    }

    /// Set the color indicator
    pub fn with_color(mut self, color: Rgba) -> Self {
        self.color = Some(color);
        self
    }

    /// Create from label and numeric value
    pub fn from_number(label: impl Into<String>, value: f64) -> Self {
        Self::new(label, format!("{:.2}", value))
    }

    /// Create from label and integer value
    pub fn from_int(label: impl Into<String>, value: i64) -> Self {
        Self::new(label, value.to_string())
    }

    /// Create from label and percentage
    pub fn from_percent(label: impl Into<String>, value: f64) -> Self {
        Self::new(label, format!("{:.1}%", value * 100.0))
    }
}

/// Content for a tooltip display
///
/// Contains a title and a list of items to display in the tooltip.
///
/// # Example
///
/// ```
/// use makepad_d3::interaction::TooltipContent;
/// use makepad_d3::color::Rgba;
///
/// let tooltip = TooltipContent::new("January 2024")
///     .add_item("Revenue", "$12,345")
///     .add_item_with_color("Profit", "$3,456", Rgba::from_hex(0x4CAF50))
///     .add_item("Growth", "+15%");
///
/// assert_eq!(tooltip.title, "January 2024");
/// assert_eq!(tooltip.items.len(), 3);
/// ```
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TooltipContent {
    /// Title of the tooltip
    pub title: String,
    /// List of items to display
    pub items: Vec<TooltipItem>,
    /// Optional subtitle
    pub subtitle: Option<String>,
    /// Optional footer text
    pub footer: Option<String>,
}

impl TooltipContent {
    /// Create a new tooltip content with a title
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            items: Vec::new(),
            subtitle: None,
            footer: None,
        }
    }

    /// Create an empty tooltip
    pub fn empty() -> Self {
        Self::default()
    }

    /// Set the title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set the subtitle
    pub fn with_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Set the footer
    pub fn with_footer(mut self, footer: impl Into<String>) -> Self {
        self.footer = Some(footer.into());
        self
    }

    /// Add an item to the tooltip
    pub fn add_item(mut self, label: impl Into<String>, value: impl Into<String>) -> Self {
        self.items.push(TooltipItem::new(label, value));
        self
    }

    /// Add an item with a color indicator
    pub fn add_item_with_color(
        mut self,
        label: impl Into<String>,
        value: impl Into<String>,
        color: Rgba,
    ) -> Self {
        self.items
            .push(TooltipItem::new(label, value).with_color(color));
        self
    }

    /// Add a numeric item
    pub fn add_number(mut self, label: impl Into<String>, value: f64) -> Self {
        self.items.push(TooltipItem::from_number(label, value));
        self
    }

    /// Add a percentage item
    pub fn add_percent(mut self, label: impl Into<String>, value: f64) -> Self {
        self.items.push(TooltipItem::from_percent(label, value));
        self
    }

    /// Add a pre-built item
    pub fn add(mut self, item: TooltipItem) -> Self {
        self.items.push(item);
        self
    }

    /// Push an item mutably
    pub fn push(&mut self, item: TooltipItem) {
        self.items.push(item);
    }

    /// Check if tooltip is empty
    pub fn is_empty(&self) -> bool {
        self.title.is_empty() && self.items.is_empty()
    }

    /// Get the number of items
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Clear all items
    pub fn clear(&mut self) {
        self.items.clear();
    }
}

/// Position hint for tooltip placement
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TooltipPosition {
    /// Position above the target
    Top,
    /// Position below the target
    #[default]
    Bottom,
    /// Position to the left of the target
    Left,
    /// Position to the right of the target
    Right,
    /// Position at cursor location
    Cursor,
}

/// Tooltip state for tracking visibility and position
#[derive(Clone, Debug, Default)]
pub struct TooltipState {
    /// Whether the tooltip is visible
    pub visible: bool,
    /// X position in screen coordinates
    pub x: f64,
    /// Y position in screen coordinates
    pub y: f64,
    /// The content to display
    pub content: TooltipContent,
    /// Preferred position
    pub position: TooltipPosition,
}

impl TooltipState {
    /// Create a new tooltip state
    pub fn new() -> Self {
        Self::default()
    }

    /// Show the tooltip with content at a position
    pub fn show(&mut self, x: f64, y: f64, content: TooltipContent) {
        self.visible = true;
        self.x = x;
        self.y = y;
        self.content = content;
    }

    /// Show at position with offset
    pub fn show_with_offset(
        &mut self,
        x: f64,
        y: f64,
        offset_x: f64,
        offset_y: f64,
        content: TooltipContent,
    ) {
        self.show(x + offset_x, y + offset_y, content);
    }

    /// Hide the tooltip
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Update position while keeping content
    pub fn update_position(&mut self, x: f64, y: f64) {
        self.x = x;
        self.y = y;
    }

    /// Check if visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Set preferred position
    pub fn set_position(&mut self, position: TooltipPosition) {
        self.position = position;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tooltip_item_new() {
        let item = TooltipItem::new("Revenue", "$1,234");
        assert_eq!(item.label, "Revenue");
        assert_eq!(item.value, "$1,234");
        assert!(item.color.is_none());
    }

    #[test]
    fn test_tooltip_item_with_color() {
        let item = TooltipItem::new("Revenue", "$1,234").with_color(Rgba::from_hex(0xFF0000));
        assert!(item.color.is_some());
        assert!((item.color.unwrap().r - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_tooltip_item_from_number() {
        let item = TooltipItem::from_number("Value", 1234.567);
        assert_eq!(item.label, "Value");
        assert_eq!(item.value, "1234.57");
    }

    #[test]
    fn test_tooltip_item_from_percent() {
        let item = TooltipItem::from_percent("Growth", 0.156);
        assert_eq!(item.label, "Growth");
        assert_eq!(item.value, "15.6%");
    }

    #[test]
    fn test_tooltip_content_new() {
        let tooltip = TooltipContent::new("January");
        assert_eq!(tooltip.title, "January");
        assert!(tooltip.items.is_empty());
    }

    #[test]
    fn test_tooltip_content_builder() {
        let tooltip = TooltipContent::new("Q1 2024")
            .with_subtitle("Financial Report")
            .add_item("Revenue", "$100,000")
            .add_item("Expenses", "$75,000")
            .add_number("Profit", 25000.0)
            .add_percent("Margin", 0.25)
            .with_footer("All values in USD");

        assert_eq!(tooltip.title, "Q1 2024");
        assert_eq!(tooltip.subtitle, Some("Financial Report".to_string()));
        assert_eq!(tooltip.items.len(), 4);
        assert_eq!(tooltip.footer, Some("All values in USD".to_string()));
    }

    #[test]
    fn test_tooltip_content_add_with_color() {
        let tooltip = TooltipContent::new("Sales")
            .add_item_with_color("Product A", "500", Rgba::from_hex(0x4285F4))
            .add_item_with_color("Product B", "300", Rgba::from_hex(0xEA4335));

        assert_eq!(tooltip.items.len(), 2);
        assert!(tooltip.items[0].color.is_some());
        assert!(tooltip.items[1].color.is_some());
    }

    #[test]
    fn test_tooltip_content_is_empty() {
        let empty = TooltipContent::empty();
        assert!(empty.is_empty());

        let with_title = TooltipContent::new("Title");
        assert!(!with_title.is_empty());

        let with_items = TooltipContent::empty().add_item("A", "B");
        assert!(!with_items.is_empty());
    }

    #[test]
    fn test_tooltip_state() {
        let mut state = TooltipState::new();
        assert!(!state.is_visible());

        state.show(100.0, 200.0, TooltipContent::new("Test"));
        assert!(state.is_visible());
        assert_eq!(state.x, 100.0);
        assert_eq!(state.y, 200.0);

        state.update_position(150.0, 250.0);
        assert_eq!(state.x, 150.0);
        assert_eq!(state.y, 250.0);

        state.hide();
        assert!(!state.is_visible());
    }

    #[test]
    fn test_tooltip_state_with_offset() {
        let mut state = TooltipState::new();
        state.show_with_offset(100.0, 100.0, 10.0, -20.0, TooltipContent::new("Test"));
        assert_eq!(state.x, 110.0);
        assert_eq!(state.y, 80.0);
    }

    #[test]
    fn test_tooltip_position_default() {
        let state = TooltipState::new();
        assert_eq!(state.position, TooltipPosition::Bottom);
    }

    #[test]
    fn test_tooltip_content_clear() {
        let mut tooltip = TooltipContent::new("Test")
            .add_item("A", "1")
            .add_item("B", "2");

        assert_eq!(tooltip.len(), 2);
        tooltip.clear();
        assert_eq!(tooltip.len(), 0);
    }
}

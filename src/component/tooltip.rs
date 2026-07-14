//! Tooltip widget component for data visualization
//!
//! Provides a configurable tooltip widget that can display data point
//! information with automatic positioning and styling.
//!
//! # Example
//!
//! ```
//! use makepad_d3::component::{TooltipWidget, TooltipConfig, TooltipAnchor};
//! use makepad_d3::interaction::TooltipContent;
//! use makepad_d3::color::Rgba;
//!
//! let config = TooltipConfig::default()
//!     .background(Rgba::from_hex(0x333333))
//!     .border_radius(4.0)
//!     .padding(8.0);
//!
//! let mut tooltip = TooltipWidget::new(config);
//!
//! // Show tooltip at cursor
//! tooltip.show_at(100.0, 200.0, TooltipContent::new("January"));
//! ```

use crate::color::Rgba;
use crate::interaction::{TooltipContent, TooltipPosition, TooltipState};
use serde::{Deserialize, Serialize};

/// Anchor point for tooltip positioning
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TooltipAnchor {
    /// Anchor at top-left
    TopLeft,
    /// Anchor at top-center
    TopCenter,
    /// Anchor at top-right
    TopRight,
    /// Anchor at middle-left
    MiddleLeft,
    /// Anchor at center
    #[default]
    Center,
    /// Anchor at middle-right
    MiddleRight,
    /// Anchor at bottom-left
    BottomLeft,
    /// Anchor at bottom-center
    BottomCenter,
    /// Anchor at bottom-right
    BottomRight,
}

/// Follow mode for tooltip positioning
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TooltipFollowMode {
    /// Tooltip stays at fixed position
    Fixed,
    /// Tooltip follows cursor with offset
    #[default]
    Cursor,
    /// Tooltip snaps to data points
    DataPoint,
}

/// Configuration for tooltip appearance and behavior
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TooltipConfig {
    /// Background color
    #[serde(skip)]
    pub background: Rgba,
    /// Border color
    #[serde(skip)]
    pub border_color: Rgba,
    /// Border width in pixels
    pub border_width: f64,
    /// Border radius for rounded corners
    pub border_radius: f64,
    /// Inner padding
    pub padding: f64,
    /// Shadow offset X
    pub shadow_offset_x: f64,
    /// Shadow offset Y
    pub shadow_offset_y: f64,
    /// Shadow blur radius
    pub shadow_blur: f64,
    /// Shadow color
    #[serde(skip)]
    pub shadow_color: Rgba,
    /// Font size for title
    pub title_font_size: f64,
    /// Font size for items
    pub item_font_size: f64,
    /// Title color
    #[serde(skip)]
    pub title_color: Rgba,
    /// Label color
    #[serde(skip)]
    pub label_color: Rgba,
    /// Value color
    #[serde(skip)]
    pub value_color: Rgba,
    /// Color swatch size
    pub swatch_size: f64,
    /// Spacing between items
    pub item_spacing: f64,
    /// Offset from cursor/anchor point
    pub offset_x: f64,
    /// Offset from cursor/anchor point
    pub offset_y: f64,
    /// Anchor point
    pub anchor: TooltipAnchor,
    /// Follow mode
    pub follow_mode: TooltipFollowMode,
    /// Show animation duration (ms)
    pub show_delay: u32,
    /// Hide animation duration (ms)
    pub hide_delay: u32,
    /// Maximum width (0 = auto)
    pub max_width: f64,
    /// Show pointer/arrow
    pub show_pointer: bool,
    /// Pointer size
    pub pointer_size: f64,
}

impl Default for TooltipConfig {
    fn default() -> Self {
        Self {
            background: Rgba::new(0.2, 0.2, 0.2, 0.95),
            border_color: Rgba::new(0.3, 0.3, 0.3, 1.0),
            border_width: 1.0,
            border_radius: 4.0,
            padding: 10.0,
            shadow_offset_x: 0.0,
            shadow_offset_y: 2.0,
            shadow_blur: 8.0,
            shadow_color: Rgba::new(0.0, 0.0, 0.0, 0.2),
            title_font_size: 14.0,
            item_font_size: 12.0,
            title_color: Rgba::WHITE,
            label_color: Rgba::new(0.7, 0.7, 0.7, 1.0),
            value_color: Rgba::WHITE,
            swatch_size: 10.0,
            item_spacing: 6.0,
            offset_x: 10.0,
            offset_y: 10.0,
            anchor: TooltipAnchor::TopLeft,
            follow_mode: TooltipFollowMode::Cursor,
            show_delay: 0,
            hide_delay: 100,
            max_width: 300.0,
            show_pointer: false,
            pointer_size: 8.0,
        }
    }
}

impl TooltipConfig {
    /// Create a light-themed tooltip config
    pub fn light() -> Self {
        Self {
            background: Rgba::new(1.0, 1.0, 1.0, 0.98),
            border_color: Rgba::new(0.8, 0.8, 0.8, 1.0),
            title_color: Rgba::new(0.1, 0.1, 0.1, 1.0),
            label_color: Rgba::new(0.4, 0.4, 0.4, 1.0),
            value_color: Rgba::new(0.1, 0.1, 0.1, 1.0),
            shadow_color: Rgba::new(0.0, 0.0, 0.0, 0.15),
            ..Default::default()
        }
    }

    /// Create a dark-themed tooltip config
    pub fn dark() -> Self {
        Self::default()
    }

    /// Set background color
    pub fn background(mut self, color: Rgba) -> Self {
        self.background = color;
        self
    }

    /// Set border
    pub fn border(mut self, color: Rgba, width: f64) -> Self {
        self.border_color = color;
        self.border_width = width;
        self
    }

    /// Set border radius
    pub fn border_radius(mut self, radius: f64) -> Self {
        self.border_radius = radius;
        self
    }

    /// Set padding
    pub fn padding(mut self, padding: f64) -> Self {
        self.padding = padding;
        self
    }

    /// Set shadow
    pub fn shadow(mut self, offset_x: f64, offset_y: f64, blur: f64, color: Rgba) -> Self {
        self.shadow_offset_x = offset_x;
        self.shadow_offset_y = offset_y;
        self.shadow_blur = blur;
        self.shadow_color = color;
        self
    }

    /// Set offset from cursor
    pub fn offset(mut self, x: f64, y: f64) -> Self {
        self.offset_x = x;
        self.offset_y = y;
        self
    }

    /// Set anchor point
    pub fn anchor(mut self, anchor: TooltipAnchor) -> Self {
        self.anchor = anchor;
        self
    }

    /// Set follow mode
    pub fn follow_mode(mut self, mode: TooltipFollowMode) -> Self {
        self.follow_mode = mode;
        self
    }

    /// Set maximum width
    pub fn max_width(mut self, width: f64) -> Self {
        self.max_width = width;
        self
    }

    /// Enable/disable pointer arrow
    pub fn pointer(mut self, show: bool, size: f64) -> Self {
        self.show_pointer = show;
        self.pointer_size = size;
        self
    }

    /// Set delays
    pub fn delays(mut self, show: u32, hide: u32) -> Self {
        self.show_delay = show;
        self.hide_delay = hide;
        self
    }
}

/// Tooltip widget for displaying data information
#[derive(Clone, Debug)]
pub struct TooltipWidget {
    /// Configuration
    pub config: TooltipConfig,
    /// Current state
    pub state: TooltipState,
    /// Viewport bounds for clamping (x, y, width, height)
    pub viewport: (f64, f64, f64, f64),
    /// Calculated tooltip size (width, height)
    calculated_size: (f64, f64),
    /// Target position before clamping
    target_x: f64,
    /// Target position before clamping
    target_y: f64,
}

impl Default for TooltipWidget {
    fn default() -> Self {
        Self::new(TooltipConfig::default())
    }
}

impl TooltipWidget {
    /// Create a new tooltip widget with configuration
    pub fn new(config: TooltipConfig) -> Self {
        Self {
            config,
            state: TooltipState::new(),
            viewport: (0.0, 0.0, f64::MAX, f64::MAX),
            calculated_size: (0.0, 0.0),
            target_x: 0.0,
            target_y: 0.0,
        }
    }

    /// Set viewport bounds for tooltip clamping
    pub fn set_viewport(&mut self, x: f64, y: f64, width: f64, height: f64) {
        self.viewport = (x, y, width, height);
    }

    /// Show tooltip at cursor position
    pub fn show_at(&mut self, x: f64, y: f64, content: TooltipContent) {
        self.target_x = x;
        self.target_y = y;
        self.calculated_size = self.calculate_size(&content);
        let (final_x, final_y) = self.calculate_position();
        self.state.show(final_x, final_y, content);
    }

    /// Show tooltip with specified position preference
    pub fn show_at_position(
        &mut self,
        x: f64,
        y: f64,
        content: TooltipContent,
        position: TooltipPosition,
    ) {
        self.target_x = x;
        self.target_y = y;
        self.state.position = position;
        self.calculated_size = self.calculate_size(&content);
        let (final_x, final_y) = self.calculate_position();
        self.state.show(final_x, final_y, content);
    }

    /// Update cursor position (for follow mode)
    pub fn update_cursor(&mut self, x: f64, y: f64) {
        if self.state.visible && self.config.follow_mode == TooltipFollowMode::Cursor {
            self.target_x = x;
            self.target_y = y;
            let (final_x, final_y) = self.calculate_position();
            self.state.update_position(final_x, final_y);
        }
    }

    /// Hide the tooltip
    pub fn hide(&mut self) {
        self.state.hide();
    }

    /// Check if visible
    pub fn is_visible(&self) -> bool {
        self.state.is_visible()
    }

    /// Get current content
    pub fn content(&self) -> &TooltipContent {
        &self.state.content
    }

    /// Get tooltip position (x, y)
    pub fn position(&self) -> (f64, f64) {
        (self.state.x, self.state.y)
    }

    /// Get tooltip size (width, height)
    pub fn size(&self) -> (f64, f64) {
        self.calculated_size
    }

    /// Get bounding box (x, y, width, height)
    pub fn bounds(&self) -> (f64, f64, f64, f64) {
        (
            self.state.x,
            self.state.y,
            self.calculated_size.0,
            self.calculated_size.1,
        )
    }

    /// Calculate tooltip size based on content
    fn calculate_size(&self, content: &TooltipContent) -> (f64, f64) {
        let config = &self.config;
        let padding = config.padding * 2.0;

        // Estimate width based on content
        let title_width = content.title.len() as f64 * config.title_font_size * 0.6;

        let items_width = content
            .items
            .iter()
            .map(|item| {
                let label_width = item.label.len() as f64 * config.item_font_size * 0.6;
                let value_width = item.value.len() as f64 * config.item_font_size * 0.6;
                let swatch_width = if item.color.is_some() {
                    config.swatch_size + 6.0
                } else {
                    0.0
                };
                swatch_width + label_width + 10.0 + value_width
            })
            .fold(0.0_f64, |a, b| a.max(b));

        let content_width = title_width.max(items_width);
        let width = if config.max_width > 0.0 {
            content_width.min(config.max_width - padding) + padding
        } else {
            content_width + padding
        };

        // Calculate height
        let title_height = if !content.title.is_empty() {
            config.title_font_size + config.item_spacing
        } else {
            0.0
        };

        let subtitle_height = if content.subtitle.is_some() {
            config.item_font_size + config.item_spacing
        } else {
            0.0
        };

        let items_height = if content.items.is_empty() {
            0.0
        } else {
            content.items.len() as f64 * (config.item_font_size + config.item_spacing)
                - config.item_spacing
        };

        let footer_height = if content.footer.is_some() {
            config.item_spacing + config.item_font_size
        } else {
            0.0
        };

        let height = title_height + subtitle_height + items_height + footer_height + padding;

        (width, height)
    }

    /// Calculate final position with clamping
    fn calculate_position(&self) -> (f64, f64) {
        let config = &self.config;
        let (width, height) = self.calculated_size;
        let (vp_x, vp_y, vp_w, vp_h) = self.viewport;

        // Apply offset based on anchor
        let (mut x, mut y) = match config.anchor {
            TooltipAnchor::TopLeft => (
                self.target_x + config.offset_x,
                self.target_y + config.offset_y,
            ),
            TooltipAnchor::TopCenter => {
                (self.target_x - width / 2.0, self.target_y + config.offset_y)
            }
            TooltipAnchor::TopRight => (
                self.target_x - width - config.offset_x,
                self.target_y + config.offset_y,
            ),
            TooltipAnchor::MiddleLeft => (
                self.target_x + config.offset_x,
                self.target_y - height / 2.0,
            ),
            TooltipAnchor::Center => (self.target_x - width / 2.0, self.target_y - height / 2.0),
            TooltipAnchor::MiddleRight => (
                self.target_x - width - config.offset_x,
                self.target_y - height / 2.0,
            ),
            TooltipAnchor::BottomLeft => (
                self.target_x + config.offset_x,
                self.target_y - height - config.offset_y,
            ),
            TooltipAnchor::BottomCenter => (
                self.target_x - width / 2.0,
                self.target_y - height - config.offset_y,
            ),
            TooltipAnchor::BottomRight => (
                self.target_x - width - config.offset_x,
                self.target_y - height - config.offset_y,
            ),
        };

        // Clamp to viewport
        x = x.max(vp_x).min(vp_x + vp_w - width);
        y = y.max(vp_y).min(vp_y + vp_h - height);

        (x, y)
    }

    /// Get pointer position if enabled
    ///
    /// Returns (x, y, direction) where direction indicates which side
    /// the pointer should appear on (0=top, 1=right, 2=bottom, 3=left).
    pub fn pointer_position(&self) -> Option<(f64, f64, u8)> {
        if !self.config.show_pointer || !self.state.visible {
            return None;
        }

        let (width, height) = self.calculated_size;
        let (x, y) = (self.state.x, self.state.y);

        // Determine which side of the tooltip is closest to the target
        let center_x = x + width / 2.0;
        let center_y = y + height / 2.0;

        let dx = self.target_x - center_x;
        let dy = self.target_y - center_y;

        if dx.abs() > dy.abs() {
            if dx > 0.0 {
                // Target is to the right
                Some((x + width, center_y, 1))
            } else {
                // Target is to the left
                Some((x, center_y, 3))
            }
        } else if dy > 0.0 {
            // Target is below
            Some((center_x, y + height, 2))
        } else {
            // Target is above
            Some((center_x, y, 0))
        }
    }
}

/// Builder for creating tooltips from data points
pub struct DataTooltipBuilder {
    title: String,
    subtitle: Option<String>,
    footer: Option<String>,
    items: Vec<(String, String, Option<Rgba>)>,
}

impl DataTooltipBuilder {
    /// Create a new builder
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            subtitle: None,
            footer: None,
            items: Vec::new(),
        }
    }

    /// Set subtitle
    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Set footer
    pub fn footer(mut self, footer: impl Into<String>) -> Self {
        self.footer = Some(footer.into());
        self
    }

    /// Add a labeled value
    pub fn add(mut self, label: impl Into<String>, value: impl Into<String>) -> Self {
        self.items.push((label.into(), value.into(), None));
        self
    }

    /// Add a labeled value with color
    pub fn add_colored(
        mut self,
        label: impl Into<String>,
        value: impl Into<String>,
        color: Rgba,
    ) -> Self {
        self.items.push((label.into(), value.into(), Some(color)));
        self
    }

    /// Add a numeric value with formatting
    pub fn add_number(self, label: impl Into<String>, value: f64) -> Self {
        self.add(label, format!("{:.2}", value))
    }

    /// Add a percentage value
    pub fn add_percent(self, label: impl Into<String>, value: f64) -> Self {
        self.add(label, format!("{:.1}%", value * 100.0))
    }

    /// Add a currency value
    pub fn add_currency(self, label: impl Into<String>, value: f64, symbol: &str) -> Self {
        self.add(label, format!("{}{:.2}", symbol, value))
    }

    /// Build the tooltip content
    pub fn build(self) -> TooltipContent {
        let mut content = TooltipContent::new(self.title);

        if let Some(subtitle) = self.subtitle {
            content = content.with_subtitle(subtitle);
        }

        for (label, value, color) in self.items {
            if let Some(c) = color {
                content = content.add_item_with_color(label, value, c);
            } else {
                content = content.add_item(label, value);
            }
        }

        if let Some(footer) = self.footer {
            content = content.with_footer(footer);
        }

        content
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tooltip_config_default() {
        let config = TooltipConfig::default();
        assert_eq!(config.border_radius, 4.0);
        assert_eq!(config.padding, 10.0);
    }

    #[test]
    fn test_tooltip_config_light() {
        let config = TooltipConfig::light();
        // Light theme should have white background
        assert!(config.background.r > 0.9);
    }

    #[test]
    fn test_tooltip_config_builder() {
        let config = TooltipConfig::default()
            .background(Rgba::from_hex(0x333333))
            .border_radius(8.0)
            .padding(12.0)
            .offset(15.0, 15.0);

        assert_eq!(config.border_radius, 8.0);
        assert_eq!(config.padding, 12.0);
        assert_eq!(config.offset_x, 15.0);
    }

    #[test]
    fn test_tooltip_widget_new() {
        let widget = TooltipWidget::default();
        assert!(!widget.is_visible());
    }

    #[test]
    fn test_tooltip_widget_show_hide() {
        let mut widget = TooltipWidget::default();
        let content = TooltipContent::new("Test");

        widget.show_at(100.0, 200.0, content);
        assert!(widget.is_visible());

        widget.hide();
        assert!(!widget.is_visible());
    }

    #[test]
    fn test_tooltip_widget_position() {
        let mut widget = TooltipWidget::default();
        let content = TooltipContent::new("Test").add_item("Value", "123");

        widget.show_at(100.0, 200.0, content);

        let (x, y) = widget.position();
        // Position should include offset
        assert!(x >= 100.0);
        assert!(y >= 200.0);
    }

    #[test]
    fn test_tooltip_widget_viewport_clamping() {
        let mut widget = TooltipWidget::new(TooltipConfig::default());
        widget.set_viewport(0.0, 0.0, 500.0, 500.0);

        // Try to show tooltip near edge
        let content = TooltipContent::new("Test");
        widget.show_at(480.0, 480.0, content);

        let (x, y) = widget.position();
        let (w, h) = widget.size();

        // Should be clamped within viewport
        assert!(x + w <= 500.0);
        assert!(y + h <= 500.0);
    }

    #[test]
    fn test_tooltip_widget_update_cursor() {
        let mut widget =
            TooltipWidget::new(TooltipConfig::default().follow_mode(TooltipFollowMode::Cursor));

        widget.show_at(100.0, 100.0, TooltipContent::new("Test"));
        let (x1, y1) = widget.position();

        widget.update_cursor(200.0, 200.0);
        let (x2, y2) = widget.position();

        // Position should have changed
        assert!(x2 > x1);
        assert!(y2 > y1);
    }

    #[test]
    fn test_tooltip_widget_bounds() {
        let mut widget = TooltipWidget::default();
        widget.show_at(50.0, 50.0, TooltipContent::new("Test").add_item("A", "1"));

        let (x, y, w, h) = widget.bounds();
        assert!(w > 0.0);
        assert!(h > 0.0);
        assert!(x >= 0.0);
        assert!(y >= 0.0);
    }

    #[test]
    fn test_data_tooltip_builder() {
        let content = DataTooltipBuilder::new("January 2024")
            .subtitle("Sales Report")
            .add("Revenue", "$12,345")
            .add_colored("Profit", "$3,456", Rgba::from_hex(0x4CAF50))
            .add_number("Units", 567.89)
            .add_percent("Growth", 0.156)
            .footer("All values in USD")
            .build();

        assert_eq!(content.title, "January 2024");
        assert_eq!(content.subtitle, Some("Sales Report".to_string()));
        assert_eq!(content.items.len(), 4);
        assert_eq!(content.footer, Some("All values in USD".to_string()));
    }

    #[test]
    fn test_tooltip_anchor_positions() {
        let config = TooltipConfig::default().anchor(TooltipAnchor::BottomRight);
        let mut widget = TooltipWidget::new(config);

        widget.show_at(200.0, 200.0, TooltipContent::new("Test"));
        let (x, y) = widget.position();

        // Bottom-right anchor should position tooltip to the left and above
        assert!(x < 200.0);
        assert!(y < 200.0);
    }

    #[test]
    fn test_tooltip_pointer_position() {
        let config = TooltipConfig::default().pointer(true, 8.0);
        let mut widget = TooltipWidget::new(config);

        widget.show_at(100.0, 100.0, TooltipContent::new("Test"));

        let pointer = widget.pointer_position();
        assert!(pointer.is_some());
    }
}

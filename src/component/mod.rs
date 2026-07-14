//! Reusable UI components for data visualization
//!
//! This module provides configurable chart components that can be composed
//! to create interactive visualizations:
//!
//! # Components
//!
//! - [`Legend`]: Interactive legend for displaying series colors and labels
//! - [`TooltipWidget`]: Configurable tooltip for data point information
//! - [`Crosshair`]: Cursor tracking with guide lines
//! - [`Annotation`]: Labels, callouts, and markers for chart annotations
//! - [`ReferenceLine`]: Horizontal/vertical lines for thresholds and targets
//!
//! # Example
//!
//! ```
//! use makepad_d3::component::{Legend, TooltipWidget, Crosshair};
//! use makepad_d3::color::Rgba;
//!
//! // Create an interactive legend
//! let legend = Legend::new()
//!     .add_item("Revenue", Rgba::from_hex(0x4285F4))
//!     .add_item("Expenses", Rgba::from_hex(0xEA4335))
//!     .interactive(true);
//!
//! // Create a tooltip widget
//! let tooltip = TooltipWidget::default();
//!
//! // Create a crosshair
//! let crosshair = Crosshair::new()
//!     .bounds(0.0, 0.0, 800.0, 400.0)
//!     .show_labels(true);
//! ```
//!
//! # Legend Example
//!
//! ```
//! use makepad_d3::component::{Legend, LegendOrientation, LegendPosition, LegendBuilder};
//! use makepad_d3::color::Rgba;
//!
//! // Using builder pattern
//! let legend = LegendBuilder::new()
//!     .items(vec![
//!         ("Series A".to_string(), Rgba::from_hex(0x4285F4)),
//!         ("Series B".to_string(), Rgba::from_hex(0xEA4335)),
//!         ("Series C".to_string(), Rgba::from_hex(0x34A853)),
//!     ])
//!     .orientation(LegendOrientation::Horizontal)
//!     .position(LegendPosition::Bottom)
//!     .interactive()
//!     .build();
//!
//! // Check visibility
//! assert!(legend.is_visible(0));
//! ```
//!
//! # Annotation Example
//!
//! ```
//! use makepad_d3::component::{Annotation, AnnotationLayer, AnnotationStyle};
//! use makepad_d3::color::Rgba;
//!
//! let mut layer = AnnotationLayer::new("highlights");
//!
//! // Add a text annotation
//! layer.add(Annotation::text(100.0, 50.0, "Peak"));
//!
//! // Add a callout with arrow
//! layer.add(
//!     Annotation::callout(150.0, 75.0, 200.0, 30.0, "Important!")
//!         .with_arrow(true)
//!         .with_style(AnnotationStyle::highlight())
//! );
//!
//! // Add a highlighted region
//! layer.add(
//!     Annotation::rectangle(200.0, 100.0, 100.0, 50.0)
//!         .with_id("region1")
//!         .with_background(Rgba::new(1.0, 1.0, 0.0, 0.2))
//! );
//! ```
//!
//! # Reference Line Example
//!
//! ```
//! use makepad_d3::component::{ReferenceLine, ReferenceLineSet, ReferenceLineSetBuilder};
//! use makepad_d3::color::Rgba;
//!
//! // Using the builder
//! let lines = ReferenceLineSetBuilder::new()
//!     .threshold(90.0, "Critical Threshold")
//!     .target(75.0, "Target Goal")
//!     .average(55.0, "Average")
//!     .baseline(0.0)
//!     .build();
//!
//! // Or create individual lines
//! let custom = ReferenceLine::horizontal(80.0, "Warning Level")
//!     .with_id("warning")
//!     .with_style(
//!         makepad_d3::component::ReferenceLineStyle::dashed(
//!             Rgba::from_hex(0xFF9800),
//!             2.0
//!         )
//!     );
//! ```

mod annotation;
mod crosshair;
mod legend;
mod reference_line;
mod tooltip;

// Legend exports
pub use legend::{
    Legend, LegendBuilder, LegendItem, LegendOrientation, LegendPosition, LegendStyle, LegendSymbol,
};

// Tooltip exports
pub use tooltip::{
    DataTooltipBuilder, TooltipAnchor, TooltipConfig, TooltipFollowMode, TooltipWidget,
};

// Crosshair exports
pub use crosshair::{
    Crosshair, CrosshairBuilder, CrosshairLabelConfig, CrosshairLine, CrosshairLineStyle,
    CrosshairMode, CrosshairStyle, LabelPosition, SnapPoint,
};

// Annotation exports
pub use annotation::{
    Annotation, AnnotationLayer, AnnotationStyle, AnnotationType, ArrowStyle, ConnectorStyle,
    TextAlign, VerticalAlign,
};

// Reference line exports
pub use reference_line::{
    LabelAnchor, LineDash, ReferenceLine, ReferenceLineOrientation, ReferenceLineSet,
    ReferenceLineSetBuilder, ReferenceLineStyle,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Rgba;

    #[test]
    fn test_legend_integration() {
        let legend = Legend::new()
            .add_item("Series A", Rgba::RED)
            .add_item("Series B", Rgba::GREEN)
            .add_item("Series C", Rgba::BLUE)
            .interactive(true);

        assert_eq!(legend.len(), 3);
        assert!(legend.is_visible(0));
    }

    #[test]
    fn test_tooltip_integration() {
        let mut tooltip = TooltipWidget::default();
        tooltip.set_viewport(0.0, 0.0, 800.0, 600.0);

        let content = DataTooltipBuilder::new("January")
            .add("Revenue", "$12,345")
            .add_percent("Growth", 0.15)
            .build();

        tooltip.show_at(400.0, 300.0, content);
        assert!(tooltip.is_visible());
    }

    #[test]
    fn test_crosshair_integration() {
        let mut crosshair = Crosshair::new()
            .bounds(50.0, 50.0, 700.0, 400.0)
            .mode(CrosshairMode::Both)
            .show_labels(true);

        crosshair.update(200.0, 200.0);
        assert!(crosshair.is_active());

        let (v, h) = crosshair.get_lines();
        assert!(v.is_some());
        assert!(h.is_some());
    }

    #[test]
    fn test_annotation_integration() {
        let mut layer = AnnotationLayer::new("test");

        layer.add(Annotation::text(100.0, 50.0, "Label").with_id("label1"));
        layer.add(
            Annotation::callout(150.0, 100.0, 200.0, 50.0, "Note")
                .with_id("callout1")
                .with_arrow(true),
        );

        assert_eq!(layer.len(), 2);
        assert!(layer.find("label1").is_some());
    }

    #[test]
    fn test_reference_line_integration() {
        let lines = ReferenceLineSetBuilder::new()
            .threshold(90.0, "Max")
            .target(75.0, "Goal")
            .average(50.0, "Avg")
            .build();

        assert_eq!(lines.len(), 3);
        assert_eq!(lines.horizontal().len(), 3);
    }

    #[test]
    fn test_components_with_colors() {
        // Test that all components work with the color module
        let color = Rgba::from_hex(0x4285F4);

        let legend_item = LegendItem::new("Test", color);
        assert_eq!(legend_item.color.r, color.r);

        let crosshair_style = CrosshairStyle::solid(color, 2.0);
        assert_eq!(crosshair_style.color.r, color.r);

        let ann_style = AnnotationStyle::default().fill(color);
        assert_eq!(ann_style.fill.r, color.r);

        let ref_style = ReferenceLineStyle::solid(color, 2.0);
        assert_eq!(ref_style.color.r, color.r);
    }
}

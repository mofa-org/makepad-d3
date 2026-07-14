//! Axis components for chart rendering
//!
//! This module provides axis configuration, layout computation, and
//! number/time formatting for chart axes.
//!
//! # Features
//!
//! - **Orientation**: Bottom, Top, Left, Right axis placement
//! - **Label Rotation**: Support for diagonal and vertical labels on crowded axes
//! - **Grid Lines**: Configurable grid with styles (solid, dashed, dotted)
//! - **Minor Ticks**: Sub-division ticks between major ticks
//! - **Time Formatting**: Multi-scale time formatting for time-series charts
//! - **Discrete Scale Support**: Integration with BandScale and PointScale
//!
//! # Example
//! ```
//! use makepad_d3::scale::{LinearScale, ScaleExt};
//! use makepad_d3::axis::{Axis, AxisConfig, AxisOrientation};
//!
//! // Create a linear scale
//! let scale = LinearScale::new()
//!     .with_domain(0.0, 100.0)
//!     .with_range(50.0, 550.0);
//!
//! // Create an axis for the bottom of the chart
//! let mut axis = Axis::with_config(
//!     AxisConfig::bottom()
//!         .with_tick_size(6.0)
//!         .with_grid(300.0)
//! );
//!
//! // Update from scale
//! axis.set_scale(&scale);
//!
//! // Compute layout at y=350 (bottom of chart area)
//! let layout = axis.compute_layout(350.0);
//!
//! // Use layout for rendering
//! assert_eq!(layout.domain_start, (50.0, 350.0));
//! assert_eq!(layout.domain_end, (550.0, 350.0));
//! ```
//!
//! # Label Rotation Example
//! ```
//! use makepad_d3::axis::{AxisConfig, LabelRotation};
//!
//! // For crowded x-axis with long labels
//! let config = AxisConfig::bottom()
//!     .with_diagonal_labels()  // -45 degree rotation
//!     .with_tick_size(6.0);
//! ```

mod axis;
mod format;
mod grid;
mod tick;

// Core axis types
pub use axis::{
    Axis, AxisConfig, AxisLayout, AxisOrientation, AxisTick, LabelAlign, LabelRotation, TextAnchor,
};

// Number and time formatting
pub use format::{
    format_relative, format_si, timestamp_from_ms, timestamp_to_ms, DurationFormat,
    MultiScaleTimeFormat, NumberFormat, TimeFormat,
};

// Enhanced tick configuration
pub use tick::{generate_ticks, MinorTick, TickConfig, TickFilter, TickResult};

// Grid configuration
pub use grid::{GridConfig, GridLine, GridLineParams, GridLineStyle};

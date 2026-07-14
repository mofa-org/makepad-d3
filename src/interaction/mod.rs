//! Interaction behaviors for data visualization
//!
//! This module provides interaction primitives for adding interactivity
//! to charts and visualizations.
//!
//! # Behaviors
//!
//! - [`ZoomBehavior`]: Zoom and pan with scale constraints
//! - [`BrushBehavior`]: Rectangular selection for filtering data
//! - [`TooltipContent`]: Data structure for tooltip display
//!
//! # Example
//!
//! ```
//! use makepad_d3::interaction::{ZoomTransform, ZoomBehavior, BrushBehavior, BrushType};
//!
//! // Set up zoom behavior
//! let mut zoom = ZoomBehavior::new()
//!     .scale_extent(0.5, 4.0)
//!     .wheel_delta(0.002);
//!
//! let mut transform = ZoomTransform::identity();
//!
//! // Set up brush behavior
//! let brush = BrushBehavior::xy();
//! ```

mod brush;
mod tooltip;
mod zoom;

pub use brush::{BrushBehavior, BrushSelection, BrushType};
pub use tooltip::{TooltipContent, TooltipItem, TooltipPosition, TooltipState};
pub use zoom::{ZoomBehavior, ZoomTransform};

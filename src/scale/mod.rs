//! Scale implementations for data visualization
//!
//! Scales are functions that map from an input domain to an output range.
//! This module provides various scale types:
//!
//! - [`LinearScale`]: Linear interpolation between domain and range
//! - [`CategoryScale`]: Maps discrete categories to continuous bands
//! - [`BandScale`]: Maps discrete categories to bands with configurable padding (D3-compatible)
//! - [`PointScale`]: Maps discrete categories to evenly spaced points (zero bandwidth)
//! - [`QuantizeScale`]: Maps continuous domain to discrete range (equal-sized segments)
//! - [`QuantileScale`]: Maps continuous domain to discrete range (equal-count segments based on data)
//! - [`ThresholdScale`]: Maps continuous domain to discrete range (custom breakpoints)
//! - [`SequentialScale`]: Maps continuous domain through an interpolator (for color gradients)
//! - [`TimeScale`]: Maps DateTime values to continuous range
//! - [`LogScale`]: Logarithmic interpolation for exponential data
//! - [`PowScale`]: Power/polynomial interpolation
//! - [`SymlogScale`]: Symmetric log for data crossing zero
//!
//! # Example
//! ```
//! use makepad_d3::scale::{Scale, LinearScale, ScaleExt};
//!
//! let scale = LinearScale::new()
//!     .with_domain(0.0, 100.0)
//!     .with_range(0.0, 500.0);
//!
//! assert_eq!(scale.scale(50.0), 250.0);
//! ```

mod band;
mod category;
mod linear;
mod log;
mod point;
mod pow;
mod quantile;
mod quantize;
mod sequential;
mod symlog;
mod threshold;
mod time;
mod traits;
mod utils;

pub use band::BandScale;
pub use category::CategoryScale;
pub use linear::LinearScale;
pub use log::LogScale;
pub use point::PointScale;
pub use pow::PowScale;
pub use quantile::QuantileScale;
pub use quantize::QuantizeScale;
pub use sequential::{interpolators, SequentialScale};
pub use symlog::SymlogScale;
pub use threshold::ThresholdScale;
pub use time::{TimeInterval, TimeScale, TimeTick};
pub use traits::{ContinuousScale, DiscreteScale, Scale, ScaleExt, Tick, TickOptions};
pub use utils::{format_number, nice_bounds, nice_step};

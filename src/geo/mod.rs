//! Geographic projections and GeoJSON support
//!
//! This module provides geographic projections for mapping spherical coordinates
//! to 2D screen coordinates, along with GeoJSON parsing and path generation.
//!
//! # Projections
//!
//! - [`MercatorProjection`]: Conformal cylindrical projection (web maps)
//! - [`EquirectangularProjection`]: Simple plate carrée projection
//! - [`OrthographicProjection`]: Azimuthal projection (globe view)
//! - [`AlbersProjection`]: Equal-area conic projection (US maps)
//!
//! # GeoJSON Support
//!
//! - [`GeoJson`]: Parse and represent GeoJSON data
//! - [`Feature`]: Individual geographic features with properties
//! - [`Geometry`]: Point, LineString, Polygon, and Multi* types
//!
//! # Path Generation
//!
//! - [`GeoPath`]: Generate SVG-like paths from geographic data
//!
//! # Example
//!
//! ```
//! use makepad_d3::geo::{MercatorProjection, Projection, ProjectionBuilder, GeoPath};
//!
//! // Create a Mercator projection
//! let projection = MercatorProjection::new()
//!     .scale(100.0)
//!     .translate(400.0, 300.0);
//!
//! // Project a point (longitude, latitude) to screen coordinates
//! let (x, y) = projection.project(-122.4, 37.8); // San Francisco
//!
//! // Invert screen coordinates back to geographic
//! let (lon, lat) = projection.invert(x, y);
//! ```

mod area;
mod geojson;
mod path;
mod projection;

pub use projection::{
    compute_fit_extent, compute_fit_size, project_bounds, AlbersProjection,
    EquirectangularProjection, MercatorProjection, OrthographicProjection, Projection,
    ProjectionBuilder,
};

pub use geojson::{
    BoundingBox, Feature, FeatureCollection, GeoJson, Geometry, GeometryType, Position, Properties,
};

pub use path::{GeoPath, GeoPathSegment};

pub use area::{geo_area, geo_bounds, geo_centroid, geo_length};

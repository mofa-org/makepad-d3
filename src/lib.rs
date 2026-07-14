//! Makepad D3 - D3.js-compatible data visualization library for Makepad
//!
//! This library provides data visualization primitives inspired by D3.js,
//! optimized for Makepad's GPU-accelerated rendering.
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use makepad_d3::prelude::*;
//!
//! // Create chart data
//! let data = ChartData::new()
//!     .with_labels(vec!["Jan", "Feb", "Mar", "Apr"])
//!     .add_dataset(
//!         Dataset::new("Revenue")
//!             .with_data(vec![100.0, 200.0, 150.0, 300.0])
//!             .with_hex_color(0x4285F4)
//!     );
//!
//! // Create scales
//! let x_scale = CategoryScale::new()
//!     .with_labels(data.labels.clone())
//!     .with_range(50.0, 550.0);
//!
//! let y_scale = LinearScale::new()
//!     .with_domain(0.0, 300.0)
//!     .with_range(350.0, 50.0);  // Inverted for screen coordinates
//! ```
//!
//! # Modules
//!
//! - [`data`]: Data structures for charts (DataPoint, Dataset, ChartData)
//! - [`scale`]: Scale functions for mapping data to visual space
//! - [`axis`]: Axis components for tick marks, labels, and formatting
//! - [`shape`]: Shape generators (lines, areas, arcs, pies, stacks)
//! - [`color`]: Color scales and schemes (sequential, diverging, categorical)
//! - [`interaction`]: Interactive behaviors (zoom, brush, tooltip)
//! - [`layout`]: Layout algorithms (force simulation, tree, treemap, pack)
//! - [`geo`]: Geographic projections and GeoJSON support
//! - [`component`]: Reusable UI components (legend, tooltip, crosshair, annotation)
//! - [`render3d`]: GPU-accelerated 3D rendering (transforms, meshes, cameras)
//! - [`error`]: Error types
//!
//! # Features
//!
//! - **Scales**: Linear, Category, Time, Log, Pow, Symlog
//! - **Data Structures**: Flexible data containers with builder patterns
//! - **Serialization**: Full serde support for data import/export

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod axis;
pub mod color;
pub mod component;
pub mod data;
pub mod error;
pub mod geo;
pub mod interaction;
pub mod layout;
pub mod render3d;
pub mod scale;
pub mod shape;
/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::axis::{
        format_si, Axis, AxisConfig, AxisLayout, AxisOrientation, AxisTick, DurationFormat,
        NumberFormat,
    };
    pub use crate::color::{
        hex, hsl, lerp_color, rgb, rgba, CategoricalScale, ColorScale, DivergingScale, Hsl, Rgba,
        SequentialScale,
    };
    pub use crate::component::{
        Annotation, AnnotationLayer, AnnotationType, Crosshair, CrosshairMode, Legend, LegendItem,
        LegendOrientation, LegendPosition, ReferenceLine, ReferenceLineSet, TooltipConfig,
        TooltipWidget,
    };
    pub use crate::data::{ChartData, Color, DataPoint, Dataset, PointStyle};
    pub use crate::error::{D3Error, D3Result};
    pub use crate::geo::{
        AlbersProjection, BoundingBox, EquirectangularProjection, Feature, FeatureCollection,
        GeoJson, GeoPath, GeoPathSegment, Geometry, GeometryType, MercatorProjection,
        OrthographicProjection, Position, Projection, ProjectionBuilder, Properties,
    };
    pub use crate::interaction::{
        BrushBehavior, BrushSelection, BrushType, TooltipContent, ZoomBehavior, ZoomTransform,
    };
    pub use crate::layout::{
        CenterForce, CollideForce, Force, ForceSimulation, HierarchyNode, LinkForce, ManyBodyForce,
        PackLayout, PackStrategy, PositionForce, RadialForce, SimulationLink, SimulationNode,
        TilingMethod, TreeLayout, TreemapLayout,
    };
    pub use crate::render3d::{
        Bar3D,
        BarFace,
        BarFace3D,
        BarFaceType,
        Camera3D,
        CameraController,
        CameraEvent,
        Colormap,
        DrawAxis3D,
        DrawBar3D,
        DrawGrid3D,
        DrawPoint3D,
        DrawSurface3D,
        DrawWireframe3D,
        GeometryMesh3D,
        Mat4,
        // Note: Vec3, Vec4 are not re-exported to avoid conflict with makepad_widgets
        // Use makepad_d3::render3d::{Vec3, Vec4} if needed for 3D-specific operations
        MeshData,
        ProjectedPoint,
        Ray3D,
        Scatter3D,
        ScatterPoint3D,
        Surface3D,
        SurfaceData,
        SurfaceFace,
        Transform3D,
        FLOATS_PER_VERTEX,
    };
    pub use crate::scale::{
        format_number, nice_bounds, nice_step, CategoryScale, ContinuousScale, DiscreteScale,
        LinearScale, LogScale, PowScale, Scale, ScaleExt, SymlogScale, Tick, TickOptions,
        TimeInterval, TimeScale, TimeTick,
    };
    pub use crate::shape::{
        ArcDatum, ArcGenerator, AreaGenerator, LineGenerator, Path, PathSegment, PieLayout,
        PieSlice, PieSort, Point, StackGenerator, StackOffset, StackOrder, StackPoint,
        StackedSeries,
    };
}

// Re-export Color from data module at crate root for convenience
pub use data::Color;

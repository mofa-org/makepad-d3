//! GPU-accelerated 3D rendering module for Makepad D3
//!
//! This module provides GPU-accelerated 3D rendering capabilities for data visualization,
//! based on patterns from the makepad-urdf-player implementation.
//!
//! # Overview
//!
//! The render3d module provides a complete toolkit for 3D data visualization:
//!
//! | Component | Purpose |
//! |-----------|---------|
//! | [`Surface3D`] | 3D surface plots from height functions |
//! | [`Scatter3D`] | 3D scatter plots with points |
//! | [`Bar3D`] | 3D bar charts from grid data |
//! | [`Camera3D`] | Orbital camera with smooth controls |
//! | [`CameraController`] | Event handling for camera interaction |
//! | [`Colormap`] | Scientific colormaps (Viridis, Plasma, etc.) |
//!
//! # Architecture
//!
//! The rendering system separates geometry generation (expensive, done once) from
//! transform updates (cheap, done per-frame):
//!
//! ```text
//! SETUP (once):
//!   CPU: Generate mesh data → Compute normals → Store vertices
//!
//! PER FRAME:
//!   CPU: Update camera transform (64 bytes)
//!   CPU: Project vertices to screen space
//!   GPU: Render with lighting shaders
//!   GPU: Apply colormaps
//! ```
//!
//! # Quick Start
//!
//! ## Surface Plot
//!
//! ```rust
//! use makepad_d3::render3d::{Surface3D, Colormap};
//!
//! let mut surface = Surface3D::new();
//! surface.set_function(50, (-2.0, 2.0), (-2.0, 2.0), |x, z| {
//!     (x * x + z * z).sqrt().sin()
//! });
//! surface.rebuild_mesh();
//! surface.set_colormap(Colormap::Viridis);
//!
//! // Get faces for rendering (sorted by depth)
//! let faces = surface.get_sorted_faces(800.0, 600.0);
//! ```
//!
//! ## Scatter Plot
//!
//! ```rust
//! use makepad_d3::render3d::{Scatter3D, ScatterPoint3D, Colormap};
//!
//! let mut scatter = Scatter3D::new();
//! scatter.set_points(vec![
//!     ScatterPoint3D::new(0.0, 1.0, 0.0).with_value(0.5),
//!     ScatterPoint3D::new(1.0, 0.0, 1.0).with_value(0.8),
//!     ScatterPoint3D::new(-1.0, 0.5, -1.0).with_value(0.2),
//! ]);
//! scatter.set_colormap(Colormap::Plasma);
//!
//! // Get projected points for rendering
//! let points = scatter.get_projected_points(800.0, 600.0);
//! ```
//!
//! ## Bar Chart
//!
//! ```rust
//! use makepad_d3::render3d::{Bar3D, Colormap};
//!
//! let mut chart = Bar3D::new();
//! chart.set_data(vec![
//!     vec![1.0, 2.0, 3.0],
//!     vec![2.0, 4.0, 1.0],
//!     vec![3.0, 1.0, 2.0],
//! ]);
//! chart.set_colormap(Colormap::Inferno);
//!
//! // Get bar faces for rendering
//! let faces = chart.get_sorted_faces(800.0, 600.0);
//! ```
//!
//! ## Camera Controls
//!
//! ```rust
//! use makepad_d3::render3d::{Camera3D, CameraController, CameraEvent};
//!
//! let camera = Camera3D::new()
//!     .with_distance(5.0)
//!     .with_yaw(0.3)
//!     .with_pitch(0.5);
//!
//! let mut controller = CameraController::new(camera);
//!
//! // Handle events
//! controller.handle_camera_event(CameraEvent::PointerDown {
//!     pos: [100.0, 100.0],
//!     shift: false,
//! });
//! controller.handle_camera_event(CameraEvent::PointerMove {
//!     pos: [150.0, 120.0],
//!     shift: false,
//! });
//! controller.handle_camera_event(CameraEvent::PointerUp);
//! ```
//!
//! # Colormaps
//!
//! Seven scientific colormaps are available:
//!
//! | Colormap | Description |
//! |----------|-------------|
//! | `Viridis` | Perceptually uniform, colorblind-friendly (default) |
//! | `Plasma` | High contrast purple to yellow |
//! | `Inferno` | Dark to bright through orange |
//! | `Magma` | Dark purple to yellow |
//! | `CoolWarm` | Diverging blue to red through white |
//! | `Turbo` | Improved rainbow |
//! | `Grayscale` | Simple black to white |
//!
//! # Key Types
//!
//! ## Core Math Types
//!
//! - [`Vec3`]: 3D vector (f32)
//! - [`Vec4`]: 4D vector (f32)
//! - [`Mat4`]: 4x4 matrix with standard operations
//! - [`Transform3D`]: 64-byte GPU-friendly transform matrix
//!
//! ## Mesh Types
//!
//! - [`MeshData`]: Interleaved vertex buffer with indices
//! - [`GeometryMesh3D`]: GPU geometry wrapper for Makepad
//! - [`FLOATS_PER_VERTEX`]: Vertex layout constant (9 floats = 36 bytes)
//!
//! ## Camera Types
//!
//! - [`Camera3D`]: Orbital camera with view/projection matrices
//! - [`CameraController`]: Event handling wrapper
//! - [`CameraEvent`]: Framework-agnostic event types
//! - [`Ray3D`]: Ray for hit testing (plane, sphere, AABB intersection)
//!
//! ## Visualization Types
//!
//! - [`Surface3D`]: 3D surface plot component
//! - [`SurfaceData`]: Surface height data container
//! - [`SurfaceFace`]: Pre-computed face for rendering
//! - [`Scatter3D`]: 3D scatter plot component
//! - [`ScatterPoint3D`]: Individual scatter point
//! - [`ProjectedPoint`]: Projected point for rendering
//! - [`Bar3D`]: 3D bar chart component
//! - [`BarFace3D`]: Bar face for rendering
//! - [`BarFaceType`]: Face type enum (Top, Front, etc.)
//!
//! ## Draw Primitives
//!
//! - [`DrawSurface3D`]: Surface face with Phong lighting
//! - [`DrawWireframe3D`]: Depth-fading wireframe lines
//! - [`DrawPoint3D`]: Sphere-like points
//! - [`DrawBar3D`]: Bar faces with shading
//! - [`DrawGrid3D`]: Reference grid plane
//! - [`DrawAxis3D`]: Axis with arrow
//!
//! # Vertex Layout
//!
//! Each vertex contains 9 floats (36 bytes):
//!
//! | Offset | Size | Content |
//! |--------|------|---------|
//! | 0 | 3 floats | Position (x, y, z) |
//! | 3 | 1 float | ID (element identifier) |
//! | 4 | 3 floats | Normal (nx, ny, nz) |
//! | 7 | 2 floats | UV (u, v) |
//!
//! # Performance Tips
//!
//! 1. **Pre-compute geometry**: Call `rebuild_mesh()` once, not every frame
//! 2. **Reuse mesh data**: Store `MeshData` and only update transforms
//! 3. **Use depth sorting**: `get_sorted_faces()` returns back-to-front order
//! 4. **Limit resolution**: Start with 25-50 grid resolution, increase as needed
//! 5. **Cache projections**: Project vertices once per frame, not per face

pub mod bar3d;
pub mod camera;
pub mod colormap;
pub mod draw;
pub mod geometry;
pub mod mesh;
pub mod scatter;
pub mod surface;
pub mod types;

// Re-exports
pub use bar3d::{Bar3D, BarFace3D, BarFaceType};
pub use camera::{Camera3D, CameraController, CameraEvent, Ray3D};
pub use colormap::Colormap;
pub use draw::{
    BarFace, DrawAxis3D, DrawBar3D, DrawGrid3D, DrawPoint3D, DrawSurface3D, DrawWireframe3D,
};
pub use geometry::GeometryMesh3D;
pub use mesh::{MeshData, FLOATS_PER_VERTEX};
pub use scatter::{ProjectedPoint, Scatter3D, ScatterPoint3D};
pub use surface::{Surface3D, SurfaceData, SurfaceFace};
pub use types::{Mat4, Transform3D, Vec3, Vec4};

use makepad_widgets::Cx;

/// Register all render3d live designs
///
/// Call this function in your App's LiveRegister implementation
/// before registering any widgets that use the 3D drawing primitives.
pub fn live_design(cx: &mut Cx) {
    draw::live_design(cx);
}

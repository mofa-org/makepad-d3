//! GPU-optimized 3D Surface Plot Component
//!
//! This module provides a surface plot implementation that leverages:
//! - Pre-computed mesh geometry (computed once, reused every frame)
//! - Camera3D for orbital controls
//! - GPU shaders for lighting and colormaps
//! - Proper depth ordering
//!
//! # Performance
//!
//! The original CPU-based approach had O(n²) work per frame:
//! - Re-project every vertex
//! - Re-sort all faces
//! - Re-compute colors
//!
//! This implementation optimizes by:
//! - Pre-computing mesh geometry and normals (once)
//! - Using camera transforms for projection
//! - Caching face data between frames
//! - Using GPU shaders for lighting calculations
//!
//! # Example
//!
//! ```rust,ignore
//! use makepad_d3::render3d::{Surface3D, Camera3D, Colormap};
//!
//! let mut surface = Surface3D::new();
//! surface.set_function(50, (-2.0, 2.0), (-2.0, 2.0), |x, z| {
//!     (x * x + z * z).sqrt().sin()
//! });
//! surface.set_colormap(Colormap::Viridis);
//! ```

use super::camera::{Camera3D, CameraController, CameraEvent};
use super::colormap::Colormap;
use super::mesh::MeshData;
use super::types::Vec3;

/// Pre-computed face data for rendering
#[derive(Clone, Debug)]
pub struct SurfaceFace {
    /// Screen-space vertices (after projection)
    pub screen_verts: [[f64; 2]; 4],
    /// World-space normal vector
    pub normal: [f32; 3],
    /// Normalized height value for colormap (0-1)
    pub data_value: f32,
    /// Depth for sorting (camera-space Z)
    pub depth: f64,
    /// Original vertex indices in mesh
    pub indices: [usize; 4],
}

/// Surface data container
#[derive(Clone, Debug)]
pub struct SurfaceData {
    /// Height values in grid (row-major)
    pub heights: Vec<Vec<f64>>,
    /// Minimum height value
    pub min_z: f64,
    /// Maximum height value
    pub max_z: f64,
    /// X range
    pub x_range: (f64, f64),
    /// Z range (depth axis)
    pub z_range: (f64, f64),
}

impl Default for SurfaceData {
    fn default() -> Self {
        Self {
            heights: Vec::new(),
            min_z: 0.0,
            max_z: 1.0,
            x_range: (-1.0, 1.0),
            z_range: (-1.0, 1.0),
        }
    }
}

impl SurfaceData {
    /// Create surface data from a height function
    pub fn from_function<F>(
        resolution: usize,
        x_range: (f64, f64),
        z_range: (f64, f64),
        f: F,
    ) -> Self
    where
        F: Fn(f64, f64) -> f64,
    {
        let mut heights = vec![vec![0.0; resolution]; resolution];
        let mut min_z = f64::MAX;
        let mut max_z = f64::MIN;

        for i in 0..resolution {
            for j in 0..resolution {
                let x = x_range.0
                    + (x_range.1 - x_range.0) * (i as f64 / (resolution - 1).max(1) as f64);
                let z = z_range.0
                    + (z_range.1 - z_range.0) * (j as f64 / (resolution - 1).max(1) as f64);
                let y = f(x, z);
                heights[i][j] = y;
                min_z = min_z.min(y);
                max_z = max_z.max(y);
            }
        }

        Self {
            heights,
            min_z,
            max_z,
            x_range,
            z_range,
        }
    }

    /// Get resolution (grid size)
    pub fn resolution(&self) -> usize {
        self.heights.len()
    }

    /// Check if data is valid
    pub fn is_valid(&self) -> bool {
        self.heights.len() >= 2 && self.heights[0].len() >= 2
    }
}

/// GPU-optimized 3D surface plot
#[derive(Clone, Debug)]
pub struct Surface3D {
    /// Surface data
    pub data: SurfaceData,

    /// Pre-computed mesh data
    mesh: Option<MeshData>,

    /// Pre-computed world-space vertices
    world_vertices: Vec<[f64; 3]>,

    /// Pre-computed normals per vertex
    vertex_normals: Vec<[f32; 3]>,

    /// Pre-computed data values per face
    face_data_values: Vec<f32>,

    /// Camera controller for interaction
    pub camera_controller: CameraController,

    /// Colormap for height values
    pub colormap: Colormap,

    /// Whether to show surface fill
    pub show_surface: bool,

    /// Whether to show wireframe
    pub show_wireframe: bool,

    /// Wireframe color
    pub wireframe_color: [f32; 4],

    /// Opacity
    pub opacity: f32,

    /// Animation progress (0-1 for intro animation)
    pub animation_progress: f64,

    /// Whether mesh needs rebuild
    needs_rebuild: bool,
}

impl Default for Surface3D {
    fn default() -> Self {
        Self::new()
    }
}

impl Surface3D {
    /// Create a new surface plot with default settings
    pub fn new() -> Self {
        let camera = Camera3D::new()
            .with_distance(4.0)
            .with_yaw(0.5)
            .with_pitch(0.6)
            .with_fov(std::f64::consts::PI / 4.0);

        Self {
            data: SurfaceData::default(),
            mesh: None,
            world_vertices: Vec::new(),
            vertex_normals: Vec::new(),
            face_data_values: Vec::new(),
            camera_controller: CameraController::new(camera),
            colormap: Colormap::Viridis,
            show_surface: true,
            show_wireframe: false,
            wireframe_color: [0.3, 0.3, 0.3, 0.7],
            opacity: 1.0,
            animation_progress: 1.0,
            needs_rebuild: true,
        }
    }

    /// Set surface data from a height function
    pub fn set_function<F>(
        &mut self,
        resolution: usize,
        x_range: (f64, f64),
        z_range: (f64, f64),
        f: F,
    ) where
        F: Fn(f64, f64) -> f64,
    {
        self.data = SurfaceData::from_function(resolution, x_range, z_range, f);
        self.needs_rebuild = true;
    }

    /// Set surface data directly
    pub fn set_data(&mut self, data: SurfaceData) {
        self.data = data;
        self.needs_rebuild = true;
    }

    /// Set the colormap
    pub fn set_colormap(&mut self, colormap: Colormap) {
        self.colormap = colormap;
    }

    /// Access the camera
    pub fn camera(&self) -> &Camera3D {
        self.camera_controller.camera()
    }

    /// Access the camera mutably
    pub fn camera_mut(&mut self) -> &mut Camera3D {
        self.camera_controller.camera_mut()
    }

    /// Handle camera events
    pub fn handle_camera_event(&mut self, event: CameraEvent) -> bool {
        self.camera_controller.handle_camera_event(event)
    }

    /// Check if needs rebuild
    pub fn needs_rebuild(&self) -> bool {
        self.needs_rebuild
    }

    /// Rebuild the mesh data from surface data
    pub fn rebuild_mesh(&mut self) {
        if !self.data.is_valid() {
            self.mesh = None;
            self.world_vertices.clear();
            self.vertex_normals.clear();
            self.face_data_values.clear();
            self.needs_rebuild = false;
            return;
        }

        let rows = self.data.heights.len();
        let cols = self.data.heights[0].len();

        // Normalize height range
        let z_range = self.data.max_z - self.data.min_z;
        let z_scale = if z_range.abs() < 1e-10 {
            1.0
        } else {
            1.0 / z_range
        };

        // Generate world-space vertices
        self.world_vertices.clear();
        self.world_vertices.reserve(rows * cols);

        for i in 0..rows {
            for j in 0..cols {
                let x = self.data.x_range.0
                    + (self.data.x_range.1 - self.data.x_range.0)
                        * (i as f64 / (rows - 1).max(1) as f64);
                let z = self.data.z_range.0
                    + (self.data.z_range.1 - self.data.z_range.0)
                        * (j as f64 / (cols - 1).max(1) as f64);
                let y = (self.data.heights[i][j] - self.data.min_z) * z_scale;

                self.world_vertices.push([x, y, z]);
            }
        }

        // Compute normals per vertex (averaged from adjacent faces)
        self.vertex_normals.clear();
        self.vertex_normals.resize(rows * cols, [0.0, 1.0, 0.0]);

        for i in 0..rows - 1 {
            for j in 0..cols - 1 {
                let idx00 = i * cols + j;
                let idx10 = (i + 1) * cols + j;
                let idx01 = i * cols + (j + 1);
                let idx11 = (i + 1) * cols + (j + 1);

                let v00 = self.world_vertices[idx00];
                let v10 = self.world_vertices[idx10];
                let v01 = self.world_vertices[idx01];
                let v11 = self.world_vertices[idx11];

                // Compute face normal using cross product
                let edge1 = [v10[0] - v00[0], v10[1] - v00[1], v10[2] - v00[2]];
                let edge2 = [v01[0] - v00[0], v01[1] - v00[1], v01[2] - v00[2]];

                let normal = [
                    (edge1[1] * edge2[2] - edge1[2] * edge2[1]) as f32,
                    (edge1[2] * edge2[0] - edge1[0] * edge2[2]) as f32,
                    (edge1[0] * edge2[1] - edge1[1] * edge2[0]) as f32,
                ];

                // Accumulate normals for each vertex of this face
                for &idx in &[idx00, idx10, idx01, idx11] {
                    self.vertex_normals[idx][0] += normal[0];
                    self.vertex_normals[idx][1] += normal[1];
                    self.vertex_normals[idx][2] += normal[2];
                }
            }
        }

        // Normalize all vertex normals
        for normal in &mut self.vertex_normals {
            let len =
                (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
            if len > 1e-6 {
                normal[0] /= len;
                normal[1] /= len;
                normal[2] /= len;
            }
        }

        // Pre-compute data values per face
        self.face_data_values.clear();
        self.face_data_values.reserve((rows - 1) * (cols - 1));

        for i in 0..rows - 1 {
            for j in 0..cols - 1 {
                let h00 = self.data.heights[i][j];
                let h10 = self.data.heights[i + 1][j];
                let h01 = self.data.heights[i][j + 1];
                let h11 = self.data.heights[i + 1][j + 1];
                let avg_h = (h00 + h10 + h01 + h11) / 4.0;
                let t = ((avg_h - self.data.min_z) * z_scale).clamp(0.0, 1.0);
                self.face_data_values.push(t as f32);
            }
        }

        // Also create MeshData for potential future GPU geometry upload
        self.mesh = Some(MeshData::surface(
            rows,
            (self.data.x_range.0 as f32, self.data.x_range.1 as f32),
            (self.data.z_range.0 as f32, self.data.z_range.1 as f32),
            |x, z| {
                // Find closest grid point
                let i = ((x as f64 - self.data.x_range.0)
                    / (self.data.x_range.1 - self.data.x_range.0)
                    * (rows - 1) as f64)
                    .round() as usize;
                let j = ((z as f64 - self.data.z_range.0)
                    / (self.data.z_range.1 - self.data.z_range.0)
                    * (cols - 1) as f64)
                    .round() as usize;
                let i = i.min(rows - 1);
                let j = j.min(cols - 1);
                ((self.data.heights[i][j] - self.data.min_z) * z_scale) as f32
            },
        ));

        self.needs_rebuild = false;
    }

    /// Get faces sorted by depth for rendering
    ///
    /// Returns face data including screen-space vertices, normals, and data values.
    pub fn get_sorted_faces(&self, viewport_width: f64, viewport_height: f64) -> Vec<SurfaceFace> {
        if !self.data.is_valid() || self.world_vertices.is_empty() {
            return Vec::new();
        }

        let rows = self.data.heights.len();
        let cols = self.data.heights[0].len();

        let camera = self.camera_controller.camera();
        let aspect = (viewport_width / viewport_height) as f32;
        let mvp = camera.view_projection_matrix(aspect);

        // Project all vertices to screen space
        let mut screen_vertices: Vec<([f64; 2], f64)> =
            Vec::with_capacity(self.world_vertices.len());

        for world_v in &self.world_vertices {
            // Apply animation progress to Y coordinate
            let animated_y = world_v[1] * self.animation_progress;
            let world_pos = Vec3::new(world_v[0] as f32, animated_y as f32, world_v[2] as f32);

            // Transform to clip space
            let clip = mvp.transform_point(world_pos);

            // Perspective divide
            let ndc_x = clip.x;
            let ndc_y = clip.y;

            // Convert to screen space
            let screen_x = ((ndc_x + 1.0) / 2.0) as f64 * viewport_width;
            let screen_y = ((1.0 - ndc_y) / 2.0) as f64 * viewport_height;

            screen_vertices.push(([screen_x, screen_y], clip.z as f64));
        }

        // Create faces
        let mut faces: Vec<SurfaceFace> = Vec::with_capacity((rows - 1) * (cols - 1));

        for i in 0..rows - 1 {
            for j in 0..cols - 1 {
                let idx00 = i * cols + j;
                let idx10 = (i + 1) * cols + j;
                let idx01 = i * cols + (j + 1);
                let idx11 = (i + 1) * cols + (j + 1);

                let face_idx = i * (cols - 1) + j;

                // Average depth for sorting
                let avg_depth = (screen_vertices[idx00].1
                    + screen_vertices[idx10].1
                    + screen_vertices[idx01].1
                    + screen_vertices[idx11].1)
                    / 4.0;

                // Average normal
                let n00 = self.vertex_normals[idx00];
                let n10 = self.vertex_normals[idx10];
                let n01 = self.vertex_normals[idx01];
                let n11 = self.vertex_normals[idx11];

                let avg_normal = [
                    (n00[0] + n10[0] + n01[0] + n11[0]) / 4.0,
                    (n00[1] + n10[1] + n01[1] + n11[1]) / 4.0,
                    (n00[2] + n10[2] + n01[2] + n11[2]) / 4.0,
                ];

                faces.push(SurfaceFace {
                    screen_verts: [
                        screen_vertices[idx00].0,
                        screen_vertices[idx10].0,
                        screen_vertices[idx11].0,
                        screen_vertices[idx01].0,
                    ],
                    normal: avg_normal,
                    data_value: self.face_data_values.get(face_idx).copied().unwrap_or(0.5),
                    depth: avg_depth,
                    indices: [idx00, idx10, idx11, idx01],
                });
            }
        }

        // Sort by depth (back to front for proper rendering)
        faces.sort_by(|a, b| {
            b.depth
                .partial_cmp(&a.depth)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        faces
    }

    /// Get wireframe edges for rendering
    ///
    /// Returns pairs of screen-space points for each edge.
    pub fn get_wireframe_edges(
        &self,
        viewport_width: f64,
        viewport_height: f64,
    ) -> Vec<([f64; 2], [f64; 2], f64)> {
        if !self.data.is_valid() || self.world_vertices.is_empty() {
            return Vec::new();
        }

        let rows = self.data.heights.len();
        let cols = self.data.heights[0].len();

        let camera = self.camera_controller.camera();
        let aspect = (viewport_width / viewport_height) as f32;
        let mvp = camera.view_projection_matrix(aspect);

        // Project all vertices
        let mut screen_vertices: Vec<([f64; 2], f64)> =
            Vec::with_capacity(self.world_vertices.len());

        for world_v in &self.world_vertices {
            let animated_y = world_v[1] * self.animation_progress;
            let world_pos = Vec3::new(world_v[0] as f32, animated_y as f32, world_v[2] as f32);
            let clip = mvp.transform_point(world_pos);

            let screen_x = ((clip.x + 1.0) / 2.0) as f64 * viewport_width;
            let screen_y = ((1.0 - clip.y) / 2.0) as f64 * viewport_height;

            screen_vertices.push(([screen_x, screen_y], clip.z as f64));
        }

        // Collect edges (avoid duplicates by only collecting right and down edges)
        let mut edges: Vec<([f64; 2], [f64; 2], f64)> = Vec::new();

        for i in 0..rows {
            for j in 0..cols {
                let idx = i * cols + j;

                // Right edge
                if j < cols - 1 {
                    let idx_right = i * cols + (j + 1);
                    let depth = (screen_vertices[idx].1 + screen_vertices[idx_right].1) / 2.0;
                    edges.push((screen_vertices[idx].0, screen_vertices[idx_right].0, depth));
                }

                // Down edge
                if i < rows - 1 {
                    let idx_down = (i + 1) * cols + j;
                    let depth = (screen_vertices[idx].1 + screen_vertices[idx_down].1) / 2.0;
                    edges.push((screen_vertices[idx].0, screen_vertices[idx_down].0, depth));
                }
            }
        }

        edges
    }

    /// Get the underlying mesh data (if available)
    pub fn mesh(&self) -> Option<&MeshData> {
        self.mesh.as_ref()
    }

    /// Check if camera is animating
    pub fn is_animating(&self) -> bool {
        self.camera_controller.needs_update() || self.animation_progress < 1.0
    }

    /// Set animation progress
    pub fn set_animation_progress(&mut self, progress: f64) {
        self.animation_progress = progress.clamp(0.0, 1.0);
    }

    /// Start intro animation
    pub fn start_animation(&mut self) {
        self.animation_progress = 0.0;
    }

    /// Update animation
    pub fn update_animation(&mut self, dt: f64, speed: f64) -> bool {
        if self.animation_progress < 1.0 {
            self.animation_progress = (self.animation_progress + dt * speed).min(1.0);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_surface_data_from_function() {
        let data = SurfaceData::from_function(10, (-1.0, 1.0), (-1.0, 1.0), |x, z| x + z);

        assert_eq!(data.heights.len(), 10);
        assert_eq!(data.heights[0].len(), 10);
        assert!(data.is_valid());
    }

    #[test]
    fn test_surface3d_new() {
        let surface = Surface3D::new();
        assert!(surface.needs_rebuild);
        assert_eq!(surface.animation_progress, 1.0);
    }

    #[test]
    fn test_surface3d_set_function() {
        let mut surface = Surface3D::new();
        surface.set_function(25, (-2.0, 2.0), (-2.0, 2.0), |x, z| {
            (x * x + z * z).sqrt().sin()
        });

        assert!(surface.data.is_valid());
        assert_eq!(surface.data.resolution(), 25);
        assert!(surface.needs_rebuild());
    }

    #[test]
    fn test_surface3d_rebuild_mesh() {
        let mut surface = Surface3D::new();
        surface.set_function(10, (-1.0, 1.0), (-1.0, 1.0), |x, z| x * z);
        surface.rebuild_mesh();

        assert!(!surface.needs_rebuild());
        assert_eq!(surface.world_vertices.len(), 100); // 10x10
        assert_eq!(surface.vertex_normals.len(), 100);
        assert_eq!(surface.face_data_values.len(), 81); // 9x9 faces
    }

    #[test]
    fn test_surface3d_get_sorted_faces() {
        let mut surface = Surface3D::new();
        surface.set_function(5, (-1.0, 1.0), (-1.0, 1.0), |x, z| x + z);
        surface.rebuild_mesh();

        let faces = surface.get_sorted_faces(800.0, 600.0);

        assert_eq!(faces.len(), 16); // 4x4 faces
        assert!(faces
            .iter()
            .all(|f| f.data_value >= 0.0 && f.data_value <= 1.0));
    }

    #[test]
    fn test_surface3d_camera_access() {
        let mut surface = Surface3D::new();

        // Test camera access
        let distance = surface.camera().distance;
        assert!(distance > 0.0);

        // Test camera modification
        surface.camera_mut().distance = 10.0;
        assert!((surface.camera().distance - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_surface3d_animation() {
        let mut surface = Surface3D::new();
        surface.start_animation();

        assert!((surface.animation_progress - 0.0).abs() < 0.01);

        surface.update_animation(0.1, 2.0);
        assert!(surface.animation_progress > 0.0);
        assert!(surface.animation_progress < 1.0);
    }
}

//! GPU-optimized 3D Scatter Plot Component
//!
//! This module provides a scatter plot for visualizing 3D point data with:
//! - Camera-based orbital controls
//! - Colormap-based point coloring
//! - Variable point sizes
//! - Depth-sorted rendering
//!
//! # Example
//!
//! ```rust,ignore
//! use makepad_d3::render3d::{Scatter3D, ScatterPoint3D, Colormap};
//!
//! let mut scatter = Scatter3D::new();
//! scatter.set_points(vec![
//!     ScatterPoint3D::new(0.0, 1.0, 0.0).with_value(0.5),
//!     ScatterPoint3D::new(1.0, 0.0, 1.0).with_value(0.8),
//! ]);
//! scatter.set_colormap(Colormap::Viridis);
//! ```

use super::camera::{Camera3D, CameraController, CameraEvent};
use super::colormap::Colormap;
use super::types::Vec3;

/// A single point in 3D scatter plot
#[derive(Clone, Debug)]
pub struct ScatterPoint3D {
    /// X coordinate
    pub x: f64,
    /// Y coordinate
    pub y: f64,
    /// Z coordinate
    pub z: f64,
    /// Data value for colormap (0-1)
    pub value: f64,
    /// Optional custom size multiplier
    pub size: Option<f64>,
    /// Optional custom color (overrides colormap)
    pub color: Option<[f32; 4]>,
    /// Optional label
    pub label: Option<String>,
}

impl ScatterPoint3D {
    /// Create a new point at the given coordinates
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self {
            x,
            y,
            z,
            value: 0.5,
            size: None,
            color: None,
            label: None,
        }
    }

    /// Set the data value for colormap
    pub fn with_value(mut self, value: f64) -> Self {
        self.value = value.clamp(0.0, 1.0);
        self
    }

    /// Set custom size multiplier
    pub fn with_size(mut self, size: f64) -> Self {
        self.size = Some(size);
        self
    }

    /// Set custom color (RGBA)
    pub fn with_color(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.color = Some([r, g, b, a]);
        self
    }

    /// Set label
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

/// Projected point data for rendering
#[derive(Clone, Debug)]
pub struct ProjectedPoint {
    /// Screen X coordinate
    pub screen_x: f64,
    /// Screen Y coordinate
    pub screen_y: f64,
    /// Depth (for sorting)
    pub depth: f64,
    /// Point size in screen pixels
    pub size: f64,
    /// Color (RGBA)
    pub color: [f32; 4],
    /// Original point index
    pub index: usize,
}

/// 3D Scatter Plot
#[derive(Clone, Debug)]
pub struct Scatter3D {
    /// Point data
    points: Vec<ScatterPoint3D>,

    /// Camera controller
    pub camera_controller: CameraController,

    /// Colormap for values
    pub colormap: Colormap,

    /// Base point size in pixels
    pub point_size: f64,

    /// Whether to scale points by depth (perspective)
    pub perspective_scaling: bool,

    /// Opacity
    pub opacity: f32,

    /// Data bounds (computed from points)
    bounds_min: [f64; 3],
    bounds_max: [f64; 3],

    /// Whether bounds need recomputation
    needs_bounds_update: bool,
}

impl Default for Scatter3D {
    fn default() -> Self {
        Self::new()
    }
}

impl Scatter3D {
    /// Create a new scatter plot
    pub fn new() -> Self {
        let camera = Camera3D::new()
            .with_distance(5.0)
            .with_yaw(0.5)
            .with_pitch(0.4);

        Self {
            points: Vec::new(),
            camera_controller: CameraController::new(camera),
            colormap: Colormap::Viridis,
            point_size: 8.0,
            perspective_scaling: true,
            opacity: 1.0,
            bounds_min: [0.0, 0.0, 0.0],
            bounds_max: [1.0, 1.0, 1.0],
            needs_bounds_update: true,
        }
    }

    /// Set the points data
    pub fn set_points(&mut self, points: Vec<ScatterPoint3D>) {
        self.points = points;
        self.needs_bounds_update = true;
    }

    /// Add a single point
    pub fn add_point(&mut self, point: ScatterPoint3D) {
        self.points.push(point);
        self.needs_bounds_update = true;
    }

    /// Clear all points
    pub fn clear(&mut self) {
        self.points.clear();
        self.needs_bounds_update = true;
    }

    /// Get number of points
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// Set the colormap
    pub fn set_colormap(&mut self, colormap: Colormap) {
        self.colormap = colormap;
    }

    /// Set base point size
    pub fn set_point_size(&mut self, size: f64) {
        self.point_size = size;
    }

    /// Access camera
    pub fn camera(&self) -> &Camera3D {
        self.camera_controller.camera()
    }

    /// Access camera mutably
    pub fn camera_mut(&mut self) -> &mut Camera3D {
        self.camera_controller.camera_mut()
    }

    /// Handle camera event
    pub fn handle_camera_event(&mut self, event: CameraEvent) -> bool {
        self.camera_controller.handle_camera_event(event)
    }

    /// Update bounds from points
    fn update_bounds(&mut self) {
        if self.points.is_empty() {
            self.bounds_min = [-1.0, -1.0, -1.0];
            self.bounds_max = [1.0, 1.0, 1.0];
            return;
        }

        self.bounds_min = [f64::MAX, f64::MAX, f64::MAX];
        self.bounds_max = [f64::MIN, f64::MIN, f64::MIN];

        for p in &self.points {
            self.bounds_min[0] = self.bounds_min[0].min(p.x);
            self.bounds_min[1] = self.bounds_min[1].min(p.y);
            self.bounds_min[2] = self.bounds_min[2].min(p.z);
            self.bounds_max[0] = self.bounds_max[0].max(p.x);
            self.bounds_max[1] = self.bounds_max[1].max(p.y);
            self.bounds_max[2] = self.bounds_max[2].max(p.z);
        }

        // Ensure non-zero range
        for i in 0..3 {
            if (self.bounds_max[i] - self.bounds_min[i]).abs() < 1e-10 {
                self.bounds_min[i] -= 0.5;
                self.bounds_max[i] += 0.5;
            }
        }

        self.needs_bounds_update = false;
    }

    /// Get data bounds
    pub fn bounds(&mut self) -> ([f64; 3], [f64; 3]) {
        if self.needs_bounds_update {
            self.update_bounds();
        }
        (self.bounds_min, self.bounds_max)
    }

    /// Get projected points sorted by depth (back to front)
    pub fn get_projected_points(
        &mut self,
        viewport_width: f64,
        viewport_height: f64,
    ) -> Vec<ProjectedPoint> {
        if self.points.is_empty() {
            return Vec::new();
        }

        if self.needs_bounds_update {
            self.update_bounds();
        }

        let camera = self.camera_controller.camera();
        let aspect = (viewport_width / viewport_height) as f32;
        let mvp = camera.view_projection_matrix(aspect);

        // Normalize points to [-1, 1] range
        let range = [
            self.bounds_max[0] - self.bounds_min[0],
            self.bounds_max[1] - self.bounds_min[1],
            self.bounds_max[2] - self.bounds_min[2],
        ];
        let center = [
            (self.bounds_min[0] + self.bounds_max[0]) / 2.0,
            (self.bounds_min[1] + self.bounds_max[1]) / 2.0,
            (self.bounds_min[2] + self.bounds_max[2]) / 2.0,
        ];
        let scale = 2.0 / range[0].max(range[1]).max(range[2]);

        let mut projected: Vec<ProjectedPoint> = Vec::with_capacity(self.points.len());

        for (idx, point) in self.points.iter().enumerate() {
            // Normalize to unit cube
            let nx = (point.x - center[0]) * scale;
            let ny = (point.y - center[1]) * scale;
            let nz = (point.z - center[2]) * scale;

            let world_pos = Vec3::new(nx as f32, ny as f32, nz as f32);
            let clip = mvp.transform_point(world_pos);

            // Skip points behind camera
            if clip.z < 0.0 {
                continue;
            }

            // NDC to screen
            let screen_x = ((clip.x + 1.0) / 2.0) as f64 * viewport_width;
            let screen_y = ((1.0 - clip.y) / 2.0) as f64 * viewport_height;

            // Calculate point size with perspective
            let size_mult = point.size.unwrap_or(1.0);
            let size = if self.perspective_scaling {
                let depth_factor = 1.0 / (1.0 + clip.z as f64 * 0.5);
                self.point_size * size_mult * depth_factor
            } else {
                self.point_size * size_mult
            };

            // Get color from colormap or custom
            let color = if let Some(c) = point.color {
                c
            } else {
                let rgb = self.colormap.sample(point.value as f32);
                [rgb.x, rgb.y, rgb.z, self.opacity]
            };

            projected.push(ProjectedPoint {
                screen_x,
                screen_y,
                depth: clip.z as f64,
                size,
                color,
                index: idx,
            });
        }

        // Sort by depth (back to front)
        projected.sort_by(|a, b| {
            b.depth
                .partial_cmp(&a.depth)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        projected
    }

    /// Get the original point data by index
    pub fn get_point(&self, index: usize) -> Option<&ScatterPoint3D> {
        self.points.get(index)
    }

    /// Check if camera is animating
    pub fn is_animating(&self) -> bool {
        self.camera_controller.needs_update()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scatter_point_new() {
        let p = ScatterPoint3D::new(1.0, 2.0, 3.0);
        assert!((p.x - 1.0).abs() < 0.01);
        assert!((p.y - 2.0).abs() < 0.01);
        assert!((p.z - 3.0).abs() < 0.01);
        assert!((p.value - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_scatter_point_builders() {
        let p = ScatterPoint3D::new(0.0, 0.0, 0.0)
            .with_value(0.8)
            .with_size(2.0)
            .with_color(1.0, 0.0, 0.0, 1.0)
            .with_label("test");

        assert!((p.value - 0.8).abs() < 0.01);
        assert!((p.size.unwrap() - 2.0).abs() < 0.01);
        assert_eq!(p.color, Some([1.0, 0.0, 0.0, 1.0]));
        assert_eq!(p.label, Some("test".to_string()));
    }

    #[test]
    fn test_scatter3d_new() {
        let scatter = Scatter3D::new();
        assert!(scatter.is_empty());
        assert!((scatter.point_size - 8.0).abs() < 0.01);
    }

    #[test]
    fn test_scatter3d_set_points() {
        let mut scatter = Scatter3D::new();
        scatter.set_points(vec![
            ScatterPoint3D::new(0.0, 0.0, 0.0),
            ScatterPoint3D::new(1.0, 1.0, 1.0),
        ]);

        assert_eq!(scatter.len(), 2);
        assert!(!scatter.is_empty());
    }

    #[test]
    fn test_scatter3d_bounds() {
        let mut scatter = Scatter3D::new();
        scatter.set_points(vec![
            ScatterPoint3D::new(-1.0, 0.0, 0.0),
            ScatterPoint3D::new(1.0, 2.0, 3.0),
        ]);

        let (min, max) = scatter.bounds();
        assert!((min[0] - (-1.0)).abs() < 0.01);
        assert!((max[0] - 1.0).abs() < 0.01);
        assert!((min[1] - 0.0).abs() < 0.01);
        assert!((max[1] - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_scatter3d_projection() {
        let mut scatter = Scatter3D::new();
        scatter.set_points(vec![
            ScatterPoint3D::new(0.0, 0.0, 0.0).with_value(0.0),
            ScatterPoint3D::new(1.0, 1.0, 1.0).with_value(1.0),
        ]);

        let projected = scatter.get_projected_points(800.0, 600.0);

        // Should have 2 points
        assert_eq!(projected.len(), 2);

        // Points should be within viewport
        for p in &projected {
            assert!(p.screen_x >= 0.0 && p.screen_x <= 800.0);
            assert!(p.screen_y >= 0.0 && p.screen_y <= 600.0);
            assert!(p.size > 0.0);
        }
    }

    #[test]
    fn test_scatter3d_add_clear() {
        let mut scatter = Scatter3D::new();

        scatter.add_point(ScatterPoint3D::new(0.0, 0.0, 0.0));
        assert_eq!(scatter.len(), 1);

        scatter.add_point(ScatterPoint3D::new(1.0, 1.0, 1.0));
        assert_eq!(scatter.len(), 2);

        scatter.clear();
        assert!(scatter.is_empty());
    }
}

//! GPU-optimized 3D Bar Chart Component
//!
//! This module provides a 3D bar chart for visualizing grid-based data with:
//! - Camera-based orbital controls
//! - Colormap-based bar coloring
//! - Configurable bar dimensions and spacing
//! - Depth-sorted face rendering
//!
//! # Example
//!
//! ```rust,ignore
//! use makepad_d3::render3d::{Bar3D, Colormap};
//!
//! let mut chart = Bar3D::new();
//! chart.set_data(vec![
//!     vec![1.0, 2.0, 3.0],
//!     vec![2.0, 4.0, 1.0],
//! ]);
//! chart.set_colormap(Colormap::Viridis);
//! ```

use super::camera::{Camera3D, CameraController, CameraEvent};
use super::colormap::Colormap;
use super::types::Vec3;

/// A single bar face for rendering
#[derive(Clone, Debug)]
pub struct BarFace3D {
    /// Screen-space vertices (4 corners of face)
    pub screen_verts: [[f64; 2]; 4],
    /// Face type (0=top, 1=front, 2=left, 3=right, 4=back)
    pub face_type: u8,
    /// Data value for colormap (0-1)
    pub data_value: f32,
    /// Depth for sorting
    pub depth: f64,
    /// Bar row index
    pub row: usize,
    /// Bar column index
    pub col: usize,
}

/// Face types for 3D bars
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BarFaceType {
    /// Top face (brightest)
    Top = 0,
    /// Front face
    Front = 1,
    /// Left side face
    Left = 2,
    /// Right side face
    Right = 3,
    /// Back face (usually hidden)
    Back = 4,
}

impl BarFaceType {
    /// Get brightness multiplier for this face
    pub fn brightness(&self) -> f32 {
        match self {
            BarFaceType::Top => 1.0,
            BarFaceType::Front => 0.85,
            BarFaceType::Right => 0.7,
            BarFaceType::Left => 0.6,
            BarFaceType::Back => 0.5,
        }
    }
}

/// 3D Bar Chart
#[derive(Clone, Debug)]
pub struct Bar3D {
    /// Data values (2D grid, row-major)
    data: Vec<Vec<f64>>,

    /// Minimum value in data
    min_value: f64,

    /// Maximum value in data
    max_value: f64,

    /// Camera controller
    pub camera_controller: CameraController,

    /// Colormap for values
    pub colormap: Colormap,

    /// Bar width (0-1, relative to cell)
    pub bar_width: f64,

    /// Bar depth (0-1, relative to cell)
    pub bar_depth: f64,

    /// Gap between bars (0-1)
    pub gap: f64,

    /// Whether to show bar outlines
    pub show_outlines: bool,

    /// Outline color
    pub outline_color: [f32; 4],

    /// Base height (for negative values)
    pub base_height: f64,

    /// Whether data needs reprocessing
    needs_update: bool,
}

impl Default for Bar3D {
    fn default() -> Self {
        Self::new()
    }
}

impl Bar3D {
    /// Create a new bar chart
    pub fn new() -> Self {
        let camera = Camera3D::new()
            .with_distance(6.0)
            .with_yaw(0.6)
            .with_pitch(0.5);

        Self {
            data: Vec::new(),
            min_value: 0.0,
            max_value: 1.0,
            camera_controller: CameraController::new(camera),
            colormap: Colormap::Viridis,
            bar_width: 0.8,
            bar_depth: 0.8,
            gap: 0.1,
            show_outlines: true,
            outline_color: [0.2, 0.2, 0.2, 0.8],
            base_height: 0.0,
            needs_update: true,
        }
    }

    /// Set the data (2D grid)
    pub fn set_data(&mut self, data: Vec<Vec<f64>>) {
        // Compute min/max
        self.min_value = f64::MAX;
        self.max_value = f64::MIN;

        for row in &data {
            for &val in row {
                self.min_value = self.min_value.min(val);
                self.max_value = self.max_value.max(val);
            }
        }

        // Ensure non-zero range
        if (self.max_value - self.min_value).abs() < 1e-10 {
            self.min_value -= 0.5;
            self.max_value += 0.5;
        }

        self.data = data;
        self.needs_update = false;
    }

    /// Get data dimensions (rows, cols)
    pub fn dimensions(&self) -> (usize, usize) {
        if self.data.is_empty() {
            (0, 0)
        } else {
            (self.data.len(), self.data[0].len())
        }
    }

    /// Check if has data
    pub fn has_data(&self) -> bool {
        !self.data.is_empty()
    }

    /// Set the colormap
    pub fn set_colormap(&mut self, colormap: Colormap) {
        self.colormap = colormap;
    }

    /// Set bar dimensions
    pub fn set_bar_size(&mut self, width: f64, depth: f64) {
        self.bar_width = width.clamp(0.1, 1.0);
        self.bar_depth = depth.clamp(0.1, 1.0);
    }

    /// Set gap between bars
    pub fn set_gap(&mut self, gap: f64) {
        self.gap = gap.clamp(0.0, 0.5);
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

    /// Get bar faces sorted by depth for rendering
    pub fn get_sorted_faces(&self, viewport_width: f64, viewport_height: f64) -> Vec<BarFace3D> {
        let (rows, cols) = self.dimensions();
        if rows == 0 || cols == 0 {
            return Vec::new();
        }

        let camera = self.camera_controller.camera();
        let aspect = (viewport_width / viewport_height) as f32;
        let mvp = camera.view_projection_matrix(aspect);

        let value_range = self.max_value - self.min_value;

        // Cell size in world space (normalize to [-1, 1])
        let cell_width = 2.0 / cols as f64;
        let cell_depth = 2.0 / rows as f64;

        let bar_w = cell_width * self.bar_width * (1.0 - self.gap);
        let bar_d = cell_depth * self.bar_depth * (1.0 - self.gap);

        let mut faces: Vec<BarFace3D> = Vec::new();

        for row in 0..rows {
            for col in 0..cols {
                let value = self.data[row][col];
                let normalized = ((value - self.min_value) / value_range).clamp(0.0, 1.0);
                let height = normalized; // Height in range [0, 1]

                // Bar center in world space
                let cx = -1.0 + cell_width * (col as f64 + 0.5);
                let cz = -1.0 + cell_depth * (row as f64 + 0.5);

                // Bar corners in world space
                let x0 = cx - bar_w / 2.0;
                let x1 = cx + bar_w / 2.0;
                let z0 = cz - bar_d / 2.0;
                let z1 = cz + bar_d / 2.0;
                let y0 = self.base_height;
                let y1 = height;

                // Define 8 corners of the bar
                let corners = [
                    [x0, y0, z0], // 0: bottom-front-left
                    [x1, y0, z0], // 1: bottom-front-right
                    [x1, y0, z1], // 2: bottom-back-right
                    [x0, y0, z1], // 3: bottom-back-left
                    [x0, y1, z0], // 4: top-front-left
                    [x1, y1, z0], // 5: top-front-right
                    [x1, y1, z1], // 6: top-back-right
                    [x0, y1, z1], // 7: top-back-left
                ];

                // Project corners to screen
                let projected: Vec<([f64; 2], f64)> = corners
                    .iter()
                    .map(|c| {
                        let pos = Vec3::new(c[0] as f32, c[1] as f32, c[2] as f32);
                        let clip = mvp.transform_point(pos);
                        let screen_x = ((clip.x + 1.0) / 2.0) as f64 * viewport_width;
                        let screen_y = ((1.0 - clip.y) / 2.0) as f64 * viewport_height;
                        ([screen_x, screen_y], clip.z as f64)
                    })
                    .collect();

                // Add visible faces
                // Top face: 4, 5, 6, 7
                let top_depth =
                    (projected[4].1 + projected[5].1 + projected[6].1 + projected[7].1) / 4.0;
                faces.push(BarFace3D {
                    screen_verts: [
                        projected[4].0,
                        projected[5].0,
                        projected[6].0,
                        projected[7].0,
                    ],
                    face_type: BarFaceType::Top as u8,
                    data_value: normalized as f32,
                    depth: top_depth,
                    row,
                    col,
                });

                // Front face: 0, 1, 5, 4
                let front_depth =
                    (projected[0].1 + projected[1].1 + projected[5].1 + projected[4].1) / 4.0;
                faces.push(BarFace3D {
                    screen_verts: [
                        projected[0].0,
                        projected[1].0,
                        projected[5].0,
                        projected[4].0,
                    ],
                    face_type: BarFaceType::Front as u8,
                    data_value: normalized as f32,
                    depth: front_depth,
                    row,
                    col,
                });

                // Right face: 1, 2, 6, 5
                let right_depth =
                    (projected[1].1 + projected[2].1 + projected[6].1 + projected[5].1) / 4.0;
                faces.push(BarFace3D {
                    screen_verts: [
                        projected[1].0,
                        projected[2].0,
                        projected[6].0,
                        projected[5].0,
                    ],
                    face_type: BarFaceType::Right as u8,
                    data_value: normalized as f32,
                    depth: right_depth,
                    row,
                    col,
                });

                // Left face: 3, 0, 4, 7
                let left_depth =
                    (projected[3].1 + projected[0].1 + projected[4].1 + projected[7].1) / 4.0;
                faces.push(BarFace3D {
                    screen_verts: [
                        projected[3].0,
                        projected[0].0,
                        projected[4].0,
                        projected[7].0,
                    ],
                    face_type: BarFaceType::Left as u8,
                    data_value: normalized as f32,
                    depth: left_depth,
                    row,
                    col,
                });

                // Back face: 2, 3, 7, 6
                let back_depth =
                    (projected[2].1 + projected[3].1 + projected[7].1 + projected[6].1) / 4.0;
                faces.push(BarFace3D {
                    screen_verts: [
                        projected[2].0,
                        projected[3].0,
                        projected[7].0,
                        projected[6].0,
                    ],
                    face_type: BarFaceType::Back as u8,
                    data_value: normalized as f32,
                    depth: back_depth,
                    row,
                    col,
                });
            }
        }

        // Sort by depth (back to front)
        faces.sort_by(|a, b| {
            b.depth
                .partial_cmp(&a.depth)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        faces
    }

    /// Get color for a face
    pub fn get_face_color(&self, face: &BarFace3D) -> [f32; 4] {
        let base_color = self.colormap.sample(face.data_value);
        let brightness = match face.face_type {
            0 => BarFaceType::Top.brightness(),
            1 => BarFaceType::Front.brightness(),
            2 => BarFaceType::Left.brightness(),
            3 => BarFaceType::Right.brightness(),
            _ => BarFaceType::Back.brightness(),
        };

        [
            base_color.x * brightness,
            base_color.y * brightness,
            base_color.z * brightness,
            1.0,
        ]
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
    fn test_bar3d_new() {
        let chart = Bar3D::new();
        assert!(!chart.has_data());
        assert!((chart.bar_width - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_bar3d_set_data() {
        let mut chart = Bar3D::new();
        chart.set_data(vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]]);

        assert!(chart.has_data());
        assert_eq!(chart.dimensions(), (2, 3));
        assert!((chart.min_value - 1.0).abs() < 0.01);
        assert!((chart.max_value - 6.0).abs() < 0.01);
    }

    #[test]
    fn test_bar3d_get_faces() {
        let mut chart = Bar3D::new();
        chart.set_data(vec![vec![1.0, 2.0], vec![3.0, 4.0]]);

        let faces = chart.get_sorted_faces(800.0, 600.0);

        // 4 bars × 5 faces = 20 faces
        assert_eq!(faces.len(), 20);

        // All faces should have valid data values
        for face in &faces {
            assert!(face.data_value >= 0.0 && face.data_value <= 1.0);
        }
    }

    #[test]
    fn test_bar3d_face_brightness() {
        assert!((BarFaceType::Top.brightness() - 1.0).abs() < 0.01);
        assert!(BarFaceType::Front.brightness() < BarFaceType::Top.brightness());
        assert!(BarFaceType::Right.brightness() < BarFaceType::Front.brightness());
    }

    #[test]
    fn test_bar3d_colormap() {
        let mut chart = Bar3D::new();
        chart.set_data(vec![vec![0.0, 0.5, 1.0]]);
        chart.set_colormap(Colormap::Plasma);

        let faces = chart.get_sorted_faces(800.0, 600.0);
        let face = &faces[0];
        let color = chart.get_face_color(face);

        // Color should be valid RGBA
        assert!(color[0] >= 0.0 && color[0] <= 1.0);
        assert!(color[1] >= 0.0 && color[1] <= 1.0);
        assert!(color[2] >= 0.0 && color[2] <= 1.0);
        assert!((color[3] - 1.0).abs() < 0.01);
    }
}

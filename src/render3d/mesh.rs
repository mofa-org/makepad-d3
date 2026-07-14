//! Mesh data structures for 3D rendering
//!
//! This module provides mesh data structures optimized for GPU upload,
//! including interleaved vertex data and index buffers.

use super::types::{Mat4, Vec3};

/// Number of floats per vertex in the interleaved buffer
///
/// Layout: pos(3) + id(1) + normal(3) + uv(2) = 9 floats = 36 bytes
pub const FLOATS_PER_VERTEX: usize = 9;

/// Vertex data offsets within the interleaved buffer
pub mod vertex_offset {
    /// Position X offset (floats 0-2)
    pub const POSITION: usize = 0;
    /// Element ID offset (float 3)
    pub const ID: usize = 3;
    /// Normal offset (floats 4-6)
    pub const NORMAL: usize = 4;
    /// UV offset (floats 7-8)
    pub const UV: usize = 7;
}

/// Mesh data container with interleaved vertices and indices
///
/// The vertex buffer uses an interleaved layout for optimal GPU cache performance:
///
/// ```text
/// Vertex N: [pos.x, pos.y, pos.z, id, normal.x, normal.y, normal.z, u, v]
///           [  0  ,   1  ,   2  ,  3,     4   ,     5   ,     6   , 7, 8]
/// ```
///
/// # Example
///
/// ```rust
/// use makepad_d3::render3d::mesh::MeshData;
///
/// // Create a surface from a height function
/// let mesh = MeshData::surface(
///     50,  // resolution
///     (-2.0, 2.0),  // x range
///     (-2.0, 2.0),  // z range
///     |x, z| (x * x + z * z).sin(),  // height function
/// );
///
/// println!("Vertices: {}, Triangles: {}", mesh.vertex_count(), mesh.triangle_count());
/// ```
#[derive(Clone, Debug, Default)]
pub struct MeshData {
    /// Interleaved vertex data: [pos(3), id(1), normal(3), uv(2)] per vertex
    pub vertices: Vec<f32>,
    /// Triangle indices (3 indices per triangle)
    pub indices: Vec<u32>,
    /// Axis-aligned bounding box minimum
    pub bounds_min: [f32; 3],
    /// Axis-aligned bounding box maximum
    pub bounds_max: [f32; 3],
}

impl MeshData {
    /// Create a new empty mesh
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            bounds_min: [f32::MAX, f32::MAX, f32::MAX],
            bounds_max: [f32::MIN, f32::MIN, f32::MIN],
        }
    }

    /// Create a mesh with pre-allocated capacity
    pub fn with_capacity(vertex_count: usize, triangle_count: usize) -> Self {
        Self {
            vertices: Vec::with_capacity(vertex_count * FLOATS_PER_VERTEX),
            indices: Vec::with_capacity(triangle_count * 3),
            bounds_min: [f32::MAX, f32::MAX, f32::MAX],
            bounds_max: [f32::MIN, f32::MIN, f32::MIN],
        }
    }

    /// Get the number of vertices
    pub fn vertex_count(&self) -> usize {
        self.vertices.len() / FLOATS_PER_VERTEX
    }

    /// Get the number of triangles
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Add a vertex to the mesh
    ///
    /// Returns the index of the added vertex.
    pub fn add_vertex(
        &mut self,
        position: [f32; 3],
        id: f32,
        normal: [f32; 3],
        uv: [f32; 2],
    ) -> u32 {
        let index = self.vertex_count() as u32;

        // Position
        self.vertices.push(position[0]);
        self.vertices.push(position[1]);
        self.vertices.push(position[2]);

        // ID
        self.vertices.push(id);

        // Normal
        self.vertices.push(normal[0]);
        self.vertices.push(normal[1]);
        self.vertices.push(normal[2]);

        // UV
        self.vertices.push(uv[0]);
        self.vertices.push(uv[1]);

        // Update bounds
        for i in 0..3 {
            self.bounds_min[i] = self.bounds_min[i].min(position[i]);
            self.bounds_max[i] = self.bounds_max[i].max(position[i]);
        }

        index
    }

    /// Add a triangle from vertex indices
    pub fn add_triangle(&mut self, i0: u32, i1: u32, i2: u32) {
        self.indices.push(i0);
        self.indices.push(i1);
        self.indices.push(i2);
    }

    /// Get the position of a vertex
    pub fn get_position(&self, index: usize) -> [f32; 3] {
        let base = index * FLOATS_PER_VERTEX + vertex_offset::POSITION;
        [
            self.vertices[base],
            self.vertices[base + 1],
            self.vertices[base + 2],
        ]
    }

    /// Get the normal of a vertex
    pub fn get_normal(&self, index: usize) -> [f32; 3] {
        let base = index * FLOATS_PER_VERTEX + vertex_offset::NORMAL;
        [
            self.vertices[base],
            self.vertices[base + 1],
            self.vertices[base + 2],
        ]
    }

    /// Set the normal of a vertex
    pub fn set_normal(&mut self, index: usize, normal: [f32; 3]) {
        let base = index * FLOATS_PER_VERTEX + vertex_offset::NORMAL;
        self.vertices[base] = normal[0];
        self.vertices[base + 1] = normal[1];
        self.vertices[base + 2] = normal[2];
    }

    /// Compute smooth normals from face geometry
    ///
    /// This averages face normals for each vertex, producing smooth shading.
    pub fn compute_normals(&mut self) {
        let vertex_count = self.vertex_count();
        if vertex_count == 0 {
            return;
        }

        // Initialize normals to zero
        let mut normals: Vec<[f32; 3]> = vec![[0.0, 0.0, 0.0]; vertex_count];

        // Accumulate face normals for each vertex
        for i in (0..self.indices.len()).step_by(3) {
            let i0 = self.indices[i] as usize;
            let i1 = self.indices[i + 1] as usize;
            let i2 = self.indices[i + 2] as usize;

            let p0 = self.get_position(i0);
            let p1 = self.get_position(i1);
            let p2 = self.get_position(i2);

            // Compute edge vectors
            let edge1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
            let edge2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];

            // Cross product for face normal
            let normal = [
                edge1[1] * edge2[2] - edge1[2] * edge2[1],
                edge1[2] * edge2[0] - edge1[0] * edge2[2],
                edge1[0] * edge2[1] - edge1[1] * edge2[0],
            ];

            // Accumulate to all three vertices
            for &idx in &[i0, i1, i2] {
                normals[idx][0] += normal[0];
                normals[idx][1] += normal[1];
                normals[idx][2] += normal[2];
            }
        }

        // Normalize and write back
        for i in 0..vertex_count {
            let n = &normals[i];
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            if len > 1e-10 {
                self.set_normal(i, [n[0] / len, n[1] / len, n[2] / len]);
            } else {
                self.set_normal(i, [0.0, 1.0, 0.0]); // Default up normal
            }
        }
    }

    /// Recompute the bounding box from vertices
    pub fn compute_bounds(&mut self) {
        self.bounds_min = [f32::MAX, f32::MAX, f32::MAX];
        self.bounds_max = [f32::MIN, f32::MIN, f32::MIN];

        for i in 0..self.vertex_count() {
            let pos = self.get_position(i);
            for j in 0..3 {
                self.bounds_min[j] = self.bounds_min[j].min(pos[j]);
                self.bounds_max[j] = self.bounds_max[j].max(pos[j]);
            }
        }
    }

    /// Make the mesh double-sided by duplicating triangles with flipped normals
    pub fn make_double_sided(&mut self) {
        let original_vertex_count = self.vertex_count();
        let original_index_count = self.indices.len();

        // Duplicate vertices with flipped normals
        for i in 0..original_vertex_count {
            let pos = self.get_position(i);
            let normal = self.get_normal(i);
            let id = self.vertices[i * FLOATS_PER_VERTEX + vertex_offset::ID];
            let uv = [
                self.vertices[i * FLOATS_PER_VERTEX + vertex_offset::UV],
                self.vertices[i * FLOATS_PER_VERTEX + vertex_offset::UV + 1],
            ];

            // Add with flipped normal
            self.add_vertex(pos, id, [-normal[0], -normal[1], -normal[2]], uv);
        }

        // Add reversed triangles
        for i in (0..original_index_count).step_by(3) {
            let i0 = self.indices[i] + original_vertex_count as u32;
            let i1 = self.indices[i + 1] + original_vertex_count as u32;
            let i2 = self.indices[i + 2] + original_vertex_count as u32;

            // Reversed winding order
            self.add_triangle(i0, i2, i1);
        }
    }

    /// Apply a transform to all vertices (bakes the transform into geometry)
    pub fn transform(&mut self, m: &Mat4) {
        for i in 0..self.vertex_count() {
            let base = i * FLOATS_PER_VERTEX;

            // Transform position
            let pos = Vec3::new(
                self.vertices[base],
                self.vertices[base + 1],
                self.vertices[base + 2],
            );
            let new_pos = m.transform_point(pos);
            self.vertices[base] = new_pos.x;
            self.vertices[base + 1] = new_pos.y;
            self.vertices[base + 2] = new_pos.z;

            // Transform normal (direction only)
            let normal = Vec3::new(
                self.vertices[base + 4],
                self.vertices[base + 5],
                self.vertices[base + 6],
            );
            let new_normal = m.transform_vector(normal).normalize();
            self.vertices[base + 4] = new_normal.x;
            self.vertices[base + 5] = new_normal.y;
            self.vertices[base + 6] = new_normal.z;
        }

        self.compute_bounds();
    }

    /// Normalize the mesh to fit in a unit cube centered at origin
    pub fn normalize(&mut self) {
        if self.vertex_count() == 0 {
            return;
        }

        self.compute_bounds();

        // Compute center and size
        let center = [
            (self.bounds_min[0] + self.bounds_max[0]) / 2.0,
            (self.bounds_min[1] + self.bounds_max[1]) / 2.0,
            (self.bounds_min[2] + self.bounds_max[2]) / 2.0,
        ];

        let size = [
            self.bounds_max[0] - self.bounds_min[0],
            self.bounds_max[1] - self.bounds_min[1],
            self.bounds_max[2] - self.bounds_min[2],
        ];

        let max_size = size[0].max(size[1]).max(size[2]);
        let scale = if max_size > 1e-10 {
            1.0 / max_size
        } else {
            1.0
        };

        // Apply centering and scaling
        for i in 0..self.vertex_count() {
            let base = i * FLOATS_PER_VERTEX;
            self.vertices[base] = (self.vertices[base] - center[0]) * scale;
            self.vertices[base + 1] = (self.vertices[base + 1] - center[1]) * scale;
            self.vertices[base + 2] = (self.vertices[base + 2] - center[2]) * scale;
        }

        self.compute_bounds();
    }

    // ========== Primitive Generators ==========

    /// Create a unit cube centered at origin
    pub fn cube(size: f32) -> Self {
        let mut mesh = Self::with_capacity(24, 12);
        let h = size / 2.0;

        // Define 6 faces with separate vertices for flat shading
        let faces = [
            // Front face (normal: +Z)
            (
                [[-h, -h, h], [h, -h, h], [h, h, h], [-h, h, h]],
                [0.0, 0.0, 1.0],
            ),
            // Back face (normal: -Z)
            (
                [[h, -h, -h], [-h, -h, -h], [-h, h, -h], [h, h, -h]],
                [0.0, 0.0, -1.0],
            ),
            // Top face (normal: +Y)
            (
                [[-h, h, h], [h, h, h], [h, h, -h], [-h, h, -h]],
                [0.0, 1.0, 0.0],
            ),
            // Bottom face (normal: -Y)
            (
                [[-h, -h, -h], [h, -h, -h], [h, -h, h], [-h, -h, h]],
                [0.0, -1.0, 0.0],
            ),
            // Right face (normal: +X)
            (
                [[h, -h, h], [h, -h, -h], [h, h, -h], [h, h, h]],
                [1.0, 0.0, 0.0],
            ),
            // Left face (normal: -X)
            (
                [[-h, -h, -h], [-h, -h, h], [-h, h, h], [-h, h, -h]],
                [-1.0, 0.0, 0.0],
            ),
        ];

        let uvs = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];

        for (face_idx, (positions, normal)) in faces.iter().enumerate() {
            let base = mesh.vertex_count() as u32;

            for (i, pos) in positions.iter().enumerate() {
                mesh.add_vertex(*pos, face_idx as f32, *normal, uvs[i]);
            }

            mesh.add_triangle(base, base + 1, base + 2);
            mesh.add_triangle(base, base + 2, base + 3);
        }

        mesh
    }

    /// Create a sphere with latitude/longitude tessellation
    pub fn sphere(radius: f32, lat_segments: usize, lon_segments: usize) -> Self {
        let lat_segments = lat_segments.max(3);
        let lon_segments = lon_segments.max(3);

        let vertex_count = (lat_segments + 1) * (lon_segments + 1);
        let triangle_count = lat_segments * lon_segments * 2;
        let mut mesh = Self::with_capacity(vertex_count, triangle_count);

        // Generate vertices
        for lat in 0..=lat_segments {
            let theta = std::f32::consts::PI * lat as f32 / lat_segments as f32;
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();

            for lon in 0..=lon_segments {
                let phi = 2.0 * std::f32::consts::PI * lon as f32 / lon_segments as f32;
                let sin_phi = phi.sin();
                let cos_phi = phi.cos();

                let x = sin_theta * cos_phi;
                let y = cos_theta;
                let z = sin_theta * sin_phi;

                let position = [x * radius, y * radius, z * radius];
                let normal = [x, y, z];
                let uv = [
                    lon as f32 / lon_segments as f32,
                    lat as f32 / lat_segments as f32,
                ];
                let id = (lat * (lon_segments + 1) + lon) as f32;

                mesh.add_vertex(position, id, normal, uv);
            }
        }

        // Generate indices
        for lat in 0..lat_segments {
            for lon in 0..lon_segments {
                let first = (lat * (lon_segments + 1) + lon) as u32;
                let second = first + lon_segments as u32 + 1;

                mesh.add_triangle(first, second, first + 1);
                mesh.add_triangle(second, second + 1, first + 1);
            }
        }

        mesh
    }

    /// Create a cylinder along the Y axis
    pub fn cylinder(radius: f32, height: f32, segments: usize) -> Self {
        let segments = segments.max(3);
        let vertex_count = segments * 4 + 2; // sides + top/bottom centers
        let triangle_count = segments * 4; // sides + caps
        let mut mesh = Self::with_capacity(vertex_count, triangle_count);

        let half_height = height / 2.0;

        // Side vertices
        for i in 0..=segments {
            let angle = 2.0 * std::f32::consts::PI * i as f32 / segments as f32;
            let x = angle.cos() * radius;
            let z = angle.sin() * radius;
            let u = i as f32 / segments as f32;

            // Bottom vertex
            mesh.add_vertex(
                [x, -half_height, z],
                i as f32,
                [angle.cos(), 0.0, angle.sin()],
                [u, 0.0],
            );

            // Top vertex
            mesh.add_vertex(
                [x, half_height, z],
                (i + segments + 1) as f32,
                [angle.cos(), 0.0, angle.sin()],
                [u, 1.0],
            );
        }

        // Side triangles
        for i in 0..segments {
            let base = (i * 2) as u32;
            mesh.add_triangle(base, base + 2, base + 1);
            mesh.add_triangle(base + 1, base + 2, base + 3);
        }

        // Top cap center
        let top_center = mesh.add_vertex(
            [0.0, half_height, 0.0],
            (segments * 2 + 2) as f32,
            [0.0, 1.0, 0.0],
            [0.5, 0.5],
        );

        // Bottom cap center
        let bottom_center = mesh.add_vertex(
            [0.0, -half_height, 0.0],
            (segments * 2 + 3) as f32,
            [0.0, -1.0, 0.0],
            [0.5, 0.5],
        );

        // Cap vertices and triangles
        for i in 0..segments {
            let angle0 = 2.0 * std::f32::consts::PI * i as f32 / segments as f32;
            let angle1 = 2.0 * std::f32::consts::PI * (i + 1) as f32 / segments as f32;

            let x0 = angle0.cos() * radius;
            let z0 = angle0.sin() * radius;
            let x1 = angle1.cos() * radius;
            let z1 = angle1.sin() * radius;

            // Top cap
            let t0 = mesh.add_vertex([x0, half_height, z0], i as f32, [0.0, 1.0, 0.0], [0.0, 0.0]);
            let t1 = mesh.add_vertex([x1, half_height, z1], i as f32, [0.0, 1.0, 0.0], [1.0, 0.0]);
            mesh.add_triangle(top_center, t0, t1);

            // Bottom cap
            let b0 = mesh.add_vertex(
                [x0, -half_height, z0],
                i as f32,
                [0.0, -1.0, 0.0],
                [0.0, 0.0],
            );
            let b1 = mesh.add_vertex(
                [x1, -half_height, z1],
                i as f32,
                [0.0, -1.0, 0.0],
                [1.0, 0.0],
            );
            mesh.add_triangle(bottom_center, b1, b0);
        }

        mesh
    }

    /// Create a plane in the XZ plane
    pub fn plane(width: f32, depth: f32, segments: usize) -> Self {
        let segments = segments.max(1);
        let vertex_count = (segments + 1) * (segments + 1);
        let triangle_count = segments * segments * 2;
        let mut mesh = Self::with_capacity(vertex_count, triangle_count);

        let half_width = width / 2.0;
        let half_depth = depth / 2.0;

        // Generate vertices
        for i in 0..=segments {
            for j in 0..=segments {
                let u = i as f32 / segments as f32;
                let v = j as f32 / segments as f32;

                let x = -half_width + width * u;
                let z = -half_depth + depth * v;

                mesh.add_vertex(
                    [x, 0.0, z],
                    (i * (segments + 1) + j) as f32,
                    [0.0, 1.0, 0.0],
                    [u, v],
                );
            }
        }

        // Generate indices
        for i in 0..segments {
            for j in 0..segments {
                let idx = (i * (segments + 1) + j) as u32;
                let next_row = (segments + 1) as u32;

                mesh.add_triangle(idx, idx + next_row, idx + 1);
                mesh.add_triangle(idx + 1, idx + next_row, idx + next_row + 1);
            }
        }

        mesh
    }

    /// Create a surface mesh from a height function
    ///
    /// # Arguments
    ///
    /// * `resolution` - Number of vertices per side (resolution × resolution grid)
    /// * `x_range` - (min, max) range for X coordinates
    /// * `z_range` - (min, max) range for Z coordinates
    /// * `height_fn` - Function that computes Y height from (x, z)
    ///
    /// # Example
    ///
    /// ```rust
    /// use makepad_d3::render3d::mesh::MeshData;
    ///
    /// // Create a sine wave surface
    /// let mesh = MeshData::surface(
    ///     100,
    ///     (-3.14, 3.14),
    ///     (-3.14, 3.14),
    ///     |x, z| (x.sin() * z.sin()),
    /// );
    /// ```
    pub fn surface<F>(
        resolution: usize,
        x_range: (f32, f32),
        z_range: (f32, f32),
        height_fn: F,
    ) -> Self
    where
        F: Fn(f32, f32) -> f32,
    {
        let resolution = resolution.max(2);
        let vertex_count = resolution * resolution;
        let triangle_count = (resolution - 1) * (resolution - 1) * 2;
        let mut mesh = Self::with_capacity(vertex_count, triangle_count);

        // Generate vertices
        for i in 0..resolution {
            for j in 0..resolution {
                let u = i as f32 / (resolution - 1) as f32;
                let v = j as f32 / (resolution - 1) as f32;

                let x = x_range.0 + (x_range.1 - x_range.0) * u;
                let z = z_range.0 + (z_range.1 - z_range.0) * v;
                let y = height_fn(x, z);

                mesh.add_vertex(
                    [x, y, z],
                    (i * resolution + j) as f32,
                    [0.0, 1.0, 0.0], // Placeholder normal
                    [u, v],
                );
            }
        }

        // Generate indices
        for i in 0..(resolution - 1) {
            for j in 0..(resolution - 1) {
                let idx = (i * resolution + j) as u32;
                let next_row = resolution as u32;

                mesh.add_triangle(idx, idx + next_row, idx + 1);
                mesh.add_triangle(idx + 1, idx + next_row, idx + next_row + 1);
            }
        }

        // Compute proper normals
        mesh.compute_normals();

        mesh
    }

    /// Create a grid of lines for a ground plane
    pub fn grid(size: f32, divisions: usize) -> Self {
        let divisions = divisions.max(1);
        let line_count = (divisions + 1) * 2;
        let vertex_count = line_count * 2;
        let triangle_count = line_count; // Thin quads as lines
        let mut mesh = Self::with_capacity(vertex_count, triangle_count * 2);

        let half_size = size / 2.0;
        let step = size / divisions as f32;
        let line_width = 0.01;

        // X-parallel lines
        for i in 0..=divisions {
            let z = -half_size + i as f32 * step;

            let v0 = mesh.add_vertex(
                [-half_size, 0.0, z - line_width],
                i as f32,
                [0.0, 1.0, 0.0],
                [0.0, 0.0],
            );
            let v1 = mesh.add_vertex(
                [half_size, 0.0, z - line_width],
                i as f32,
                [0.0, 1.0, 0.0],
                [1.0, 0.0],
            );
            let v2 = mesh.add_vertex(
                [half_size, 0.0, z + line_width],
                i as f32,
                [0.0, 1.0, 0.0],
                [1.0, 1.0],
            );
            let v3 = mesh.add_vertex(
                [-half_size, 0.0, z + line_width],
                i as f32,
                [0.0, 1.0, 0.0],
                [0.0, 1.0],
            );

            mesh.add_triangle(v0, v1, v2);
            mesh.add_triangle(v0, v2, v3);
        }

        // Z-parallel lines
        for i in 0..=divisions {
            let x = -half_size + i as f32 * step;

            let v0 = mesh.add_vertex(
                [x - line_width, 0.0, -half_size],
                (i + divisions + 1) as f32,
                [0.0, 1.0, 0.0],
                [0.0, 0.0],
            );
            let v1 = mesh.add_vertex(
                [x + line_width, 0.0, -half_size],
                (i + divisions + 1) as f32,
                [0.0, 1.0, 0.0],
                [1.0, 0.0],
            );
            let v2 = mesh.add_vertex(
                [x + line_width, 0.0, half_size],
                (i + divisions + 1) as f32,
                [0.0, 1.0, 0.0],
                [1.0, 1.0],
            );
            let v3 = mesh.add_vertex(
                [x - line_width, 0.0, half_size],
                (i + divisions + 1) as f32,
                [0.0, 1.0, 0.0],
                [0.0, 1.0],
            );

            mesh.add_triangle(v0, v1, v2);
            mesh.add_triangle(v0, v2, v3);
        }

        mesh
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mesh_new() {
        let mesh = MeshData::new();
        assert_eq!(mesh.vertex_count(), 0);
        assert_eq!(mesh.triangle_count(), 0);
    }

    #[test]
    fn test_add_vertex() {
        let mut mesh = MeshData::new();
        let idx = mesh.add_vertex([1.0, 2.0, 3.0], 0.0, [0.0, 1.0, 0.0], [0.5, 0.5]);

        assert_eq!(idx, 0);
        assert_eq!(mesh.vertex_count(), 1);
        assert_eq!(mesh.vertices.len(), FLOATS_PER_VERTEX);
    }

    #[test]
    fn test_add_triangle() {
        let mut mesh = MeshData::new();
        mesh.add_vertex([0.0, 0.0, 0.0], 0.0, [0.0, 1.0, 0.0], [0.0, 0.0]);
        mesh.add_vertex([1.0, 0.0, 0.0], 1.0, [0.0, 1.0, 0.0], [1.0, 0.0]);
        mesh.add_vertex([0.0, 0.0, 1.0], 2.0, [0.0, 1.0, 0.0], [0.0, 1.0]);
        mesh.add_triangle(0, 1, 2);

        assert_eq!(mesh.triangle_count(), 1);
        assert_eq!(mesh.indices, vec![0, 1, 2]);
    }

    #[test]
    fn test_cube() {
        let mesh = MeshData::cube(2.0);

        assert_eq!(mesh.vertex_count(), 24); // 4 vertices × 6 faces
        assert_eq!(mesh.triangle_count(), 12); // 2 triangles × 6 faces
    }

    #[test]
    fn test_sphere() {
        let mesh = MeshData::sphere(1.0, 8, 8);

        assert!(mesh.vertex_count() > 0);
        assert!(mesh.triangle_count() > 0);
    }

    #[test]
    fn test_surface() {
        let mesh = MeshData::surface(10, (-1.0, 1.0), (-1.0, 1.0), |x, z| x * z);

        assert_eq!(mesh.vertex_count(), 100); // 10 × 10
        assert_eq!(mesh.triangle_count(), 162); // (10-1) × (10-1) × 2
    }

    #[test]
    fn test_compute_normals() {
        let mut mesh = MeshData::new();

        // Create a simple triangle in XZ plane with counter-clockwise winding
        // when viewed from above (+Y)
        mesh.add_vertex([0.0, 0.0, 0.0], 0.0, [0.0, 0.0, 0.0], [0.0, 0.0]);
        mesh.add_vertex([0.0, 0.0, 1.0], 1.0, [0.0, 0.0, 0.0], [0.0, 1.0]);
        mesh.add_vertex([1.0, 0.0, 0.0], 2.0, [0.0, 0.0, 0.0], [1.0, 0.0]);
        mesh.add_triangle(0, 1, 2);

        mesh.compute_normals();

        // All normals should point up (positive Y)
        for i in 0..3 {
            let normal = mesh.get_normal(i);
            assert!(
                (normal[1] - 1.0).abs() < 0.01,
                "Normal Y should be ~1.0, got {:?}",
                normal
            );
        }
    }

    #[test]
    fn test_bounds() {
        let mesh = MeshData::cube(2.0);

        assert!((mesh.bounds_min[0] - (-1.0)).abs() < 0.01);
        assert!((mesh.bounds_max[0] - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_normalize() {
        let mut mesh = MeshData::cube(4.0); // Cube with size 4 (bounds -2 to 2)
        mesh.normalize();

        // After normalization, max dimension should be 1.0
        let size_x = mesh.bounds_max[0] - mesh.bounds_min[0];
        let size_y = mesh.bounds_max[1] - mesh.bounds_min[1];
        let size_z = mesh.bounds_max[2] - mesh.bounds_min[2];
        let max_size = size_x.max(size_y).max(size_z);

        assert!((max_size - 1.0).abs() < 0.01);
    }
}

//! GPU geometry wrapper for Makepad
//!
//! This module provides a wrapper around Makepad's geometry system for efficient
//! GPU mesh rendering.

use super::mesh::MeshData;

/// GPU geometry wrapper for 3D meshes
///
/// `GeometryMesh3D` manages the GPU-side representation of mesh data.
/// It handles uploading vertex and index data to the GPU and provides
/// the interface needed by Makepad's shader system.
///
/// # Lifecycle
///
/// 1. Create a `GeometryMesh3D` with `new()`
/// 2. Set mesh data with `set_mesh()`
/// 3. Call `ensure_uploaded()` before drawing to upload to GPU
/// 4. The geometry can be reused across frames without re-uploading
///
/// # Example
///
/// ```rust,no_run
/// use makepad_d3::render3d::{GeometryMesh3D, MeshData};
///
/// let mut geometry = GeometryMesh3D::new();
///
/// // Set mesh data
/// let mesh = MeshData::cube(1.0);
/// geometry.set_mesh(mesh);
///
/// // In draw function:
/// // geometry.ensure_uploaded(cx);
/// // draw_mesh.draw(cx);
/// ```
#[derive(Clone, Debug, Default)]
pub struct GeometryMesh3D {
    /// The mesh data (CPU-side)
    mesh_data: Option<MeshData>,

    /// Unique instance ID for geometry fingerprinting
    instance_id: u64,

    /// Whether the mesh data has changed and needs re-upload
    dirty: bool,

    /// Whether the geometry has been uploaded
    uploaded: bool,
}

/// Counter for generating unique instance IDs
static INSTANCE_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

impl GeometryMesh3D {
    /// Create a new empty geometry
    pub fn new() -> Self {
        Self {
            mesh_data: None,
            instance_id: INSTANCE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            dirty: false,
            uploaded: false,
        }
    }

    /// Create geometry from mesh data
    pub fn from_mesh(mesh: MeshData) -> Self {
        let mut geo = Self::new();
        geo.set_mesh(mesh);
        geo
    }

    /// Set the mesh data
    ///
    /// This marks the geometry as dirty, requiring re-upload on next draw.
    pub fn set_mesh(&mut self, mesh: MeshData) {
        self.mesh_data = Some(mesh);
        self.dirty = true;
        self.uploaded = false;
    }

    /// Update mesh data in place
    ///
    /// Use this for dynamic mesh updates (e.g., animated surfaces).
    pub fn update_mesh<F>(&mut self, f: F)
    where
        F: FnOnce(&mut MeshData),
    {
        if let Some(ref mut mesh) = self.mesh_data {
            f(mesh);
            self.dirty = true;
        }
    }

    /// Check if the geometry needs to be uploaded
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Check if the geometry has been uploaded
    pub fn is_uploaded(&self) -> bool {
        self.uploaded
    }

    /// Get the instance ID
    pub fn instance_id(&self) -> u64 {
        self.instance_id
    }

    /// Get a reference to the mesh data
    pub fn mesh_data(&self) -> Option<&MeshData> {
        self.mesh_data.as_ref()
    }

    /// Get the vertex count
    pub fn vertex_count(&self) -> usize {
        self.mesh_data.as_ref().map_or(0, |m| m.vertex_count())
    }

    /// Get the triangle count
    pub fn triangle_count(&self) -> usize {
        self.mesh_data.as_ref().map_or(0, |m| m.triangle_count())
    }

    /// Get the vertex buffer (for GPU upload)
    pub fn vertices(&self) -> Option<&[f32]> {
        self.mesh_data.as_ref().map(|m| m.vertices.as_slice())
    }

    /// Get the index buffer (for GPU upload)
    pub fn indices(&self) -> Option<&[u32]> {
        self.mesh_data.as_ref().map(|m| m.indices.as_slice())
    }

    /// Mark as uploaded (call after GPU upload completes)
    pub fn mark_uploaded(&mut self) {
        self.dirty = false;
        self.uploaded = true;
    }

    /// Mark as dirty (forces re-upload)
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Get bounds minimum
    pub fn bounds_min(&self) -> [f32; 3] {
        self.mesh_data.as_ref().map_or([0.0; 3], |m| m.bounds_min)
    }

    /// Get bounds maximum
    pub fn bounds_max(&self) -> [f32; 3] {
        self.mesh_data.as_ref().map_or([0.0; 3], |m| m.bounds_max)
    }

    /// Get the center of the bounding box
    pub fn center(&self) -> [f32; 3] {
        let min = self.bounds_min();
        let max = self.bounds_max();
        [
            (min[0] + max[0]) / 2.0,
            (min[1] + max[1]) / 2.0,
            (min[2] + max[2]) / 2.0,
        ]
    }

    /// Get the size of the bounding box
    pub fn size(&self) -> [f32; 3] {
        let min = self.bounds_min();
        let max = self.bounds_max();
        [max[0] - min[0], max[1] - min[1], max[2] - min[2]]
    }
}

/// Geometry field descriptor for shader binding
///
/// This describes a single vertex attribute for the shader.
#[derive(Clone, Debug)]
pub struct GeometryField {
    /// Attribute name (must match shader variable)
    pub name: &'static str,
    /// Attribute type
    pub ty: GeometryFieldType,
    /// Offset in floats from start of vertex
    pub offset: usize,
}

/// Types of geometry fields
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GeometryFieldType {
    /// Single float
    Float,
    /// 2-component vector
    Vec2,
    /// 3-component vector
    Vec3,
    /// 4-component vector
    Vec4,
}

impl GeometryFieldType {
    /// Get the number of floats for this type
    pub fn float_count(&self) -> usize {
        match self {
            Self::Float => 1,
            Self::Vec2 => 2,
            Self::Vec3 => 3,
            Self::Vec4 => 4,
        }
    }
}

/// Standard geometry field layout for 3D meshes
///
/// This matches the layout defined in `mesh.rs`:
/// - Position (Vec3) at offset 0
/// - ID (Float) at offset 3
/// - Normal (Vec3) at offset 4
/// - UV (Vec2) at offset 7
pub fn standard_geometry_fields() -> Vec<GeometryField> {
    vec![
        GeometryField {
            name: "geom_pos",
            ty: GeometryFieldType::Vec3,
            offset: 0,
        },
        GeometryField {
            name: "geom_id",
            ty: GeometryFieldType::Float,
            offset: 3,
        },
        GeometryField {
            name: "geom_normal",
            ty: GeometryFieldType::Vec3,
            offset: 4,
        },
        GeometryField {
            name: "geom_uv",
            ty: GeometryFieldType::Vec2,
            offset: 7,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geometry_new() {
        let geo = GeometryMesh3D::new();
        assert!(!geo.is_dirty());
        assert!(!geo.is_uploaded());
        assert!(geo.mesh_data().is_none());
    }

    #[test]
    fn test_geometry_from_mesh() {
        let mesh = MeshData::cube(1.0);
        let vertex_count = mesh.vertex_count();

        let geo = GeometryMesh3D::from_mesh(mesh);
        assert!(geo.is_dirty());
        assert!(!geo.is_uploaded());
        assert_eq!(geo.vertex_count(), vertex_count);
    }

    #[test]
    fn test_geometry_set_mesh() {
        let mut geo = GeometryMesh3D::new();
        geo.set_mesh(MeshData::sphere(1.0, 8, 8));

        assert!(geo.is_dirty());
        assert!(geo.vertex_count() > 0);
    }

    #[test]
    fn test_geometry_mark_uploaded() {
        let mut geo = GeometryMesh3D::from_mesh(MeshData::cube(1.0));
        assert!(geo.is_dirty());

        geo.mark_uploaded();
        assert!(!geo.is_dirty());
        assert!(geo.is_uploaded());
    }

    #[test]
    fn test_geometry_update_mesh() {
        let mut geo = GeometryMesh3D::from_mesh(MeshData::cube(1.0));
        geo.mark_uploaded();

        geo.update_mesh(|mesh| {
            mesh.normalize();
        });

        assert!(geo.is_dirty());
    }

    #[test]
    fn test_unique_instance_ids() {
        let geo1 = GeometryMesh3D::new();
        let geo2 = GeometryMesh3D::new();

        assert_ne!(geo1.instance_id(), geo2.instance_id());
    }

    #[test]
    fn test_standard_fields() {
        let fields = standard_geometry_fields();
        assert_eq!(fields.len(), 4);

        assert_eq!(fields[0].name, "geom_pos");
        assert_eq!(fields[0].ty, GeometryFieldType::Vec3);
        assert_eq!(fields[0].offset, 0);

        assert_eq!(fields[1].name, "geom_id");
        assert_eq!(fields[1].ty, GeometryFieldType::Float);
        assert_eq!(fields[1].offset, 3);
    }

    #[test]
    fn test_bounds() {
        let geo = GeometryMesh3D::from_mesh(MeshData::cube(2.0));

        let min = geo.bounds_min();
        let max = geo.bounds_max();
        let center = geo.center();
        let size = geo.size();

        assert!((min[0] - (-1.0)).abs() < 0.01);
        assert!((max[0] - 1.0).abs() < 0.01);
        assert!((center[0] - 0.0).abs() < 0.01);
        assert!((size[0] - 2.0).abs() < 0.01);
    }
}

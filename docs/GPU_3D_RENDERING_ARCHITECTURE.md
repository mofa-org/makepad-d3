# GPU 3D Rendering Architecture for Makepad D3

**Version:** 1.0
**Created:** 2026-01-18
**Status:** Reference Architecture (see note below)
**Based on:** Analysis of `makepad-urdf-player` GPU rendering pipeline

> **Note (2026-07-13):** Makepad 2.0 replaced the Live system with the
> Script/Splash system. The pipeline concepts in this document (passes,
> depth, instancing, camera) remain valid, but all `live_design!` shader
> samples are superseded — shaders are now authored in `script_mod!` blocks
> and bound via `script_shader`. See
> [`SPLASH_INTEGRATION_DESIGN.md`](SPLASH_INTEGRATION_DESIGN.md) §9 for the
> render3d migration.

---

## Table of Contents

1. [Overview](#1-overview)
2. [Current vs Target Architecture](#2-current-vs-target-architecture)
3. [GPU Shader Pipeline](#3-gpu-shader-pipeline)
4. [Vertex Data Layout](#4-vertex-data-layout)
5. [Transform System](#5-transform-system)
6. [Lighting Model](#6-lighting-model)
7. [Camera System](#7-camera-system)
8. [Mesh Generation](#8-mesh-generation)
9. [Depth Handling](#9-depth-handling)
10. [Implementation Patterns](#10-implementation-patterns)
11. [Performance Considerations](#11-performance-considerations)

---

## 1. Overview

This document defines the GPU-accelerated 3D rendering architecture for Makepad D3, derived from analysis of the production-quality `makepad-urdf-player` implementation.

### Key Principles

| Principle | Description |
|-----------|-------------|
| **GPU-First** | All transforms, lighting, and projections run on GPU |
| **Transform-Only Updates** | Geometry uploaded once, only 64-byte transforms sent per frame |
| **Modular Shaders** | Separate DrawMesh, DrawGrid, DrawSkybox components |
| **Trait-Based Extensibility** | GeometryFields trait for custom vertex layouts |

### Applicable Visualizations

- Surface Plots (3D height maps)
- 3D Scatter Plots
- 3D Bar Charts
- Globe/Map Projections
- Network Graphs (3D force-directed)
- Volumetric Visualizations

---

## 2. Current vs Target Architecture

### Current Implementation (`surface_plot.rs`)

```
┌─────────────────────────────────────────────────────────┐
│                    CPU RENDERING                        │
├─────────────────────────────────────────────────────────┤
│  1. Generate height grid on CPU                         │
│  2. For each face:                                      │
│     - Project 4 corners via CPU trig                    │
│     - Calculate depth for sorting                       │
│  3. Sort all faces by depth (painter's algorithm)       │
│  4. Draw individual triangles via DrawTriangle          │
│     - Each triangle = separate draw call                │
└─────────────────────────────────────────────────────────┘

Performance: O(n²) for n×n grid, CPU-bound
Bottleneck: Per-face projection, sorting, draw calls
```

### Target Architecture (GPU-Accelerated)

```
┌─────────────────────────────────────────────────────────┐
│                    GPU RENDERING                        │
├─────────────────────────────────────────────────────────┤
│  SETUP (once):                                          │
│  1. Generate mesh geometry on CPU                       │
│  2. Upload vertex buffer (pos + normal + uv + id)       │
│  3. Upload index buffer (triangle indices)              │
│                                                         │
│  PER FRAME (fast):                                      │
│  1. Update 4×Vec4 transform matrix (64 bytes)           │
│  2. GPU vertex shader: transform + lighting             │
│  3. GPU fragment shader: colormaps + specular           │
│  4. Hardware depth buffer: automatic sorting            │
└─────────────────────────────────────────────────────────┘

Performance: O(1) per frame, GPU-bound
Bottleneck: Fill rate (pixels), not geometry
```

### Performance Comparison

| Metric | Current (CPU) | Target (GPU) | Improvement |
|--------|---------------|--------------|-------------|
| 25×25 grid | ~5ms | ~0.1ms | 50× |
| 100×100 grid | ~80ms | ~0.2ms | 400× |
| 500×500 grid | >1s (unusable) | ~1ms | 1000× |
| Memory bandwidth | High (per-frame) | Low (transform only) | 100× |

---

## 3. GPU Shader Pipeline

### Shader Components

```rust
// Three-tier shader architecture from URDF player

// 1. DrawMesh3D - Primary 3D object rendering
#[derive(Live, LiveHook, LiveRegister)]
#[repr(C)]
pub struct DrawMesh3D {
    #[deref] draw_super: DrawQuad,
    #[live] geometry_mesh: GeometryMesh3D,

    // Transform (4 column vectors = 64 bytes)
    #[live] transform_col0: Vec4,
    #[live] transform_col1: Vec4,
    #[live] transform_col2: Vec4,
    #[live] transform_col3: Vec4,

    // Material
    #[live] color: Vec4,
    #[live] bottom_color: Vec4,
    #[live] shininess: f32,
    #[live] specular_strength: f32,

    // Camera
    #[live] camera_pos: Vec3,

    // Clipping
    #[live] draw_clip: Vec4,
}

// 2. DrawGrid3D - Ground plane / reference grid
#[derive(Live, LiveHook, LiveRegister)]
pub struct DrawGrid3D {
    #[deref] draw_super: DrawQuad,
    #[live] grid_size: f32,
    #[live] grid_color: Vec4,
    #[live] line_width: f32,
}

// 3. DrawSkybox - Background gradient
#[derive(Live, LiveHook, LiveRegister)]
pub struct DrawSkybox {
    #[deref] draw_super: DrawQuad,
    #[live] top_color: Vec4,
    #[live] bottom_color: Vec4,
}
```

### Vertex Shader (GLSL-like Makepad DSL)

```glsl
fn vertex(self) -> vec4 {
    // 1. Reconstruct transform matrix from column vectors
    let col0 = self.transform_col0;
    let col1 = self.transform_col1;
    let col2 = self.transform_col2;
    let col3 = self.transform_col3;

    // 2. Transform position: M * v
    let pos_in = self.geom_pos;
    let pos = vec3(
        col0.x * pos_in.x + col1.x * pos_in.y + col2.x * pos_in.z + col3.x,
        col0.y * pos_in.x + col1.y * pos_in.y + col2.y * pos_in.z + col3.y,
        col0.z * pos_in.x + col1.z * pos_in.y + col2.z * pos_in.z + col3.z
    );

    // 3. Transform normal (rotation only, no translation)
    let normal_in = self.geom_normal;
    let normal = vec3(
        col0.x * normal_in.x + col1.x * normal_in.y + col2.x * normal_in.z,
        col0.y * normal_in.x + col1.y * normal_in.y + col2.y * normal_in.z,
        col0.z * normal_in.x + col1.z * normal_in.y + col2.z * normal_in.z
    );

    // 4. Compute Lambertian diffuse lighting (per-vertex for performance)
    let light_dir = normalize(vec3(0.3, 0.8, 0.5));
    let n = normalize(normal);
    let diff = max(0.0, dot(n, light_dir));
    let ambient = 0.4;
    let diffuse_brightness = ambient + diff * 0.6;

    // 5. Two-tone shading (darker undersides)
    let bottom_blend = max(0.0, -n.y);
    let base_color = mix(self.color.xyz, self.bottom_color.xyz, bottom_blend);

    // 6. Store pre-computed diffuse for fragment shader
    self.lit_color = vec4(base_color * diffuse_brightness, 1.0);
    self.world_pos = pos;
    self.world_normal = normal;

    // 7. Projection (orthographic with depth)
    let scale = 4.0;
    let depth = clamp((pos.z + 2.0) * 0.2, 0.1, 0.9);

    return vec4(pos.x * scale, pos.y * scale, depth, 1.0);
}
```

### Fragment Shader

```glsl
fn pixel(self) -> vec4 {
    // 1. Viewport clipping
    if self.clip_pos.x < self.draw_clip.x || self.clip_pos.x > self.draw_clip.z ||
       self.clip_pos.y < self.draw_clip.y || self.clip_pos.y > self.draw_clip.w {
        return vec4(0.0, 0.0, 0.0, 0.0);  // Discard
    }

    // 2. Blinn-Phong specular (per-pixel for quality)
    let light_dir = normalize(vec3(0.3, 0.8, 0.5));
    let view_dir = normalize(self.camera_pos - self.world_pos);
    let normal = normalize(self.world_normal);

    let halfway = normalize(light_dir + view_dir);
    let spec_angle = max(dot(normal, halfway), 0.0);
    let specular = pow(spec_angle, self.shininess) * self.specular_strength;

    // 3. Combine diffuse + specular
    let final_color = self.lit_color.xyz + vec3(specular, specular, specular);

    return vec4(final_color, 1.0);
}
```

---

## 4. Vertex Data Layout

### Interleaved Vertex Format

```rust
const FLOATS_PER_VERTEX: usize = 9;

// Memory layout per vertex (36 bytes):
// ┌──────────────────────────────────────────────────────┐
// │ pos.x │ pos.y │ pos.z │ id │ nx │ ny │ nz │ u │ v   │
// │ f32   │ f32   │ f32   │f32 │f32 │f32 │f32 │f32│f32  │
// └──────────────────────────────────────────────────────┘
//   0       1       2       3    4    5    6    7   8

pub struct MeshData {
    pub vertices: Vec<f32>,     // Interleaved vertex data
    pub indices: Vec<u32>,      // Triangle indices
    pub bounds_min: [f32; 3],   // AABB for culling
    pub bounds_max: [f32; 3],
}
```

### GeometryFields Trait

```rust
impl GeometryFields for GeometryMesh3D {
    fn geometry_fields(&self, fields: &mut Vec<GeometryField>) {
        fields.push(GeometryField {
            id: live_id!(geom_pos),
            ty: ShaderTy::Vec3
        });
        fields.push(GeometryField {
            id: live_id!(geom_id),
            ty: ShaderTy::Float
        });
        fields.push(GeometryField {
            id: live_id!(geom_normal),
            ty: ShaderTy::Vec3
        });
        fields.push(GeometryField {
            id: live_id!(geom_uv),
            ty: ShaderTy::Vec2
        });
    }
}
```

### Extended Layout for Data Visualization

```rust
// Extended layout for D3 visualizations (13 floats = 52 bytes):
const FLOATS_PER_VIS_VERTEX: usize = 13;

// ┌──────────────────────────────────────────────────────────────────┐
// │ pos(3) │ id(1) │ normal(3) │ uv(2) │ data_value(1) │ category(1)│ flags(2) │
// └──────────────────────────────────────────────────────────────────┘

// Enables:
// - data_value: for colormap lookup in shader
// - category: for categorical coloring
// - flags: selection state, hover state, etc.
```

---

## 5. Transform System

### Transform Matrix as Column Vectors

The key optimization: send only 64 bytes per object per frame instead of re-uploading geometry.

```rust
pub struct Transform3D {
    // Column-major 4x4 matrix stored as 4 Vec4s
    col0: Vec4,  // First column (x-axis + scale)
    col1: Vec4,  // Second column (y-axis + scale)
    col2: Vec4,  // Third column (z-axis + scale)
    col3: Vec4,  // Fourth column (translation)
}

impl Transform3D {
    /// Convert from glam::Mat4 to Makepad-compatible format
    pub fn from_glam(m: &glam::Mat4) -> Self {
        let cols = m.to_cols_array();
        Self {
            col0: vec4(cols[0], cols[1], cols[2], cols[3]),
            col1: vec4(cols[4], cols[5], cols[6], cols[7]),
            col2: vec4(cols[8], cols[9], cols[10], cols[11]),
            col3: vec4(cols[12], cols[13], cols[14], cols[15]),
        }
    }

    /// Apply to DrawMesh3D
    pub fn apply_to(&self, draw: &mut DrawMesh3D) {
        draw.transform_col0 = self.col0;
        draw.transform_col1 = self.col1;
        draw.transform_col2 = self.col2;
        draw.transform_col3 = self.col3;
    }
}
```

### Transform Composition for Data Visualization

```rust
// Example: 3D Bar Chart transform pipeline
fn compute_bar_transform(
    bar_index: usize,
    value: f64,
    camera: &Camera3D,
    layout: &BarLayout,
) -> Transform3D {
    // 1. Scale bar by value (height)
    let scale = glam::Mat4::from_scale(glam::Vec3::new(
        layout.bar_width,
        value as f32,  // Height from data
        layout.bar_depth,
    ));

    // 2. Position in grid
    let (row, col) = layout.index_to_grid(bar_index);
    let translation = glam::Mat4::from_translation(glam::Vec3::new(
        col as f32 * layout.spacing_x,
        value as f32 / 2.0,  // Center vertically
        row as f32 * layout.spacing_z,
    ));

    // 3. Apply camera view
    let model = translation * scale;
    let view = camera.view_matrix();
    let projection = camera.projection_matrix(layout.aspect_ratio);

    // 4. Combined MVP matrix
    Transform3D::from_glam(&(projection * view * model))
}
```

---

## 6. Lighting Model

### Two-Stage Hybrid Phong

| Stage | Computation | Location | Purpose |
|-------|-------------|----------|---------|
| **Diffuse** | Lambertian | Vertex shader | Performance (shared across fragments) |
| **Specular** | Blinn-Phong | Fragment shader | Quality (per-pixel precision) |

### Lighting Parameters

```rust
pub struct LightingConfig {
    // Directional light
    pub light_direction: Vec3,  // Default: (0.3, 0.8, 0.5) normalized

    // Ambient
    pub ambient_strength: f32,  // Default: 0.4

    // Diffuse
    pub diffuse_strength: f32,  // Default: 0.6

    // Specular (Blinn-Phong)
    pub specular_strength: f32, // Default: 0.5
    pub shininess: f32,         // Default: 32.0

    // Two-tone (bottom shading)
    pub bottom_color_factor: f32, // Default: 0.5 (50% darker undersides)
}

impl Default for LightingConfig {
    fn default() -> Self {
        Self {
            light_direction: vec3(0.3, 0.8, 0.5).normalize(),
            ambient_strength: 0.4,
            diffuse_strength: 0.6,
            specular_strength: 0.5,
            shininess: 32.0,
            bottom_color_factor: 0.5,
        }
    }
}
```

### Colormap Integration in Fragment Shader

```glsl
fn pixel(self) -> vec4 {
    // Get data value from vertex attribute
    let t = self.geom_data_value;  // 0.0 to 1.0

    // Viridis colormap (can be swapped via uniform)
    let base_color = viridis(t);

    // Apply lighting to colormap color
    let lit = base_color * self.diffuse_brightness;
    let final_color = lit + vec3(self.specular, self.specular, self.specular);

    return vec4(final_color, 1.0);
}

// Viridis colormap approximation
fn viridis(t: f32) -> vec3 {
    let c0 = vec3(0.267, 0.004, 0.329);
    let c1 = vec3(0.282, 0.140, 0.458);
    let c2 = vec3(0.254, 0.265, 0.530);
    let c3 = vec3(0.163, 0.471, 0.558);
    let c4 = vec3(0.134, 0.658, 0.517);
    let c5 = vec3(0.477, 0.821, 0.318);
    let c6 = vec3(0.993, 0.906, 0.144);

    if t < 0.167 { return mix(c0, c1, t / 0.167); }
    if t < 0.333 { return mix(c1, c2, (t - 0.167) / 0.166); }
    if t < 0.5   { return mix(c2, c3, (t - 0.333) / 0.167); }
    if t < 0.667 { return mix(c3, c4, (t - 0.5) / 0.167); }
    if t < 0.833 { return mix(c4, c5, (t - 0.667) / 0.166); }
    return mix(c5, c6, (t - 0.833) / 0.167);
}
```

---

## 7. Camera System

### Orbital Camera

```rust
pub struct Camera3D {
    // Orbital parameters
    pub distance: f64,      // Distance from target
    pub yaw: f64,           // Rotation around Y axis (radians)
    pub pitch: f64,         // Rotation around X axis (radians, clamped)
    pub target: DVec3,      // Look-at point

    // Pan offset
    pub pan_x: f64,
    pub pan_y: f64,

    // Projection
    pub fov: f64,           // Vertical FOV in radians
    pub near: f64,          // Near clip plane
    pub far: f64,           // Far clip plane
}

impl Camera3D {
    pub fn new() -> Self {
        Self {
            distance: 5.0,
            yaw: 0.3,
            pitch: 0.5,
            target: DVec3::ZERO,
            pan_x: 0.0,
            pan_y: 0.0,
            fov: std::f64::consts::FRAC_PI_4,  // 45 degrees
            near: 0.01,
            far: 100.0,
        }
    }

    /// Get camera position in world space
    pub fn position(&self) -> DVec3 {
        let x = self.distance * self.pitch.cos() * self.yaw.sin();
        let y = self.distance * self.pitch.sin();
        let z = self.distance * self.pitch.cos() * self.yaw.cos();
        self.target + DVec3::new(x, y, z)
    }

    /// Compute view matrix
    pub fn view_matrix(&self) -> glam::Mat4 {
        let pos = self.position().as_vec3();
        let target = self.target.as_vec3();
        glam::Mat4::look_at_rh(pos, target, glam::Vec3::Y)
    }

    /// Compute projection matrix
    pub fn projection_matrix(&self, aspect_ratio: f32) -> glam::Mat4 {
        glam::Mat4::perspective_rh(
            self.fov as f32,
            aspect_ratio,
            self.near as f32,
            self.far as f32,
        )
    }

    /// Combined view-projection with pan
    pub fn view_projection(&self, aspect_ratio: f32) -> glam::Mat4 {
        let base_rot = glam::Mat4::from_rotation_x(-std::f32::consts::FRAC_PI_2);
        let orbital = glam::Mat4::from_euler(
            glam::EulerRot::YXZ,
            self.yaw as f32,
            self.pitch as f32,
            0.0,
        );
        let scale = glam::Mat4::from_scale(glam::Vec3::splat(1.0 / self.distance as f32));
        let pan = glam::Mat4::from_translation(glam::Vec3::new(
            self.pan_x as f32,
            self.pan_y as f32,
            0.0,
        ));

        pan * scale * orbital * base_rot
    }
}
```

### Mouse Interaction Handling

```rust
impl Camera3D {
    pub fn handle_drag(&mut self, delta: DVec2, modifiers: KeyModifiers) {
        if modifiers.shift {
            // Pan mode
            let pan_speed = 0.003 * self.distance;
            self.pan_x += delta.x * pan_speed;
            self.pan_y -= delta.y * pan_speed;
        } else {
            // Orbit mode
            self.yaw += delta.x * 0.01;
            self.pitch += delta.y * 0.01;
            self.pitch = self.pitch.clamp(-1.4, 1.4);  // Prevent gimbal lock
        }
    }

    pub fn handle_scroll(&mut self, scroll: DVec2) {
        let zoom_factor = 1.0 - scroll.y * 0.01;
        self.distance *= zoom_factor;
        self.distance = self.distance.clamp(0.1, 50.0);
    }

    pub fn handle_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Equals => {
                self.distance *= 0.9;
                self.distance = self.distance.clamp(0.2, 50.0);
            }
            KeyCode::Minus => {
                self.distance *= 1.1;
                self.distance = self.distance.clamp(0.2, 50.0);
            }
            KeyCode::Home => {
                // Reset to default view
                *self = Self::new();
            }
            _ => {}
        }
    }
}
```

---

## 8. Mesh Generation

### Procedural Surface Mesh

```rust
impl MeshData {
    /// Create surface mesh from height function
    pub fn surface_from_function<F>(
        resolution: usize,
        x_range: (f32, f32),
        z_range: (f32, f32),
        height_fn: F,
    ) -> Self
    where
        F: Fn(f32, f32) -> f32,
    {
        let mut vertices = Vec::with_capacity(resolution * resolution * FLOATS_PER_VERTEX);
        let mut indices = Vec::with_capacity((resolution - 1) * (resolution - 1) * 6);

        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;

        // Generate vertices
        for i in 0..resolution {
            for j in 0..resolution {
                let u = i as f32 / (resolution - 1) as f32;
                let v = j as f32 / (resolution - 1) as f32;

                let x = x_range.0 + (x_range.1 - x_range.0) * u;
                let z = z_range.0 + (z_range.1 - z_range.0) * v;
                let y = height_fn(x, z);

                min_y = min_y.min(y);
                max_y = max_y.max(y);

                // Position
                vertices.push(x);
                vertices.push(y);
                vertices.push(z);

                // ID (for selection)
                vertices.push((i * resolution + j) as f32);

                // Normal (computed later)
                vertices.push(0.0);
                vertices.push(1.0);
                vertices.push(0.0);

                // UV
                vertices.push(u);
                vertices.push(v);
            }
        }

        // Generate indices (two triangles per quad)
        for i in 0..(resolution - 1) {
            for j in 0..(resolution - 1) {
                let idx = (i * resolution + j) as u32;

                // First triangle
                indices.push(idx);
                indices.push(idx + resolution as u32);
                indices.push(idx + 1);

                // Second triangle
                indices.push(idx + 1);
                indices.push(idx + resolution as u32);
                indices.push(idx + resolution as u32 + 1);
            }
        }

        let mut mesh = Self {
            vertices,
            indices,
            bounds_min: [x_range.0, min_y, z_range.0],
            bounds_max: [x_range.1, max_y, z_range.1],
        };

        // Compute smooth normals
        mesh.compute_normals();

        mesh
    }

    /// Compute smooth vertex normals from face normals
    fn compute_normals(&mut self) {
        let vertex_count = self.vertices.len() / FLOATS_PER_VERTEX;
        let mut normals = vec![[0.0f32; 3]; vertex_count];

        // Accumulate face normals
        for i in (0..self.indices.len()).step_by(3) {
            let i0 = self.indices[i] as usize;
            let i1 = self.indices[i + 1] as usize;
            let i2 = self.indices[i + 2] as usize;

            let p0 = self.get_position(i0);
            let p1 = self.get_position(i1);
            let p2 = self.get_position(i2);

            let edge1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
            let edge2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];

            // Cross product
            let normal = [
                edge1[1] * edge2[2] - edge1[2] * edge2[1],
                edge1[2] * edge2[0] - edge1[0] * edge2[2],
                edge1[0] * edge2[1] - edge1[1] * edge2[0],
            ];

            // Accumulate
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
            if len > 0.0001 {
                let base = i * FLOATS_PER_VERTEX + 4;  // Normal offset
                self.vertices[base] = n[0] / len;
                self.vertices[base + 1] = n[1] / len;
                self.vertices[base + 2] = n[2] / len;
            }
        }
    }

    fn get_position(&self, index: usize) -> [f32; 3] {
        let base = index * FLOATS_PER_VERTEX;
        [
            self.vertices[base],
            self.vertices[base + 1],
            self.vertices[base + 2],
        ]
    }
}
```

### Primitive Shapes

```rust
impl MeshData {
    /// Unit cube centered at origin
    pub fn cube(size: f32) -> Self {
        // 24 vertices (4 per face for flat shading)
        // 36 indices (6 faces × 2 triangles × 3 vertices)
        // ... implementation
    }

    /// Sphere with lat/lon tessellation
    pub fn sphere(radius: f32, lat_segments: usize, lon_segments: usize) -> Self {
        // ... implementation
    }

    /// Cylinder along Y axis
    pub fn cylinder(radius: f32, height: f32, segments: usize) -> Self {
        // ... implementation
    }

    /// XZ plane grid
    pub fn grid_plane(size: f32, divisions: usize) -> Self {
        // ... implementation
    }
}
```

---

## 9. Depth Handling

### Linear Depth Mapping

```glsl
// Vertex shader depth calculation
fn vertex(self) -> vec4 {
    // ... transform position ...

    // Map world Z to normalized device coordinates
    // Assumes scene fits in Z range [-2, 8]
    let raw_depth = (pos.z + 2.0) * 0.2;  // Map to [0, 2]
    let depth = clamp(raw_depth, 0.1, 0.9);  // Leave headroom

    return vec4(screen_x, screen_y, depth, 1.0);
}
```

### Draw Order Strategy

For scenes requiring explicit draw order:

```rust
pub enum DrawOrder {
    /// Use GPU depth buffer (automatic)
    DepthBuffer,

    /// Painter's algorithm (far to near)
    BackToFront,

    /// Transparency ordering (near to far for alpha)
    FrontToBack,
}

fn draw_scene(&mut self, cx: &mut Cx2d, order: DrawOrder) {
    match order {
        DrawOrder::DepthBuffer => {
            // Just draw everything, GPU handles depth
            for object in &self.objects {
                object.draw(cx);
            }
        }
        DrawOrder::BackToFront => {
            // Sort by camera distance (for transparency)
            let mut sorted: Vec<_> = self.objects.iter().collect();
            sorted.sort_by(|a, b| {
                b.distance_to_camera(&self.camera)
                    .partial_cmp(&a.distance_to_camera(&self.camera))
                    .unwrap()
            });
            for object in sorted {
                object.draw(cx);
            }
        }
        DrawOrder::FrontToBack => {
            // Reverse for early-Z optimization
            // ...
        }
    }
}
```

---

## 10. Implementation Patterns

### Pattern 1: Geometry Template

Create geometry once, instance many times:

```rust
pub struct Chart3DRenderer {
    // Shared geometry templates
    bar_template: GeometryMesh3D,
    sphere_template: GeometryMesh3D,

    // Per-instance drawers
    drawers: Vec<DrawMesh3D>,
}

impl Chart3DRenderer {
    pub fn new(cx: &mut Cx) -> Self {
        Self {
            bar_template: GeometryMesh3D::from_mesh(MeshData::cube(1.0)),
            sphere_template: GeometryMesh3D::from_mesh(MeshData::sphere(1.0, 16, 16)),
            drawers: Vec::new(),
        }
    }

    pub fn draw_bar_chart(&mut self, cx: &mut Cx2d, data: &[f64], camera: &Camera3D) {
        // Ensure enough drawers
        while self.drawers.len() < data.len() {
            self.drawers.push(DrawMesh3D::new_with_geometry(
                cx,
                self.bar_template.clone(),
            ));
        }

        // Update and draw each bar
        for (i, &value) in data.iter().enumerate() {
            let transform = self.compute_bar_transform(i, value, camera);
            let color = self.value_to_color(value);

            self.drawers[i].set_transform(&transform);
            self.drawers[i].color = color;
            self.drawers[i].draw(cx);
        }
    }
}
```

### Pattern 2: Dynamic Mesh Updates

For animated surfaces:

```rust
impl SurfacePlot3D {
    pub fn update_surface<F>(&mut self, cx: &mut Cx, time: f64, height_fn: F)
    where
        F: Fn(f32, f32, f64) -> f32,
    {
        // Only update vertex Y values and normals, not full mesh
        let resolution = self.resolution;

        for i in 0..resolution {
            for j in 0..resolution {
                let u = i as f32 / (resolution - 1) as f32;
                let v = j as f32 / (resolution - 1) as f32;

                let x = self.x_range.0 + (self.x_range.1 - self.x_range.0) * u;
                let z = self.z_range.0 + (self.z_range.1 - self.z_range.0) * v;
                let y = height_fn(x, z, time);

                // Update Y in vertex buffer
                let vertex_idx = i * resolution + j;
                let base = vertex_idx * FLOATS_PER_VERTEX;
                self.mesh.vertices[base + 1] = y;  // Y position
            }
        }

        // Recompute normals
        self.mesh.compute_normals();

        // Re-upload to GPU
        self.geometry.update(cx, &self.mesh);
    }
}
```

### Pattern 3: Selection and Highlighting

```rust
impl Chart3DInteraction {
    pub fn handle_hit_test(&mut self, cx: &Cx, pos: DVec2) -> Option<usize> {
        // Read back from geom_id in shader
        // (Requires render-to-texture with ID buffer)

        // Simpler: ray-box intersection on CPU
        let ray = self.camera.screen_to_ray(pos);

        for (i, bounds) in self.object_bounds.iter().enumerate() {
            if ray.intersects_aabb(bounds) {
                return Some(i);
            }
        }

        None
    }

    pub fn set_selection(&mut self, index: Option<usize>) {
        // Update shader uniforms for selected object
        for (i, drawer) in self.drawers.iter_mut().enumerate() {
            if Some(i) == index {
                drawer.specular_strength = 1.0;  // Highlight
                drawer.color = self.highlight_color;
            } else {
                drawer.specular_strength = 0.3;
                drawer.color = self.normal_colors[i];
            }
        }
    }
}
```

---

## 11. Performance Considerations

### Memory Budgets

| Component | Per-Vertex | Per-Object | Per-Frame |
|-----------|------------|------------|-----------|
| Vertex data | 36 bytes | - | 0 (static) |
| Index data | 4 bytes | - | 0 (static) |
| Transform | - | 64 bytes | 64 bytes |
| Uniforms | - | ~128 bytes | ~128 bytes |

### Optimization Checklist

- [ ] **Geometry Upload**: Once at initialization, not per-frame
- [ ] **Instancing**: Reuse geometry templates with different transforms
- [ ] **Culling**: Skip objects outside camera frustum
- [ ] **LOD**: Reduce tessellation for distant objects
- [ ] **Batching**: Combine multiple objects into single draw call where possible
- [ ] **Depth Pre-pass**: For complex scenes with transparency

### Benchmarking

```rust
pub struct RenderStats {
    pub frame_time_ms: f64,
    pub transform_time_ms: f64,
    pub draw_call_count: usize,
    pub triangle_count: usize,
    pub vertex_count: usize,
}

impl RenderStats {
    pub fn print_summary(&self) {
        println!(
            "[Frame] {:.2}ms | {} draws | {}K tris | {:.0} FPS",
            self.frame_time_ms,
            self.draw_call_count,
            self.triangle_count / 1000,
            1000.0 / self.frame_time_ms,
        );
    }
}
```

---

## Summary

This architecture enables:

1. **60+ FPS** with 100K+ triangles
2. **Transform-only updates** for animations (64 bytes/object/frame)
3. **GPU-accelerated lighting** with customizable materials
4. **Orbital camera** with intuitive mouse/keyboard controls
5. **Colormap integration** in fragment shaders
6. **Selection/highlighting** via shader uniforms
7. **Modular design** with reusable geometry templates

The key insight from the URDF player is that 3D rendering should be structured as:
- **Setup phase**: Generate and upload geometry once
- **Render phase**: Update transforms and uniforms only

This separates the expensive (geometry) from the cheap (transforms), enabling smooth 60fps rendering even with complex scenes.

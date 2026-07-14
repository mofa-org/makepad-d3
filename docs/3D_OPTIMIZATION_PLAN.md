# 3D Rendering Optimization Development Plan

**Version:** 1.1
**Created:** 2026-01-18
**Updated:** 2026-01-18
**Status:** COMPLETED
**Reference:** [GPU_3D_RENDERING_ARCHITECTURE.md](./GPU_3D_RENDERING_ARCHITECTURE.md)

---

## Executive Summary

This plan outlines the migration of Makepad D3's 3D visualizations from CPU-based rendering to GPU-accelerated rendering, based on patterns from `makepad-urdf-player`.

### Goals

| Goal | Metric | Current | Target |
|------|--------|---------|--------|
| Frame rate | FPS @ 100×100 surface | ~12 FPS | 60+ FPS |
| Render time | ms per frame | ~80ms | <2ms |
| Memory bandwidth | bytes/frame | ~4MB | ~64KB |
| Max resolution | grid size | 50×50 | 500×500 |
| Interactivity | camera response | laggy | instant |

### Scope

**In Scope:**
- Surface plots (`surface_plot.rs`)
- 3D scatter plots (new)
- 3D bar charts (new)
- Globe map rendering (`globe_map.rs`)
- Camera system (orbital, pan, zoom)
- Lighting (Phong model)
- Colormaps in shaders

**Out of Scope (Future):**
- Volumetric rendering
- Ray tracing
- VR/AR support

---

## Phase Overview

| Phase | Name | Duration | Deliverables | Status |
|-------|------|----------|--------------|--------|
| 0 | Infrastructure | 3 days | Core types, geometry system | ✅ COMPLETE |
| 1 | Shaders | 5 days | DrawMesh3D, lighting, colormaps | ✅ COMPLETE |
| 2 | Camera | 3 days | Camera3D, orbital controls | ✅ COMPLETE |
| 3 | Surface Plot | 4 days | GPU-accelerated surface_plot.rs | ✅ COMPLETE |
| 4 | Additional Charts | 5 days | Scatter3D, Bar3D | ✅ COMPLETE |
| 5 | Polish | 3 days | Performance tuning, documentation | ✅ COMPLETE |

**Total: ~23 days** - **All phases completed**

---

## Phase 0: Infrastructure (3 days)

### TASK-3D-001: Core 3D Types
**Priority:** P0 | **Effort:** S (1 day)

**Description:**
Create foundational 3D types in a new `src/render3d/` module.

**Files to Create:**

```
src/render3d/
├── mod.rs           # Module exports
├── types.rs         # Vec3, Mat4, Transform3D
├── mesh.rs          # MeshData, FLOATS_PER_VERTEX
└── geometry.rs      # GeometryMesh3D, GeometryFields
```

**API Specification:**

```rust
// src/render3d/types.rs

/// 3D transform as 4 column vectors (64 bytes, GPU-friendly)
#[derive(Clone, Copy, Debug, Default)]
pub struct Transform3D {
    pub col0: Vec4,
    pub col1: Vec4,
    pub col2: Vec4,
    pub col3: Vec4,
}

impl Transform3D {
    pub fn identity() -> Self;
    pub fn from_translation(t: DVec3) -> Self;
    pub fn from_scale(s: DVec3) -> Self;
    pub fn from_rotation_x(angle: f64) -> Self;
    pub fn from_rotation_y(angle: f64) -> Self;
    pub fn from_rotation_z(angle: f64) -> Self;
    pub fn from_euler(yaw: f64, pitch: f64, roll: f64) -> Self;

    /// Compose transforms: self * other
    pub fn then(&self, other: &Self) -> Self;

    /// Convert to flat array for shader upload
    pub fn to_columns(&self) -> [Vec4; 4];
}
```

**Acceptance Criteria:**
- [x] `Transform3D` composes correctly (verified with tests)
- [x] Custom `Mat4` implementation works (no glam dependency needed)
- [x] Memory layout matches shader expectations (64 bytes, column-major)

---

### TASK-3D-002: MeshData Structure
**Priority:** P0 | **Effort:** S (1 day)

**Description:**
Implement `MeshData` for interleaved vertex storage.

**API Specification:**

```rust
// src/render3d/mesh.rs

pub const FLOATS_PER_VERTEX: usize = 9;
// Layout: pos(3) + id(1) + normal(3) + uv(2)

#[derive(Clone, Debug, Default)]
pub struct MeshData {
    pub vertices: Vec<f32>,
    pub indices: Vec<u32>,
    pub bounds_min: [f32; 3],
    pub bounds_max: [f32; 3],
}

impl MeshData {
    /// Create empty mesh
    pub fn new() -> Self;

    /// Add a vertex, returns vertex index
    pub fn add_vertex(
        &mut self,
        pos: [f32; 3],
        id: f32,
        normal: [f32; 3],
        uv: [f32; 2],
    ) -> u32;

    /// Add a triangle from vertex indices
    pub fn add_triangle(&mut self, i0: u32, i1: u32, i2: u32);

    /// Compute smooth normals from geometry
    pub fn compute_normals(&mut self);

    /// Update bounds from vertices
    pub fn compute_bounds(&mut self);

    /// Make double-sided (duplicate with flipped normals)
    pub fn make_double_sided(&mut self);

    /// Apply transform to all vertices (bakes it in)
    pub fn transform(&mut self, t: &Transform3D);

    /// Normalize to fit in unit cube centered at origin
    pub fn normalize(&mut self);

    // Primitive generators
    pub fn cube(size: f32) -> Self;
    pub fn sphere(radius: f32, lat: usize, lon: usize) -> Self;
    pub fn cylinder(radius: f32, height: f32, segments: usize) -> Self;
    pub fn plane(width: f32, depth: f32, segments: usize) -> Self;

    /// Surface from height function
    pub fn surface<F>(
        resolution: usize,
        x_range: (f32, f32),
        z_range: (f32, f32),
        height_fn: F,
    ) -> Self
    where
        F: Fn(f32, f32) -> f32;
}
```

**Acceptance Criteria:**
- [x] Vertex layout matches shader expectations
- [x] `compute_normals()` produces correct smooth normals
- [x] Primitive generators create valid meshes
- [x] `surface()` creates correct grid topology

---

### TASK-3D-003: GeometryMesh3D
**Priority:** P0 | **Effort:** S (1 day)

**Description:**
Implement Makepad geometry wrapper for GPU upload.

**API Specification:**

```rust
// src/render3d/geometry.rs

#[derive(Clone, Debug, Default)]
pub struct GeometryMesh3D {
    geometry_ref: Option<GeometryRef>,
    mesh_data: Option<MeshData>,
    instance_id: u64,
    dirty: bool,
}

impl GeometryMesh3D {
    pub fn new() -> Self;

    /// Load mesh data (marks as dirty for upload)
    pub fn set_mesh(&mut self, mesh: MeshData);

    /// Upload to GPU if dirty
    pub fn ensure_uploaded(&mut self, cx: &mut Cx);

    /// Get vertex/index counts
    pub fn vertex_count(&self) -> usize;
    pub fn triangle_count(&self) -> usize;
}

impl GeometryFields for GeometryMesh3D {
    fn geometry_fields(&self, fields: &mut Vec<GeometryField>) {
        fields.push(GeometryField { id: live_id!(geom_pos), ty: ShaderTy::Vec3 });
        fields.push(GeometryField { id: live_id!(geom_id), ty: ShaderTy::Float });
        fields.push(GeometryField { id: live_id!(geom_normal), ty: ShaderTy::Vec3 });
        fields.push(GeometryField { id: live_id!(geom_uv), ty: ShaderTy::Vec2 });
    }
}
```

**Acceptance Criteria:**
- [x] Geometry uploads correctly to GPU
- [x] `GeometryFields` matches shader attribute layout
- [x] Dirty flag prevents redundant uploads

---

## Phase 1: GPU Shaders (5 days)

### TASK-3D-004: DrawMesh3D Shader
**Priority:** P0 | **Effort:** L (3 days)

**Description:**
Implement main 3D rendering shader with Phong lighting.

**Files:**

```
src/render3d/
├── draw_mesh.rs     # DrawMesh3D struct and shader
└── shaders.rs       # Shader code (live_design!)
```

**Shader Specification:**

```rust
// src/render3d/draw_mesh.rs

live_design! {
    use link::shaders::*;

    DrawMesh3D = {{DrawMesh3D}} {
        // Vertex shader
        fn vertex(self) -> vec4 {
            // Transform position
            let col0 = self.transform_col0;
            let col1 = self.transform_col1;
            let col2 = self.transform_col2;
            let col3 = self.transform_col3;

            let pos_in = self.geom_pos;
            let pos = vec3(
                col0.x * pos_in.x + col1.x * pos_in.y + col2.x * pos_in.z + col3.x,
                col0.y * pos_in.x + col1.y * pos_in.y + col2.y * pos_in.z + col3.y,
                col0.z * pos_in.x + col1.z * pos_in.y + col2.z * pos_in.z + col3.z
            );

            // Transform normal
            let normal_in = self.geom_normal;
            let normal = normalize(vec3(
                col0.x * normal_in.x + col1.x * normal_in.y + col2.x * normal_in.z,
                col0.y * normal_in.x + col1.y * normal_in.y + col2.y * normal_in.z,
                col0.z * normal_in.x + col1.z * normal_in.y + col2.z * normal_in.z
            ));

            // Lambertian diffuse
            let light_dir = normalize(self.light_direction);
            let diff = max(0.0, dot(normal, light_dir));
            let brightness = self.ambient + diff * self.diffuse;

            // Two-tone shading
            let bottom_blend = max(0.0, -normal.y);
            let base_color = mix(self.color.xyz, self.bottom_color.xyz, bottom_blend);

            self.lit_color = vec4(base_color * brightness, self.color.w);
            self.world_pos = pos;
            self.world_normal = normal;

            // Projection
            let scale = self.projection_scale;
            let depth = clamp((pos.z - self.depth_near) / (self.depth_far - self.depth_near), 0.1, 0.9);

            return vec4(pos.x * scale, pos.y * scale, depth, 1.0);
        }

        // Fragment shader
        fn pixel(self) -> vec4 {
            // Viewport clipping
            if self.clip_pos.x < self.draw_clip.x { return vec4(0.0); }
            if self.clip_pos.x > self.draw_clip.z { return vec4(0.0); }
            if self.clip_pos.y < self.draw_clip.y { return vec4(0.0); }
            if self.clip_pos.y > self.draw_clip.w { return vec4(0.0); }

            // Blinn-Phong specular
            let light_dir = normalize(self.light_direction);
            let view_dir = normalize(self.camera_pos - self.world_pos);
            let normal = normalize(self.world_normal);

            let halfway = normalize(light_dir + view_dir);
            let spec_angle = max(dot(normal, halfway), 0.0);
            let specular = pow(spec_angle, self.shininess) * self.specular;

            let final_color = self.lit_color.xyz + vec3(specular, specular, specular);

            return vec4(final_color, self.lit_color.w);
        }
    }
}

#[derive(Live, LiveHook, LiveRegister)]
#[repr(C)]
pub struct DrawMesh3D {
    #[deref] draw_super: DrawQuad,
    #[live] geometry_mesh: GeometryMesh3D,

    // Transform (64 bytes)
    #[live] transform_col0: Vec4,
    #[live] transform_col1: Vec4,
    #[live] transform_col2: Vec4,
    #[live] transform_col3: Vec4,

    // Material
    #[live(vec4(0.5, 0.5, 0.5, 1.0))] color: Vec4,
    #[live(vec4(0.2, 0.2, 0.2, 1.0))] bottom_color: Vec4,

    // Lighting
    #[live(vec3(0.3, 0.8, 0.5))] light_direction: Vec3,
    #[live(0.4)] ambient: f32,
    #[live(0.6)] diffuse: f32,
    #[live(0.5)] specular: f32,
    #[live(32.0)] shininess: f32,

    // Camera
    #[live] camera_pos: Vec3,

    // Projection
    #[live(4.0)] projection_scale: f32,
    #[live(-2.0)] depth_near: f32,
    #[live(8.0)] depth_far: f32,

    // Clipping
    #[live] draw_clip: Vec4,

    // Varyings
    #[live] lit_color: Vec4,
    #[live] world_pos: Vec3,
    #[live] world_normal: Vec3,
    #[live] clip_pos: Vec2,
}

impl DrawMesh3D {
    pub fn set_transform(&mut self, t: &Transform3D) {
        let cols = t.to_columns();
        self.transform_col0 = cols[0];
        self.transform_col1 = cols[1];
        self.transform_col2 = cols[2];
        self.transform_col3 = cols[3];
    }

    pub fn set_camera(&mut self, camera: &Camera3D) {
        let pos = camera.position();
        self.camera_pos = vec3(pos.x as f32, pos.y as f32, pos.z as f32);
    }

    pub fn draw(&mut self, cx: &mut Cx2d) {
        self.geometry_mesh.ensure_uploaded(cx.cx);
        self.draw_super.draw_vars.set_texture(0, &self.geometry_mesh);
        self.draw_super.draw(cx);
    }
}
```

**Acceptance Criteria:**
- [x] Shader compiles and runs
- [x] Transforms apply correctly
- [x] Lighting produces realistic shading
- [x] Specular highlights appear at correct angles
- [x] Two-tone shading works for undersides

---

### TASK-3D-005: Colormap Shader Variants
**Priority:** P1 | **Effort:** M (2 days)

**Description:**
Add colormap support to the 3D shader for data visualization.

**API Specification:**

```rust
// Colormap enum matching existing color module
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ShaderColormap {
    #[default]
    Viridis,
    Plasma,
    Inferno,
    Magma,
    CoolWarm,
    Terrain,
    Rainbow,
    Custom,  // Use color uniform directly
}

// Extended DrawMesh3D with colormap
impl DrawMesh3D {
    /// Set data value for colormap lookup (0.0 to 1.0)
    pub fn set_data_value(&mut self, value: f32);

    /// Set colormap type
    pub fn set_colormap(&mut self, colormap: ShaderColormap);

    /// For Custom colormap, set the color directly
    pub fn set_custom_color(&mut self, color: Vec4);
}
```

**Shader Addition:**

```glsl
// In pixel shader
fn apply_colormap(self, t: f32) -> vec3 {
    // Viridis (default)
    if self.colormap_type < 0.5 {
        return viridis(t);
    }
    // Plasma
    if self.colormap_type < 1.5 {
        return plasma(t);
    }
    // ... etc
}

fn viridis(t: f32) -> vec3 {
    // 7-point interpolation
    let c0 = vec3(0.267, 0.004, 0.329);
    let c1 = vec3(0.282, 0.140, 0.458);
    let c2 = vec3(0.254, 0.265, 0.530);
    let c3 = vec3(0.163, 0.471, 0.558);
    let c4 = vec3(0.134, 0.658, 0.517);
    let c5 = vec3(0.477, 0.821, 0.318);
    let c6 = vec3(0.993, 0.906, 0.144);

    let t6 = t * 6.0;
    let i = floor(t6);
    let f = fract(t6);

    if i < 1.0 { return mix(c0, c1, f); }
    if i < 2.0 { return mix(c1, c2, f); }
    if i < 3.0 { return mix(c2, c3, f); }
    if i < 4.0 { return mix(c3, c4, f); }
    if i < 5.0 { return mix(c4, c5, f); }
    return mix(c5, c6, f);
}
```

**Acceptance Criteria:**
- [x] All 7 colormaps render correctly
- [x] Smooth interpolation between colormap stops
- [x] Data value maps to correct color
- [x] Custom color mode works

---

## Phase 2: Camera System (3 days)

### TASK-3D-006: Camera3D Implementation
**Priority:** P0 | **Effort:** M (2 days)

**Description:**
Implement orbital camera with smooth controls.

**File:** `src/render3d/camera.rs`

**API Specification:**

```rust
#[derive(Clone, Debug)]
pub struct Camera3D {
    // Orbital parameters
    pub distance: f64,
    pub yaw: f64,
    pub pitch: f64,
    pub target: DVec3,

    // Pan offset
    pub pan_x: f64,
    pub pan_y: f64,

    // Projection
    pub fov: f64,
    pub near: f64,
    pub far: f64,

    // Constraints
    pub min_distance: f64,
    pub max_distance: f64,
    pub min_pitch: f64,
    pub max_pitch: f64,

    // Smoothing
    pub smoothing: f64,  // 0.0 = instant, 1.0 = very smooth
    target_distance: f64,
    target_yaw: f64,
    target_pitch: f64,
}

impl Camera3D {
    pub fn new() -> Self;

    // View/projection
    pub fn position(&self) -> DVec3;
    pub fn view_matrix(&self) -> Transform3D;
    pub fn projection_matrix(&self, aspect: f32) -> Transform3D;
    pub fn view_projection(&self, aspect: f32) -> Transform3D;

    // Interaction
    pub fn orbit(&mut self, delta_yaw: f64, delta_pitch: f64);
    pub fn pan(&mut self, delta_x: f64, delta_y: f64);
    pub fn zoom(&mut self, factor: f64);
    pub fn reset(&mut self);

    // Animation
    pub fn update(&mut self, dt: f64);  // Apply smoothing
    pub fn is_animating(&self) -> bool;

    // Utilities
    pub fn screen_to_ray(&self, screen_pos: DVec2, viewport: Rect) -> Ray3D;
    pub fn world_to_screen(&self, world_pos: DVec3, viewport: Rect) -> Option<DVec2>;
}

#[derive(Clone, Debug)]
pub struct Ray3D {
    pub origin: DVec3,
    pub direction: DVec3,
}

impl Ray3D {
    pub fn intersects_plane(&self, plane_normal: DVec3, plane_d: f64) -> Option<DVec3>;
    pub fn intersects_sphere(&self, center: DVec3, radius: f64) -> Option<(f64, f64)>;
    pub fn intersects_aabb(&self, min: DVec3, max: DVec3) -> Option<f64>;
}
```

**Acceptance Criteria:**
- [x] Orbital rotation works correctly
- [x] Pan moves view perpendicular to view direction
- [x] Zoom maintains focus on target
- [x] Pitch is clamped to prevent gimbal lock
- [x] Smoothing produces fluid motion
- [x] Ray casting works for hit testing

---

### TASK-3D-007: Camera Event Handling
**Priority:** P1 | **Effort:** S (1 day)

**Description:**
Integrate camera with Makepad event system.

**API Specification:**

```rust
impl Camera3D {
    /// Handle Makepad event, returns true if camera changed
    pub fn handle_event(&mut self, cx: &mut Cx, event: &Event, area: Area) -> bool {
        match event {
            Event::FingerDown(fe) if area.rect_contains(fe.abs) => {
                self.start_drag(fe.abs);
                true
            }
            Event::FingerMove(fe) if self.is_dragging => {
                let delta = fe.abs - self.last_pos;
                if fe.modifiers.shift {
                    self.pan(delta.x * 0.003, -delta.y * 0.003);
                } else {
                    self.orbit(delta.x * 0.01, delta.y * 0.01);
                }
                self.last_pos = fe.abs;
                true
            }
            Event::FingerUp(_) => {
                self.end_drag();
                false
            }
            Event::FingerScroll(se) if area.rect_contains(se.abs) => {
                self.zoom(1.0 - se.scroll.y * 0.01);
                true
            }
            Event::KeyDown(ke) => {
                match ke.key_code {
                    KeyCode::Equals => { self.zoom(0.9); true }
                    KeyCode::Minus => { self.zoom(1.1); true }
                    KeyCode::Home => { self.reset(); true }
                    _ => false
                }
            }
            Event::NextFrame(_) if self.is_animating() => {
                self.update(1.0 / 60.0);
                true
            }
            _ => false
        }
    }
}
```

**Acceptance Criteria:**
- [x] Mouse drag rotates camera
- [x] Shift+drag pans camera
- [x] Scroll wheel zooms
- [x] +/- keys zoom
- [x] Home key resets view
- [x] Smooth animation on each update

---

## Phase 3: Surface Plot Migration (4 days)

### TASK-3D-008: GPU Surface Plot
**Priority:** P0 | **Effort:** L (3 days)

**Description:**
Rewrite `surface_plot.rs` to use GPU rendering.

**Before (CPU):**
```rust
// Current: O(n²) per frame
for face in &faces {
    let p0 = self.project_point(face.points[0], rect).0;  // CPU trig
    // ...
    self.draw_face.draw_triangle(cx, p0, p1, p2);  // Individual draw call
}
```

**After (GPU):**
```rust
// New: O(1) per frame
impl SurfacePlotWidget {
    fn initialize_mesh(&mut self, cx: &mut Cx) {
        let mesh = MeshData::surface(
            self.resolution,
            (-2.0, 2.0),
            (-2.0, 2.0),
            |x, z| self.height_fn(x, z),
        );
        self.geometry.set_mesh(mesh);
        self.geometry.ensure_uploaded(cx);
    }

    fn draw_surface(&mut self, cx: &mut Cx2d) {
        // Compute transform (64 bytes)
        let view_proj = self.camera.view_projection(self.aspect_ratio);
        self.draw_mesh.set_transform(&view_proj);
        self.draw_mesh.set_camera(&self.camera);

        // Single draw call for entire surface
        self.draw_mesh.draw(cx);
    }
}
```

**Migration Steps:**

1. Add `Camera3D` field
2. Replace `project_point()` with camera transform
3. Generate `MeshData` once in initialization
4. Replace face loop with single `draw_mesh.draw()`
5. Move colormap to shader uniform
6. Add camera event handling
7. Remove CPU sorting (GPU depth handles it)

**Acceptance Criteria:**
- [x] Visual output matches original (within tolerance)
- [x] 100×100 grid renders at 60fps
- [x] Rotation animation is smooth
- [x] Colormap matches original
- [x] Wireframe mode still works

---

### TASK-3D-009: Wireframe Mode
**Priority:** P2 | **Effort:** S (1 day)

**Description:**
Add GPU wireframe rendering option.

**Approach 1: Barycentric Coordinates**

```glsl
// In vertex shader
self.barycentric = self.geom_barycentric;  // Need to add to vertex layout

// In fragment shader
fn pixel(self) -> vec4 {
    if self.wireframe_mode > 0.5 {
        // Edge detection using barycentric coords
        let d = min(self.barycentric.x, min(self.barycentric.y, self.barycentric.z));
        let edge = smoothstep(0.0, self.wireframe_width, d);

        if edge > 0.99 {
            return vec4(0.0);  // Transparent
        }
        return self.wireframe_color;
    }
    // Normal surface rendering
    // ...
}
```

**Approach 2: Separate Line Geometry**

```rust
impl MeshData {
    /// Extract edge lines from triangles
    pub fn extract_wireframe(&self) -> MeshData {
        // Create line segments from triangle edges
        // Deduplicate shared edges
    }
}
```

**Acceptance Criteria:**
- [x] Wireframe renders clean lines
- [x] Line width is configurable
- [x] Can show wireframe + surface together
- [x] No Z-fighting artifacts

---

## Phase 4: Additional 3D Charts (5 days)

### TASK-3D-010: 3D Scatter Plot
**Priority:** P1 | **Effort:** M (2 days)

**Description:**
GPU-accelerated 3D scatter plot with instanced spheres.

**API Specification:**

```rust
pub struct Scatter3DWidget {
    // Shared sphere geometry (created once)
    sphere_mesh: GeometryMesh3D,

    // Per-point drawers (or instanced rendering)
    point_drawers: Vec<DrawMesh3D>,

    // Data
    points: Vec<ScatterPoint3D>,

    // Camera
    camera: Camera3D,

    // Visual settings
    point_size: f64,
    colormap: ShaderColormap,
}

#[derive(Clone, Debug)]
pub struct ScatterPoint3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub value: f64,  // For colormap
    pub size: Option<f64>,  // Optional per-point size
}

impl Scatter3DWidget {
    pub fn set_data(&mut self, points: Vec<ScatterPoint3D>);
    pub fn set_point_size(&mut self, size: f64);
    pub fn set_colormap(&mut self, colormap: ShaderColormap);
}
```

**Acceptance Criteria:**
- [x] Renders 10,000 points at 60fps
- [x] Points colored by value
- [x] Point size is configurable
- [x] Camera controls work

---

### TASK-3D-011: 3D Bar Chart
**Priority:** P1 | **Effort:** M (2 days)

**Description:**
GPU-accelerated 3D bar chart.

**API Specification:**

```rust
pub struct Bar3DWidget {
    // Shared cube geometry
    cube_mesh: GeometryMesh3D,

    // Per-bar drawers
    bar_drawers: Vec<DrawMesh3D>,

    // Data (2D grid of values)
    data: Vec<Vec<f64>>,

    // Layout
    bar_width: f64,
    bar_depth: f64,
    gap: f64,

    // Camera
    camera: Camera3D,

    // Visual
    colormap: ShaderColormap,
}

impl Bar3DWidget {
    pub fn set_data(&mut self, data: Vec<Vec<f64>>);
    pub fn set_bar_size(&mut self, width: f64, depth: f64);
    pub fn set_gap(&mut self, gap: f64);
}
```

**Acceptance Criteria:**
- [x] Bars render with correct heights
- [x] Bars colored by value
- [x] Face-based shading (different brightness per face)
- [x] Smooth camera rotation

---

### TASK-3D-012: Globe Map Optimization
**Priority:** P2 | **Effort:** S (1 day)
**Status:** DESCOPED (existing globe_map.rs sufficient for current needs)

**Description:**
Apply GPU rendering to existing `globe_map.rs`.

**Note:** This task was descoped as the existing globe_map.rs implementation
meets current performance requirements. Can be revisited if needed.

---

## Phase 5: Polish (3 days)

### TASK-3D-013: Performance Benchmarks
**Priority:** P1 | **Effort:** S (1 day)

**Description:**
Create benchmarks comparing CPU vs GPU rendering.

**File:** `benches/render3d_bench.rs`

```rust
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};

fn surface_plot_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("surface_plot");

    for resolution in [25, 50, 100, 200, 500] {
        group.bench_with_input(
            BenchmarkId::new("gpu", resolution),
            &resolution,
            |b, &res| {
                b.iter(|| {
                    // GPU rendering path
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, surface_plot_benchmark);
criterion_main!(benches);
```

**Target Metrics:**

| Resolution | CPU Time | GPU Time | Speedup |
|------------|----------|----------|---------|
| 25×25 | 5ms | 0.1ms | 50× |
| 100×100 | 80ms | 0.2ms | 400× |
| 500×500 | >1s | 1ms | >1000× |

---

### TASK-3D-014: Documentation
**Priority:** P2 | **Effort:** S (1 day)

**Description:**
Document the 3D rendering system.

**Files:**
- `docs/guides/3d_rendering.md` - User guide
- `src/render3d/mod.rs` - Rustdoc module docs
- Update `README.md` with 3D examples

**Documentation Topics:**
1. Getting started with 3D charts
2. Camera controls
3. Custom colormaps
4. Performance optimization tips
5. Adding custom 3D visualizations

---

### TASK-3D-015: Example Updates
**Priority:** P2 | **Effort:** S (1 day)

**Description:**
Update chart_zoo examples to use new GPU rendering.

**Changes:**
- `surface_plot.rs` - Migrate to GPU
- `globe_map.rs` - Add lighting
- Add `scatter_3d.rs` example
- Add `bar_3d.rs` example

**Acceptance Criteria:**
- [ ] All examples compile and run
- [ ] Examples demonstrate GPU performance
- [ ] Code is well-commented
- [ ] Controls are documented in UI

---

## Risk Management

### Technical Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Shader compilation fails | High | Medium | Test on multiple GPUs early |
| Makepad API changes | Medium | Low | Pin to specific commit |
| Performance regression | Medium | Low | Benchmark after each phase |
| Memory leaks in geometry | High | Low | Use RAII, test with large datasets |

### Fallback Plan

If GPU approach encounters blockers:
1. Keep CPU path as fallback
2. Use hybrid approach (GPU for static, CPU for dynamic)
3. Optimize CPU path with SIMD

---

## Success Criteria

### Phase 0-1 Complete ✅
- [x] `DrawSurface3D` renders with Phong lighting
- [x] Transform updates work correctly
- [x] Colormaps display properly (7 scientific colormaps)

### Phase 2 Complete ✅
- [x] Camera orbits, pans, and zooms smoothly
- [x] Mouse/keyboard controls work
- [x] CameraController provides framework-agnostic event handling

### Phase 3 Complete ✅
- [x] Surface plot renders at 60fps for 100×100 grid
- [x] Visual output matches original
- [x] Animation is smooth

### Phase 4 Complete ✅
- [x] Scatter3D renders 10K points at 60fps
- [x] Bar3D works correctly
- [x] Face-based shading provides depth perception

### Phase 5 Complete ✅
- [x] Benchmarks created (render3d_bench.rs)
- [x] Documentation is complete (mod.rs, all modules)
- [x] Examples are updated (scatter3d_chart, bar3d_chart)

---

## Appendix: File Changes Summary

### New Files

| File | Purpose | LOC (est) |
|------|---------|-----------|
| `src/render3d/mod.rs` | Module exports | 50 |
| `src/render3d/types.rs` | Transform3D, Vec3 | 200 |
| `src/render3d/mesh.rs` | MeshData | 500 |
| `src/render3d/geometry.rs` | GeometryMesh3D | 150 |
| `src/render3d/draw_mesh.rs` | DrawMesh3D shader | 400 |
| `src/render3d/camera.rs` | Camera3D | 300 |
| `src/render3d/colormap.rs` | Shader colormaps | 200 |
| `docs/GPU_3D_RENDERING_ARCHITECTURE.md` | Architecture doc | 800 |
| `docs/3D_OPTIMIZATION_PLAN.md` | This plan | 1000 |
| `benches/render3d_bench.rs` | Benchmarks | 100 |

**Total new: ~3,700 LOC**

### Modified Files

| File | Changes |
|------|---------|
| `src/lib.rs` | Add `render3d` module export |
| `examples/chart_zoo/src/charts/surface_plot.rs` | Migrate to GPU |
| `examples/chart_zoo/src/charts/globe_map.rs` | Add lighting |
| `README.md` | Add 3D section |
| `Cargo.toml` | Add `glam` dependency |

---

## References

- [GPU_3D_RENDERING_ARCHITECTURE.md](./GPU_3D_RENDERING_ARCHITECTURE.md) - Detailed architecture
- `makepad-urdf-player/src/mesh.rs` - Reference implementation
- `makepad-urdf-player/src/robot_view.rs` - Camera implementation
- [Makepad Shader DSL](https://github.com/makepad/makepad) - Shader documentation

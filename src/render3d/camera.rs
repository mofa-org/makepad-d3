//! 3D Camera system with orbital controls
//!
//! This module provides a camera implementation optimized for data visualization,
//! featuring orbital rotation, panning, and smooth zoom controls.

use super::types::{Mat4, Transform3D, Vec3};
use std::f64::consts::PI;

/// Orbital camera for 3D visualization
///
/// The camera orbits around a target point, supporting:
/// - Orbital rotation (yaw/pitch)
/// - Panning (shift + drag)
/// - Smooth zoom
/// - Pitch clamping to prevent gimbal lock
///
/// # Coordinate System
///
/// - Y-axis is up
/// - Camera looks toward target
/// - Positive yaw rotates counterclockwise when viewed from above
/// - Positive pitch tilts the camera up
///
/// # Example
///
/// ```rust
/// use makepad_d3::render3d::camera::Camera3D;
///
/// let mut camera = Camera3D::new()
///     .with_distance(5.0)
///     .with_yaw(0.3)
///     .with_pitch(0.5);
///
/// // Handle mouse drag
/// camera.orbit(0.01, 0.005);  // Rotate
///
/// // Handle scroll wheel
/// camera.zoom(0.95);  // Zoom in 5%
///
/// // Get view-projection matrix
/// let transform = camera.view_projection(1.0);  // aspect ratio
/// ```
#[derive(Clone, Debug)]
pub struct Camera3D {
    /// Distance from camera to target
    pub distance: f64,

    /// Rotation around Y axis (radians)
    pub yaw: f64,

    /// Rotation around X axis (radians), clamped to prevent gimbal lock
    pub pitch: f64,

    /// Look-at target point
    pub target: [f64; 3],

    /// Pan offset in screen space
    pub pan_x: f64,
    pub pan_y: f64,

    /// Vertical field of view (radians)
    pub fov: f64,

    /// Near clipping plane
    pub near: f64,

    /// Far clipping plane
    pub far: f64,

    /// Minimum allowed distance
    pub min_distance: f64,

    /// Maximum allowed distance
    pub max_distance: f64,

    /// Minimum pitch (radians)
    pub min_pitch: f64,

    /// Maximum pitch (radians)
    pub max_pitch: f64,

    /// Smoothing factor (0.0 = instant, 1.0 = very smooth)
    pub smoothing: f64,

    // Animation targets for smooth motion
    target_distance: f64,
    target_yaw: f64,
    target_pitch: f64,
    target_pan_x: f64,
    target_pan_y: f64,
}

impl Default for Camera3D {
    fn default() -> Self {
        Self::new()
    }
}

impl Camera3D {
    /// Create a new camera with default settings
    pub fn new() -> Self {
        Self {
            distance: 5.0,
            yaw: 0.3,
            pitch: 0.5,
            target: [0.0, 0.0, 0.0],
            pan_x: 0.0,
            pan_y: 0.0,
            fov: PI / 4.0, // 45 degrees
            near: 0.01,
            far: 100.0,
            min_distance: 0.1,
            max_distance: 50.0,
            min_pitch: -PI / 2.0 + 0.1,
            max_pitch: PI / 2.0 - 0.1,
            smoothing: 0.0,
            target_distance: 5.0,
            target_yaw: 0.3,
            target_pitch: 0.5,
            target_pan_x: 0.0,
            target_pan_y: 0.0,
        }
    }

    /// Set the distance from target
    pub fn with_distance(mut self, distance: f64) -> Self {
        self.distance = distance;
        self.target_distance = distance;
        self
    }

    /// Set the yaw angle (rotation around Y axis)
    pub fn with_yaw(mut self, yaw: f64) -> Self {
        self.yaw = yaw;
        self.target_yaw = yaw;
        self
    }

    /// Set the pitch angle (rotation around X axis)
    pub fn with_pitch(mut self, pitch: f64) -> Self {
        let clamped = pitch.clamp(self.min_pitch, self.max_pitch);
        self.pitch = clamped;
        self.target_pitch = clamped;
        self
    }

    /// Set the target point
    pub fn with_target(mut self, target: [f64; 3]) -> Self {
        self.target = target;
        self
    }

    /// Set the field of view (radians)
    pub fn with_fov(mut self, fov: f64) -> Self {
        self.fov = fov;
        self
    }

    /// Set clipping planes
    pub fn with_clip_planes(mut self, near: f64, far: f64) -> Self {
        self.near = near;
        self.far = far;
        self
    }

    /// Set distance limits
    pub fn with_distance_limits(mut self, min: f64, max: f64) -> Self {
        self.min_distance = min;
        self.max_distance = max;
        self
    }

    /// Set pitch limits
    pub fn with_pitch_limits(mut self, min: f64, max: f64) -> Self {
        self.min_pitch = min;
        self.max_pitch = max;
        self
    }

    /// Set smoothing factor
    pub fn with_smoothing(mut self, smoothing: f64) -> Self {
        self.smoothing = smoothing.clamp(0.0, 0.99);
        self
    }

    /// Get camera position in world space
    pub fn position(&self) -> Vec3 {
        let x = self.distance * self.pitch.cos() * self.yaw.sin();
        let y = self.distance * self.pitch.sin();
        let z = self.distance * self.pitch.cos() * self.yaw.cos();

        Vec3::new(
            (self.target[0] + x) as f32,
            (self.target[1] + y) as f32,
            (self.target[2] + z) as f32,
        )
    }

    /// Get camera position as f64 array
    pub fn position_f64(&self) -> [f64; 3] {
        let x = self.distance * self.pitch.cos() * self.yaw.sin();
        let y = self.distance * self.pitch.sin();
        let z = self.distance * self.pitch.cos() * self.yaw.cos();

        [self.target[0] + x, self.target[1] + y, self.target[2] + z]
    }

    /// Get the view matrix
    pub fn view_matrix(&self) -> Mat4 {
        let pos = self.position();
        let target = Vec3::new(
            self.target[0] as f32,
            self.target[1] as f32,
            self.target[2] as f32,
        );
        Mat4::look_at_rh(pos, target, Vec3::Y)
    }

    /// Get the projection matrix
    pub fn projection_matrix(&self, aspect_ratio: f32) -> Mat4 {
        Mat4::perspective_rh(
            self.fov as f32,
            aspect_ratio,
            self.near as f32,
            self.far as f32,
        )
    }

    /// Get the combined view-projection matrix
    pub fn view_projection_matrix(&self, aspect_ratio: f32) -> Mat4 {
        self.projection_matrix(aspect_ratio) * self.view_matrix()
    }

    /// Get the view-projection as a Transform3D
    pub fn view_projection(&self, aspect_ratio: f32) -> Transform3D {
        Transform3D::from_mat4(&self.view_projection_matrix(aspect_ratio))
    }

    /// Get an orbital camera transform suitable for the URDF-style rendering
    ///
    /// This produces a transform that includes:
    /// - Base rotation (Z-up to Y-up conversion if needed)
    /// - Orbital rotation (yaw/pitch)
    /// - Scale (based on distance)
    /// - Pan offset
    pub fn orbital_transform(&self) -> Transform3D {
        let cam_yaw = self.yaw as f32;
        let cam_pitch = self.pitch as f32;
        let cam_scale = 1.0 / self.distance as f32;

        // Base rotation (if converting from Z-up coordinate system)
        let base_rot = Mat4::from_rotation_x(-std::f32::consts::PI / 2.0);

        // Orbital rotation (YXZ order)
        let orbital_rot = Mat4::from_euler_yxz(cam_yaw, cam_pitch, 0.0);

        // Scale based on distance
        let scale_mat = Mat4::from_scale(Vec3::splat(cam_scale));

        // Pan offset
        let pan_mat = Mat4::from_translation(Vec3::new(self.pan_x as f32, self.pan_y as f32, 0.0));

        // Compose: pan * scale * orbital * base
        let combined = pan_mat * scale_mat * orbital_rot * base_rot;
        Transform3D::from_mat4(&combined)
    }

    /// Orbit the camera (rotate around target)
    pub fn orbit(&mut self, delta_yaw: f64, delta_pitch: f64) {
        if self.smoothing > 0.0 {
            self.target_yaw += delta_yaw;
            self.target_pitch =
                (self.target_pitch + delta_pitch).clamp(self.min_pitch, self.max_pitch);
        } else {
            self.yaw += delta_yaw;
            self.pitch = (self.pitch + delta_pitch).clamp(self.min_pitch, self.max_pitch);
            self.target_yaw = self.yaw;
            self.target_pitch = self.pitch;
        }
    }

    /// Pan the camera (move target perpendicular to view direction)
    pub fn pan(&mut self, delta_x: f64, delta_y: f64) {
        let speed = 0.003 * self.distance;

        if self.smoothing > 0.0 {
            self.target_pan_x += delta_x * speed;
            self.target_pan_y += delta_y * speed;
        } else {
            self.pan_x += delta_x * speed;
            self.pan_y += delta_y * speed;
            self.target_pan_x = self.pan_x;
            self.target_pan_y = self.pan_y;
        }
    }

    /// Zoom the camera (change distance to target)
    pub fn zoom(&mut self, factor: f64) {
        let new_distance = (self.distance * factor).clamp(self.min_distance, self.max_distance);

        if self.smoothing > 0.0 {
            self.target_distance = new_distance;
        } else {
            self.distance = new_distance;
            self.target_distance = new_distance;
        }
    }

    /// Set zoom level directly
    pub fn set_distance(&mut self, distance: f64) {
        let clamped = distance.clamp(self.min_distance, self.max_distance);

        if self.smoothing > 0.0 {
            self.target_distance = clamped;
        } else {
            self.distance = clamped;
            self.target_distance = clamped;
        }
    }

    /// Reset camera to default view
    pub fn reset(&mut self) {
        let default = Self::new();
        self.distance = default.distance;
        self.yaw = default.yaw;
        self.pitch = default.pitch;
        self.pan_x = 0.0;
        self.pan_y = 0.0;
        self.target_distance = default.distance;
        self.target_yaw = default.yaw;
        self.target_pitch = default.pitch;
        self.target_pan_x = 0.0;
        self.target_pan_y = 0.0;
    }

    /// Update camera animation (call once per frame)
    ///
    /// Returns true if the camera is still animating.
    pub fn update(&mut self, _dt: f64) -> bool {
        if self.smoothing <= 0.0 {
            return false;
        }

        let lerp = 1.0 - self.smoothing;

        let d_distance = (self.target_distance - self.distance).abs();
        let d_yaw = (self.target_yaw - self.yaw).abs();
        let d_pitch = (self.target_pitch - self.pitch).abs();
        let d_pan_x = (self.target_pan_x - self.pan_x).abs();
        let d_pan_y = (self.target_pan_y - self.pan_y).abs();

        let threshold = 0.0001;
        let is_animating = d_distance > threshold
            || d_yaw > threshold
            || d_pitch > threshold
            || d_pan_x > threshold
            || d_pan_y > threshold;

        if is_animating {
            self.distance = self.distance + (self.target_distance - self.distance) * lerp;
            self.yaw = self.yaw + (self.target_yaw - self.yaw) * lerp;
            self.pitch = self.pitch + (self.target_pitch - self.pitch) * lerp;
            self.pan_x = self.pan_x + (self.target_pan_x - self.pan_x) * lerp;
            self.pan_y = self.pan_y + (self.target_pan_y - self.pan_y) * lerp;
        }

        is_animating
    }

    /// Check if the camera is currently animating
    pub fn is_animating(&self) -> bool {
        if self.smoothing <= 0.0 {
            return false;
        }

        let threshold = 0.0001;
        (self.target_distance - self.distance).abs() > threshold
            || (self.target_yaw - self.yaw).abs() > threshold
            || (self.target_pitch - self.pitch).abs() > threshold
            || (self.target_pan_x - self.pan_x).abs() > threshold
            || (self.target_pan_y - self.pan_y).abs() > threshold
    }

    /// Convert screen coordinates to a ray in world space
    pub fn screen_to_ray(
        &self,
        screen_x: f64,
        screen_y: f64,
        viewport_width: f64,
        viewport_height: f64,
    ) -> Ray3D {
        // Normalize to [-1, 1]
        let ndc_x = (2.0 * screen_x / viewport_width - 1.0) as f32;
        let ndc_y = (1.0 - 2.0 * screen_y / viewport_height) as f32;

        // Get camera basis vectors
        let pos = self.position();
        let target = Vec3::new(
            self.target[0] as f32,
            self.target[1] as f32,
            self.target[2] as f32,
        );

        let forward = (target - pos).normalize();
        let right = forward.cross(&Vec3::Y).normalize();
        let up = right.cross(&forward);

        // Compute ray direction
        let aspect = (viewport_width / viewport_height) as f32;
        let fov_tan = (self.fov as f32 / 2.0).tan();

        let direction =
            (forward + right * (ndc_x * fov_tan * aspect) + up * (ndc_y * fov_tan)).normalize();

        Ray3D {
            origin: [pos.x as f64, pos.y as f64, pos.z as f64],
            direction: [direction.x as f64, direction.y as f64, direction.z as f64],
        }
    }

    /// Convert world coordinates to screen coordinates
    pub fn world_to_screen(
        &self,
        world_pos: [f64; 3],
        viewport_width: f64,
        viewport_height: f64,
    ) -> Option<[f64; 2]> {
        let aspect = (viewport_width / viewport_height) as f32;
        let mvp = self.view_projection_matrix(aspect);

        let pos = Vec3::new(
            world_pos[0] as f32,
            world_pos[1] as f32,
            world_pos[2] as f32,
        );
        let clip = mvp.transform_point(pos);

        // Check if behind camera
        if clip.z < 0.0 {
            return None;
        }

        // Convert to screen coordinates
        let screen_x = ((clip.x + 1.0) / 2.0) as f64 * viewport_width;
        let screen_y = ((1.0 - clip.y) / 2.0) as f64 * viewport_height;

        Some([screen_x, screen_y])
    }
}

/// 3D ray for intersection testing
#[derive(Clone, Debug)]
pub struct Ray3D {
    /// Ray origin
    pub origin: [f64; 3],
    /// Ray direction (normalized)
    pub direction: [f64; 3],
}

impl Ray3D {
    /// Create a new ray
    pub fn new(origin: [f64; 3], direction: [f64; 3]) -> Self {
        // Normalize direction
        let len = (direction[0] * direction[0]
            + direction[1] * direction[1]
            + direction[2] * direction[2])
            .sqrt();
        let normalized = if len > 1e-10 {
            [direction[0] / len, direction[1] / len, direction[2] / len]
        } else {
            [0.0, 0.0, 1.0]
        };

        Self {
            origin,
            direction: normalized,
        }
    }

    /// Get a point along the ray at parameter t
    pub fn at(&self, t: f64) -> [f64; 3] {
        [
            self.origin[0] + self.direction[0] * t,
            self.origin[1] + self.direction[1] * t,
            self.origin[2] + self.direction[2] * t,
        ]
    }

    /// Intersect with a plane defined by normal and distance from origin
    ///
    /// Returns the parameter t where the ray intersects the plane, if any.
    pub fn intersect_plane(&self, normal: [f64; 3], d: f64) -> Option<f64> {
        let denom = normal[0] * self.direction[0]
            + normal[1] * self.direction[1]
            + normal[2] * self.direction[2];

        if denom.abs() < 1e-10 {
            return None; // Parallel to plane
        }

        let numer = -(normal[0] * self.origin[0]
            + normal[1] * self.origin[1]
            + normal[2] * self.origin[2]
            + d);

        let t = numer / denom;

        if t >= 0.0 {
            Some(t)
        } else {
            None // Behind ray origin
        }
    }

    /// Intersect with a sphere
    ///
    /// Returns (t_near, t_far) if there's an intersection.
    pub fn intersect_sphere(&self, center: [f64; 3], radius: f64) -> Option<(f64, f64)> {
        let oc = [
            self.origin[0] - center[0],
            self.origin[1] - center[1],
            self.origin[2] - center[2],
        ];

        let a = self.direction[0] * self.direction[0]
            + self.direction[1] * self.direction[1]
            + self.direction[2] * self.direction[2];
        let b = 2.0
            * (oc[0] * self.direction[0] + oc[1] * self.direction[1] + oc[2] * self.direction[2]);
        let c = oc[0] * oc[0] + oc[1] * oc[1] + oc[2] * oc[2] - radius * radius;

        let discriminant = b * b - 4.0 * a * c;

        if discriminant < 0.0 {
            return None;
        }

        let sqrt_d = discriminant.sqrt();
        let t1 = (-b - sqrt_d) / (2.0 * a);
        let t2 = (-b + sqrt_d) / (2.0 * a);

        if t2 < 0.0 {
            None
        } else if t1 < 0.0 {
            Some((0.0, t2))
        } else {
            Some((t1, t2))
        }
    }

    /// Intersect with an axis-aligned bounding box
    ///
    /// Returns the parameter t where the ray enters the box, if any.
    pub fn intersect_aabb(&self, min: [f64; 3], max: [f64; 3]) -> Option<f64> {
        let mut t_min = f64::NEG_INFINITY;
        let mut t_max = f64::INFINITY;

        for i in 0..3 {
            if self.direction[i].abs() < 1e-10 {
                // Ray parallel to slab
                if self.origin[i] < min[i] || self.origin[i] > max[i] {
                    return None;
                }
            } else {
                let inv_d = 1.0 / self.direction[i];
                let mut t1 = (min[i] - self.origin[i]) * inv_d;
                let mut t2 = (max[i] - self.origin[i]) * inv_d;

                if t1 > t2 {
                    std::mem::swap(&mut t1, &mut t2);
                }

                t_min = t_min.max(t1);
                t_max = t_max.min(t2);

                if t_min > t_max {
                    return None;
                }
            }
        }

        if t_max < 0.0 {
            None
        } else if t_min < 0.0 {
            Some(0.0)
        } else {
            Some(t_min)
        }
    }
}

/// Camera controller that handles Makepad events
///
/// This wraps a Camera3D and provides integration with Makepad's event system,
/// tracking drag state and processing mouse/keyboard events.
///
/// # Example
///
/// ```rust,ignore
/// use makepad_d3::render3d::camera::CameraController;
/// use makepad_widgets::*;
///
/// #[derive(Live, Widget)]
/// struct MyView {
///     camera: CameraController,
/// }
///
/// impl Widget for MyView {
///     fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
///         if self.camera.handle_event(cx, event, self.area()) {
///             self.redraw(cx);
///         }
///     }
/// }
/// ```
#[derive(Clone, Debug)]
pub struct CameraController {
    /// The underlying camera
    pub camera: Camera3D,

    /// Whether we're currently dragging
    is_dragging: bool,

    /// Last mouse position during drag
    last_pos: [f64; 2],

    /// Position where drag started
    start_pos: [f64; 2],

    /// Sensitivity for orbit rotation
    pub orbit_sensitivity: f64,

    /// Sensitivity for pan movement
    pub pan_sensitivity: f64,

    /// Sensitivity for zoom
    pub zoom_sensitivity: f64,

    /// Whether shift key was held at drag start (for pan mode)
    shift_drag: bool,
}

impl Default for CameraController {
    fn default() -> Self {
        Self::new(Camera3D::new())
    }
}

impl CameraController {
    /// Create a new camera controller
    pub fn new(camera: Camera3D) -> Self {
        Self {
            camera,
            is_dragging: false,
            last_pos: [0.0, 0.0],
            start_pos: [0.0, 0.0],
            orbit_sensitivity: 0.01,
            pan_sensitivity: 1.0,
            zoom_sensitivity: 0.001,
            shift_drag: false,
        }
    }

    /// Create with default camera
    pub fn with_default_camera() -> Self {
        Self::new(Camera3D::new())
    }

    /// Set orbit sensitivity
    pub fn with_orbit_sensitivity(mut self, sensitivity: f64) -> Self {
        self.orbit_sensitivity = sensitivity;
        self
    }

    /// Set pan sensitivity
    pub fn with_pan_sensitivity(mut self, sensitivity: f64) -> Self {
        self.pan_sensitivity = sensitivity;
        self
    }

    /// Set zoom sensitivity
    pub fn with_zoom_sensitivity(mut self, sensitivity: f64) -> Self {
        self.zoom_sensitivity = sensitivity;
        self
    }

    /// Access the underlying camera
    pub fn camera(&self) -> &Camera3D {
        &self.camera
    }

    /// Access the underlying camera mutably
    pub fn camera_mut(&mut self) -> &mut Camera3D {
        &mut self.camera
    }

    /// Check if currently dragging
    pub fn is_dragging(&self) -> bool {
        self.is_dragging
    }

    /// Start a drag operation
    pub fn start_drag(&mut self, pos: [f64; 2], shift: bool) {
        self.is_dragging = true;
        self.last_pos = pos;
        self.start_pos = pos;
        self.shift_drag = shift;
    }

    /// Update drag position
    pub fn update_drag(&mut self, pos: [f64; 2], shift: bool) -> bool {
        if !self.is_dragging {
            return false;
        }

        let delta_x = pos[0] - self.last_pos[0];
        let delta_y = pos[1] - self.last_pos[1];
        self.last_pos = pos;

        // Use shift state from drag start, or current if shift was pressed during drag
        let is_pan = self.shift_drag || shift;

        if is_pan {
            self.camera.pan(
                delta_x * self.pan_sensitivity,
                -delta_y * self.pan_sensitivity,
            );
        } else {
            self.camera.orbit(
                delta_x * self.orbit_sensitivity,
                delta_y * self.orbit_sensitivity,
            );
        }

        true
    }

    /// End drag operation
    pub fn end_drag(&mut self) {
        self.is_dragging = false;
        self.shift_drag = false;
    }

    /// Handle scroll wheel for zoom
    pub fn handle_scroll(&mut self, scroll_y: f64) -> bool {
        let factor = 1.0 - scroll_y * self.zoom_sensitivity;
        self.camera.zoom(factor);
        true
    }

    /// Handle keyboard input
    ///
    /// Returns true if the key was handled.
    pub fn handle_key(&mut self, key_code: u32) -> bool {
        // Key codes based on common mappings
        // These match makepad_platform::keyboard::KeyCode values
        const KEY_EQUALS: u32 = 61; // =
        const KEY_MINUS: u32 = 45; // -
        const KEY_PLUS: u32 = 43; // + (shift+=)
        const KEY_HOME: u32 = 0x24; // Home
        const KEY_R: u32 = 82; // R (reset)
        const KEY_NUMPAD_PLUS: u32 = 107;
        const KEY_NUMPAD_MINUS: u32 = 109;

        match key_code {
            KEY_EQUALS | KEY_PLUS | KEY_NUMPAD_PLUS => {
                self.camera.zoom(0.9);
                true
            }
            KEY_MINUS | KEY_NUMPAD_MINUS => {
                self.camera.zoom(1.1);
                true
            }
            KEY_HOME | KEY_R => {
                self.camera.reset();
                true
            }
            _ => false,
        }
    }

    /// Update camera animation
    ///
    /// Call this every frame when the camera is animating.
    /// Returns true if still animating.
    pub fn update(&mut self, dt: f64) -> bool {
        self.camera.update(dt)
    }

    /// Check if camera needs animation update
    pub fn needs_update(&self) -> bool {
        self.camera.is_animating()
    }
}

/// Makepad event types that the camera controller can handle
///
/// This provides a framework-agnostic way to handle events.
/// Users can convert Makepad events to these types for processing.
#[derive(Clone, Debug)]
pub enum CameraEvent {
    /// Mouse/touch down at position
    PointerDown { pos: [f64; 2], shift: bool },
    /// Mouse/touch move to position
    PointerMove { pos: [f64; 2], shift: bool },
    /// Mouse/touch up
    PointerUp,
    /// Scroll wheel
    Scroll { delta_y: f64 },
    /// Key press
    KeyDown { key_code: u32 },
    /// Animation frame
    Frame { dt: f64 },
}

impl CameraController {
    /// Handle a camera event
    ///
    /// Returns true if the event was handled and the view should be redrawn.
    pub fn handle_camera_event(&mut self, event: CameraEvent) -> bool {
        match event {
            CameraEvent::PointerDown { pos, shift } => {
                self.start_drag(pos, shift);
                true
            }
            CameraEvent::PointerMove { pos, shift } => self.update_drag(pos, shift),
            CameraEvent::PointerUp => {
                self.end_drag();
                false
            }
            CameraEvent::Scroll { delta_y } => self.handle_scroll(delta_y),
            CameraEvent::KeyDown { key_code } => self.handle_key(key_code),
            CameraEvent::Frame { dt } => self.update(dt),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_new() {
        let camera = Camera3D::new();
        assert!((camera.distance - 5.0).abs() < 0.01);
        assert!((camera.yaw - 0.3).abs() < 0.01);
        assert!((camera.pitch - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_camera_position() {
        let camera = Camera3D::new()
            .with_distance(10.0)
            .with_yaw(0.0)
            .with_pitch(0.0);

        let pos = camera.position();

        // At yaw=0, pitch=0, camera should be on +Z axis
        assert!(pos.x.abs() < 0.01);
        assert!(pos.y.abs() < 0.01);
        assert!((pos.z - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_camera_orbit() {
        let mut camera = Camera3D::new().with_yaw(0.0);
        camera.orbit(0.5, 0.0);

        assert!((camera.yaw - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_camera_zoom() {
        let mut camera = Camera3D::new().with_distance(10.0);
        camera.zoom(0.5);

        assert!((camera.distance - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_camera_pitch_clamping() {
        let mut camera = Camera3D::new().with_pitch(0.0);
        camera.orbit(0.0, 10.0); // Try to pitch beyond limit

        assert!(camera.pitch < PI / 2.0);
    }

    #[test]
    fn test_ray_plane_intersection() {
        let ray = Ray3D::new([0.0, 1.0, 0.0], [0.0, -1.0, 0.0]);
        let t = ray.intersect_plane([0.0, 1.0, 0.0], 0.0);

        assert!(t.is_some());
        let point = ray.at(t.unwrap());
        assert!(point[1].abs() < 0.01); // Should hit y=0 plane
    }

    #[test]
    fn test_ray_sphere_intersection() {
        let ray = Ray3D::new([0.0, 0.0, -5.0], [0.0, 0.0, 1.0]);
        let result = ray.intersect_sphere([0.0, 0.0, 0.0], 1.0);

        assert!(result.is_some());
        let (t1, t2) = result.unwrap();
        assert!((t1 - 4.0).abs() < 0.01); // Enter at z=-1
        assert!((t2 - 6.0).abs() < 0.01); // Exit at z=1
    }

    #[test]
    fn test_ray_aabb_intersection() {
        let ray = Ray3D::new([-2.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        let t = ray.intersect_aabb([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);

        assert!(t.is_some());
        assert!((t.unwrap() - 1.0).abs() < 0.01); // Should hit at x=-1
    }

    #[test]
    fn test_ray_miss() {
        let ray = Ray3D::new([0.0, 10.0, 0.0], [1.0, 0.0, 0.0]); // Parallel to XZ plane
        let t = ray.intersect_sphere([0.0, 0.0, 0.0], 1.0);

        assert!(t.is_none());
    }

    #[test]
    fn test_view_projection() {
        let camera = Camera3D::new();
        let transform = camera.view_projection(1.0);

        // Just check it doesn't crash and produces valid values
        assert!(transform.col0.x.is_finite());
        assert!(transform.col1.y.is_finite());
        assert!(transform.col2.z.is_finite());
    }

    // CameraController tests

    #[test]
    fn test_controller_new() {
        let controller = CameraController::with_default_camera();
        assert!(!controller.is_dragging());
        assert!((controller.camera().distance - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_controller_drag_orbit() {
        let mut controller = CameraController::new(Camera3D::new().with_yaw(0.0).with_pitch(0.0));

        // Start drag without shift (orbit mode)
        controller.start_drag([100.0, 100.0], false);
        assert!(controller.is_dragging());

        // Drag to the right
        controller.update_drag([200.0, 100.0], false);

        // Yaw should increase
        assert!(controller.camera().yaw > 0.0);
        assert!((controller.camera().pitch - 0.0).abs() < 0.1); // Pitch unchanged

        controller.end_drag();
        assert!(!controller.is_dragging());
    }

    #[test]
    fn test_controller_drag_pan() {
        let mut controller = CameraController::new(Camera3D::new());
        let initial_pan_x = controller.camera().pan_x;

        // Start drag with shift (pan mode)
        controller.start_drag([100.0, 100.0], true);

        // Drag to the right
        controller.update_drag([200.0, 100.0], true);

        // Pan should change
        assert!(controller.camera().pan_x != initial_pan_x);

        controller.end_drag();
    }

    #[test]
    fn test_controller_scroll_zoom() {
        let mut controller = CameraController::new(Camera3D::new().with_distance(5.0));

        // Scroll up (negative delta = zoom out, distance increases)
        controller.handle_scroll(-100.0);

        // Distance should increase (zoom out)
        assert!(controller.camera().distance > 5.0);

        // Reset and test zoom in
        let mut controller2 = CameraController::new(Camera3D::new().with_distance(5.0));

        // Scroll down (positive delta = zoom in, distance decreases)
        controller2.handle_scroll(100.0);

        // Distance should decrease (zoom in)
        assert!(controller2.camera().distance < 5.0);
    }

    #[test]
    fn test_controller_camera_event() {
        let mut controller = CameraController::with_default_camera();

        // Test pointer down event
        let handled = controller.handle_camera_event(CameraEvent::PointerDown {
            pos: [100.0, 100.0],
            shift: false,
        });
        assert!(handled);
        assert!(controller.is_dragging());

        // Test pointer up event
        controller.handle_camera_event(CameraEvent::PointerUp);
        assert!(!controller.is_dragging());

        // Test scroll event
        let initial_distance = controller.camera().distance;
        controller.handle_camera_event(CameraEvent::Scroll { delta_y: -100.0 });
        assert!(controller.camera().distance != initial_distance);
    }
}

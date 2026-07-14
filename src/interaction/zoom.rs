//! Zoom and pan behavior for visualizations
//!
//! Provides smooth zooming and panning with constraints.

use serde::{Deserialize, Serialize};

/// A 2D point for interaction coordinates
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Point2D {
    /// X coordinate
    pub x: f64,
    /// Y coordinate
    pub y: f64,
}

impl Point2D {
    /// Create a new point
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Zero point
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

/// A rectangular extent for constraints
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Extent {
    /// Minimum X coordinate
    pub x0: f64,
    /// Minimum Y coordinate
    pub y0: f64,
    /// Maximum X coordinate
    pub x1: f64,
    /// Maximum Y coordinate
    pub y1: f64,
}

impl Extent {
    /// Create a new extent
    pub fn new(x0: f64, y0: f64, x1: f64, y1: f64) -> Self {
        Self { x0, y0, x1, y1 }
    }

    /// Create from position and size
    pub fn from_size(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x0: x,
            y0: y,
            x1: x + width,
            y1: y + height,
        }
    }

    /// Get width
    pub fn width(&self) -> f64 {
        self.x1 - self.x0
    }

    /// Get height
    pub fn height(&self) -> f64 {
        self.y1 - self.y0
    }

    /// Check if point is inside extent
    pub fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.x0 && x <= self.x1 && y >= self.y0 && y <= self.y1
    }
}

/// Zoom transform representing scale and translation
///
/// The transform can be applied to convert between screen coordinates
/// and data coordinates.
///
/// # Example
///
/// ```
/// use makepad_d3::interaction::ZoomTransform;
///
/// let transform = ZoomTransform::new(2.0, 100.0, 50.0);
///
/// // Apply transform to a point
/// let (screen_x, screen_y) = transform.apply(10.0, 20.0);
///
/// // Invert transform
/// let (data_x, data_y) = transform.invert(screen_x, screen_y);
/// assert!((data_x - 10.0).abs() < 1e-10);
/// assert!((data_y - 20.0).abs() < 1e-10);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ZoomTransform {
    /// Scale factor (1.0 = no zoom)
    pub k: f64,
    /// X translation
    pub x: f64,
    /// Y translation
    pub y: f64,
}

impl Default for ZoomTransform {
    fn default() -> Self {
        Self::identity()
    }
}

impl ZoomTransform {
    /// Create a new zoom transform
    pub fn new(k: f64, x: f64, y: f64) -> Self {
        Self { k, x, y }
    }

    /// Identity transform (no zoom, no translation)
    pub fn identity() -> Self {
        Self {
            k: 1.0,
            x: 0.0,
            y: 0.0,
        }
    }

    /// Create a transform with only scale
    pub fn scale(k: f64) -> Self {
        Self { k, x: 0.0, y: 0.0 }
    }

    /// Create a transform with only translation
    pub fn translate(x: f64, y: f64) -> Self {
        Self { k: 1.0, x, y }
    }

    /// Apply the transform to a point (data -> screen)
    pub fn apply(&self, x: f64, y: f64) -> (f64, f64) {
        (x * self.k + self.x, y * self.k + self.y)
    }

    /// Apply the transform to only the X coordinate
    pub fn apply_x(&self, x: f64) -> f64 {
        x * self.k + self.x
    }

    /// Apply the transform to only the Y coordinate
    pub fn apply_y(&self, y: f64) -> f64 {
        y * self.k + self.y
    }

    /// Invert the transform (screen -> data)
    pub fn invert(&self, x: f64, y: f64) -> (f64, f64) {
        ((x - self.x) / self.k, (y - self.y) / self.k)
    }

    /// Invert only the X coordinate
    pub fn invert_x(&self, x: f64) -> f64 {
        (x - self.x) / self.k
    }

    /// Invert only the Y coordinate
    pub fn invert_y(&self, y: f64) -> f64 {
        (y - self.y) / self.k
    }

    /// Rescale a linear domain through this transform
    ///
    /// Useful for updating scale domains based on zoom level.
    pub fn rescale_x(&self, domain: (f64, f64), range: (f64, f64)) -> (f64, f64) {
        let (d0, d1) = domain;
        let (r0, r1) = range;
        let ratio = (d1 - d0) / (r1 - r0);
        (
            d0 + (self.invert_x(r0) - r0) * ratio,
            d0 + (self.invert_x(r1) - r0) * ratio,
        )
    }

    /// Rescale Y domain
    pub fn rescale_y(&self, domain: (f64, f64), range: (f64, f64)) -> (f64, f64) {
        let (d0, d1) = domain;
        let (r0, r1) = range;
        let ratio = (d1 - d0) / (r1 - r0);
        (
            d0 + (self.invert_y(r0) - r0) * ratio,
            d0 + (self.invert_y(r1) - r0) * ratio,
        )
    }

    /// Compose this transform with another (this * other)
    pub fn compose(&self, other: &ZoomTransform) -> ZoomTransform {
        ZoomTransform {
            k: self.k * other.k,
            x: self.x + self.k * other.x,
            y: self.y + self.k * other.y,
        }
    }

    /// Check if this is the identity transform
    pub fn is_identity(&self) -> bool {
        (self.k - 1.0).abs() < 1e-10 && self.x.abs() < 1e-10 && self.y.abs() < 1e-10
    }
}

/// Zoom behavior configuration and event handling
///
/// # Example
///
/// ```
/// use makepad_d3::interaction::{ZoomBehavior, ZoomTransform};
///
/// let mut zoom = ZoomBehavior::new()
///     .scale_extent(0.5, 4.0)
///     .wheel_delta(0.002);
///
/// let mut transform = ZoomTransform::identity();
///
/// // Handle wheel event (delta is typically from scroll wheel)
/// zoom.handle_wheel(&mut transform, 120.0, 400.0, 300.0);
/// ```
#[derive(Clone, Debug)]
pub struct ZoomBehavior {
    /// Minimum and maximum scale factors
    scale_extent: (f64, f64),
    /// Optional bounds for translation
    translate_extent: Option<Extent>,
    /// Multiplier for wheel delta
    wheel_delta: f64,
    /// Whether X-axis zooming is enabled
    zoom_x: bool,
    /// Whether Y-axis zooming is enabled
    zoom_y: bool,
    /// Whether panning is enabled
    pan_enabled: bool,
    /// Constrain zoom to extent
    constrain_to_extent: bool,
}

impl Default for ZoomBehavior {
    fn default() -> Self {
        Self::new()
    }
}

impl ZoomBehavior {
    /// Create a new zoom behavior with default settings
    pub fn new() -> Self {
        Self {
            scale_extent: (0.1, 10.0),
            translate_extent: None,
            wheel_delta: 0.002,
            zoom_x: true,
            zoom_y: true,
            pan_enabled: true,
            constrain_to_extent: false,
        }
    }

    /// Set the minimum and maximum scale factors
    pub fn scale_extent(mut self, min: f64, max: f64) -> Self {
        self.scale_extent = (min.max(0.001), max.max(min));
        self
    }

    /// Set the translation extent (bounds)
    pub fn translate_extent(mut self, extent: Extent) -> Self {
        self.translate_extent = Some(extent);
        self.constrain_to_extent = true;
        self
    }

    /// Set the wheel delta multiplier
    pub fn wheel_delta(mut self, delta: f64) -> Self {
        self.wheel_delta = delta;
        self
    }

    /// Enable or disable X-axis zooming
    pub fn zoom_x(mut self, enabled: bool) -> Self {
        self.zoom_x = enabled;
        self
    }

    /// Enable or disable Y-axis zooming
    pub fn zoom_y(mut self, enabled: bool) -> Self {
        self.zoom_y = enabled;
        self
    }

    /// Enable or disable panning
    pub fn pan_enabled(mut self, enabled: bool) -> Self {
        self.pan_enabled = enabled;
        self
    }

    /// Get the scale extent
    pub fn get_scale_extent(&self) -> (f64, f64) {
        self.scale_extent
    }

    /// Get the translate extent
    pub fn get_translate_extent(&self) -> Option<&Extent> {
        self.translate_extent.as_ref()
    }

    /// Handle mouse wheel event
    ///
    /// Uses D3's invert→zoom→project pattern to keep the cursor position fixed.
    ///
    /// # Arguments
    /// * `transform` - Current zoom transform (will be modified)
    /// * `delta` - Wheel delta (positive = zoom in, negative = zoom out)
    /// * `center_x` - X coordinate of zoom center (cursor position)
    /// * `center_y` - Y coordinate of zoom center (cursor position)
    ///
    /// # Returns
    /// Whether the transform changed
    pub fn handle_wheel(
        &self,
        transform: &mut ZoomTransform,
        delta: f64,
        center_x: f64,
        center_y: f64,
    ) -> bool {
        let k0 = transform.k;
        let k1 =
            (k0 * (1.0 + delta * self.wheel_delta)).clamp(self.scale_extent.0, self.scale_extent.1);

        if (k1 - k0).abs() < 1e-10 {
            return false;
        }

        // D3's invert→zoom→project pattern:
        // 1. Find what data coordinate is under the cursor (invert)
        let (data_x, data_y) = transform.invert(center_x, center_y);

        // 2. Update scale
        transform.k = k1;

        // 3. Adjust translation so the same data point stays under the cursor (project)
        // Formula: center = data * k + translate  =>  translate = center - data * k
        if self.zoom_x {
            transform.x = center_x - data_x * k1;
        }
        if self.zoom_y {
            transform.y = center_y - data_y * k1;
        }

        self.constrain(transform);
        true
    }

    /// Handle pan/drag movement
    ///
    /// # Arguments
    /// * `transform` - Current zoom transform (will be modified)
    /// * `delta_x` - X movement delta
    /// * `delta_y` - Y movement delta
    ///
    /// # Returns
    /// Whether the transform changed
    pub fn handle_pan(&self, transform: &mut ZoomTransform, delta_x: f64, delta_y: f64) -> bool {
        if !self.pan_enabled {
            return false;
        }

        let old_x = transform.x;
        let old_y = transform.y;

        transform.x += delta_x;
        transform.y += delta_y;

        self.constrain(transform);

        (transform.x - old_x).abs() > 1e-10 || (transform.y - old_y).abs() > 1e-10
    }

    /// Handle pinch zoom (for touch devices)
    ///
    /// Uses D3's invert→zoom→project pattern to keep the pinch center fixed.
    ///
    /// # Arguments
    /// * `transform` - Current zoom transform
    /// * `scale_factor` - Pinch scale factor (> 1 = zoom in)
    /// * `center_x` - X center of pinch
    /// * `center_y` - Y center of pinch
    pub fn handle_pinch(
        &self,
        transform: &mut ZoomTransform,
        scale_factor: f64,
        center_x: f64,
        center_y: f64,
    ) -> bool {
        let k0 = transform.k;
        let k1 = (k0 * scale_factor).clamp(self.scale_extent.0, self.scale_extent.1);

        if (k1 - k0).abs() < 1e-10 {
            return false;
        }

        // D3's invert→zoom→project pattern
        let (data_x, data_y) = transform.invert(center_x, center_y);
        transform.k = k1;

        if self.zoom_x {
            transform.x = center_x - data_x * k1;
        }
        if self.zoom_y {
            transform.y = center_y - data_y * k1;
        }

        self.constrain(transform);
        true
    }

    /// Reset transform to identity
    pub fn reset(&self, transform: &mut ZoomTransform) {
        *transform = ZoomTransform::identity();
    }

    /// Constrain transform to extent bounds
    fn constrain(&self, transform: &mut ZoomTransform) {
        if !self.constrain_to_extent {
            return;
        }

        if let Some(extent) = &self.translate_extent {
            // Ensure content stays within bounds
            let max_x = 0.0;
            let min_x = extent.width() * (1.0 - transform.k);
            let max_y = 0.0;
            let min_y = extent.height() * (1.0 - transform.k);

            transform.x = transform.x.clamp(min_x, max_x);
            transform.y = transform.y.clamp(min_y, max_y);
        }
    }

    /// Programmatically zoom to a specific scale
    ///
    /// Uses D3's invert→zoom→project pattern to keep the center point fixed.
    pub fn zoom_to(&self, transform: &mut ZoomTransform, scale: f64, center_x: f64, center_y: f64) {
        let k1 = scale.clamp(self.scale_extent.0, self.scale_extent.1);

        // D3's invert→zoom→project pattern
        let (data_x, data_y) = transform.invert(center_x, center_y);
        transform.k = k1;

        if self.zoom_x {
            transform.x = center_x - data_x * k1;
        }
        if self.zoom_y {
            transform.y = center_y - data_y * k1;
        }

        self.constrain(transform);
    }

    /// Programmatically translate to a specific position
    pub fn translate_to(&self, transform: &mut ZoomTransform, x: f64, y: f64) {
        transform.x = x;
        transform.y = y;
        self.constrain(transform);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zoom_transform_identity() {
        let t = ZoomTransform::identity();
        assert_eq!(t.k, 1.0);
        assert_eq!(t.x, 0.0);
        assert_eq!(t.y, 0.0);
        assert!(t.is_identity());
    }

    #[test]
    fn test_zoom_transform_apply() {
        let t = ZoomTransform::new(2.0, 100.0, 50.0);
        let (x, y) = t.apply(10.0, 20.0);
        assert_eq!(x, 120.0); // 10 * 2 + 100
        assert_eq!(y, 90.0); // 20 * 2 + 50
    }

    #[test]
    fn test_zoom_transform_invert() {
        let t = ZoomTransform::new(2.0, 100.0, 50.0);
        let (x, y) = t.invert(120.0, 90.0);
        assert!((x - 10.0).abs() < 1e-10);
        assert!((y - 20.0).abs() < 1e-10);
    }

    #[test]
    fn test_zoom_transform_roundtrip() {
        let t = ZoomTransform::new(2.5, 150.0, -30.0);
        let original = (42.0, 73.0);
        let applied = t.apply(original.0, original.1);
        let inverted = t.invert(applied.0, applied.1);
        assert!((inverted.0 - original.0).abs() < 1e-10);
        assert!((inverted.1 - original.1).abs() < 1e-10);
    }

    #[test]
    fn test_zoom_transform_compose() {
        let t1 = ZoomTransform::new(2.0, 10.0, 20.0);
        let t2 = ZoomTransform::new(1.5, 5.0, 10.0);
        let composed = t1.compose(&t2);

        // Apply composed should equal applying t1 then t2
        let point = (1.0, 1.0);
        let via_compose = composed.apply(point.0, point.1);
        let via_steps = t1.apply(point.0, point.1);

        assert_eq!(composed.k, 3.0); // 2.0 * 1.5
        assert_eq!(composed.x, 20.0); // 10 + 2 * 5
        assert_eq!(composed.y, 40.0); // 20 + 2 * 10
    }

    #[test]
    fn test_zoom_behavior_wheel() {
        let zoom = ZoomBehavior::new().scale_extent(0.5, 4.0);
        let mut transform = ZoomTransform::identity();

        // Zoom in
        zoom.handle_wheel(&mut transform, 100.0, 200.0, 150.0);
        assert!(transform.k > 1.0);

        // Zoom out
        zoom.handle_wheel(&mut transform, -200.0, 200.0, 150.0);
        assert!(transform.k < 1.2);
    }

    #[test]
    fn test_zoom_behavior_scale_extent() {
        let zoom = ZoomBehavior::new().scale_extent(0.5, 2.0);
        let mut transform = ZoomTransform::identity();

        // Try to zoom beyond max
        for _ in 0..1000 {
            zoom.handle_wheel(&mut transform, 100.0, 100.0, 100.0);
        }
        assert!(transform.k <= 2.0);

        // Try to zoom below min
        for _ in 0..1000 {
            zoom.handle_wheel(&mut transform, -100.0, 100.0, 100.0);
        }
        assert!(transform.k >= 0.5);
    }

    #[test]
    fn test_zoom_behavior_pan() {
        let zoom = ZoomBehavior::new();
        let mut transform = ZoomTransform::identity();

        zoom.handle_pan(&mut transform, 50.0, 30.0);
        assert_eq!(transform.x, 50.0);
        assert_eq!(transform.y, 30.0);

        zoom.handle_pan(&mut transform, -20.0, 10.0);
        assert_eq!(transform.x, 30.0);
        assert_eq!(transform.y, 40.0);
    }

    #[test]
    fn test_zoom_behavior_pan_disabled() {
        let zoom = ZoomBehavior::new().pan_enabled(false);
        let mut transform = ZoomTransform::identity();

        let changed = zoom.handle_pan(&mut transform, 50.0, 30.0);
        assert!(!changed);
        assert_eq!(transform.x, 0.0);
        assert_eq!(transform.y, 0.0);
    }

    #[test]
    fn test_zoom_behavior_zoom_to() {
        let zoom = ZoomBehavior::new().scale_extent(0.5, 4.0);
        let mut transform = ZoomTransform::identity();

        zoom.zoom_to(&mut transform, 2.0, 100.0, 100.0);
        assert_eq!(transform.k, 2.0);
    }

    #[test]
    fn test_zoom_transform_rescale() {
        let t = ZoomTransform::new(2.0, 100.0, 0.0);
        let domain = (0.0, 100.0);
        let range = (0.0, 500.0);

        let new_domain = t.rescale_x(domain, range);
        // With 2x zoom centered at origin, domain should effectively halve
        assert!(new_domain.0 < domain.0 || new_domain.1 > domain.1);
    }

    #[test]
    fn test_extent() {
        let e = Extent::new(0.0, 0.0, 100.0, 50.0);
        assert_eq!(e.width(), 100.0);
        assert_eq!(e.height(), 50.0);
        assert!(e.contains(50.0, 25.0));
        assert!(!e.contains(150.0, 25.0));
    }

    #[test]
    fn test_point2d() {
        let p = Point2D::new(10.0, 20.0);
        assert_eq!(p.x, 10.0);
        assert_eq!(p.y, 20.0);

        let z = Point2D::zero();
        assert_eq!(z.x, 0.0);
        assert_eq!(z.y, 0.0);
    }

    #[test]
    fn test_zoom_center_point_preservation() {
        // Test that zooming preserves the data point under the cursor
        let zoom = ZoomBehavior::new().scale_extent(0.1, 10.0);
        let mut transform = ZoomTransform::new(1.0, 50.0, 30.0);

        let cursor = (200.0, 150.0);

        // Find data point under cursor before zoom
        let data_before = transform.invert(cursor.0, cursor.1);

        // Zoom in
        zoom.handle_wheel(&mut transform, 500.0, cursor.0, cursor.1);

        // Find data point under cursor after zoom
        let data_after = transform.invert(cursor.0, cursor.1);

        // Data point should be the same (within floating point tolerance)
        assert!(
            (data_before.0 - data_after.0).abs() < 1e-10,
            "X data point changed: {} -> {}",
            data_before.0,
            data_after.0
        );
        assert!(
            (data_before.1 - data_after.1).abs() < 1e-10,
            "Y data point changed: {} -> {}",
            data_before.1,
            data_after.1
        );
    }

    #[test]
    fn test_zoom_center_preservation_multiple_zooms() {
        // Test that multiple consecutive zooms preserve the center
        let zoom = ZoomBehavior::new().scale_extent(0.1, 10.0);
        let mut transform = ZoomTransform::identity();

        let cursor = (300.0, 200.0);
        let data_original = transform.invert(cursor.0, cursor.1);

        // Multiple zoom operations
        for _ in 0..10 {
            zoom.handle_wheel(&mut transform, 50.0, cursor.0, cursor.1);
        }
        for _ in 0..5 {
            zoom.handle_wheel(&mut transform, -100.0, cursor.0, cursor.1);
        }

        let data_after = transform.invert(cursor.0, cursor.1);

        assert!(
            (data_original.0 - data_after.0).abs() < 1e-9,
            "X drifted: {} -> {}",
            data_original.0,
            data_after.0
        );
        assert!(
            (data_original.1 - data_after.1).abs() < 1e-9,
            "Y drifted: {} -> {}",
            data_original.1,
            data_after.1
        );
    }

    #[test]
    fn test_pinch_center_preservation() {
        let zoom = ZoomBehavior::new().scale_extent(0.1, 10.0);
        let mut transform = ZoomTransform::new(1.5, 100.0, 75.0);

        let center = (250.0, 180.0);
        let data_before = transform.invert(center.0, center.1);

        zoom.handle_pinch(&mut transform, 1.5, center.0, center.1);

        let data_after = transform.invert(center.0, center.1);

        assert!((data_before.0 - data_after.0).abs() < 1e-10);
        assert!((data_before.1 - data_after.1).abs() < 1e-10);
    }

    #[test]
    fn test_zoom_to_center_preservation() {
        let zoom = ZoomBehavior::new().scale_extent(0.1, 10.0);
        let mut transform = ZoomTransform::new(2.0, 80.0, 60.0);

        let center = (400.0, 300.0);
        let data_before = transform.invert(center.0, center.1);

        zoom.zoom_to(&mut transform, 3.5, center.0, center.1);

        let data_after = transform.invert(center.0, center.1);

        assert!((data_before.0 - data_after.0).abs() < 1e-10);
        assert!((data_before.1 - data_after.1).abs() < 1e-10);
    }
}

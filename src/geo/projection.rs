//! Geographic projections
//!
//! Transforms spherical coordinates (longitude, latitude) to planar coordinates.

use std::f64::consts::PI;

/// Trait for geographic projections
///
/// Projects spherical coordinates (longitude, latitude in degrees) to
/// planar coordinates (x, y in pixels).
pub trait Projection: Send + Sync {
    /// Project geographic coordinates to screen coordinates
    ///
    /// # Arguments
    /// * `lon` - Longitude in degrees (-180 to 180)
    /// * `lat` - Latitude in degrees (-90 to 90)
    ///
    /// # Returns
    /// Screen coordinates (x, y)
    fn project(&self, lon: f64, lat: f64) -> (f64, f64);

    /// Invert screen coordinates back to geographic coordinates
    ///
    /// # Arguments
    /// * `x` - Screen X coordinate
    /// * `y` - Screen Y coordinate
    ///
    /// # Returns
    /// Geographic coordinates (longitude, latitude) in degrees
    fn invert(&self, x: f64, y: f64) -> (f64, f64);

    /// Get the projection type name
    fn projection_type(&self) -> &'static str;

    /// Check if a point is visible in this projection
    fn is_visible(&self, lon: f64, lat: f64) -> bool {
        let _ = (lon, lat);
        true
    }

    /// Get the projection's clip extent
    fn clip_extent(&self) -> Option<((f64, f64), (f64, f64))> {
        None
    }
}

/// Builder trait for projections
pub trait ProjectionBuilder: Sized {
    /// Set the scale factor
    fn scale(self, scale: f64) -> Self;

    /// Set the center point (longitude, latitude)
    fn center(self, lon: f64, lat: f64) -> Self;

    /// Set the translation (screen offset)
    fn translate(self, x: f64, y: f64) -> Self;

    /// Set the rotation
    fn rotate(self, lambda: f64, phi: f64, gamma: f64) -> Self;

    /// Set the clip angle (for azimuthal projections)
    fn clip_angle(self, angle: f64) -> Self;

    /// Set the precision for adaptive resampling
    fn precision(self, precision: f64) -> Self;
}

/// Mercator projection (conformal cylindrical)
///
/// The standard projection for web maps. Preserves angles but distorts
/// areas significantly at high latitudes.
///
/// # Example
///
/// ```
/// use makepad_d3::geo::{MercatorProjection, Projection, ProjectionBuilder};
///
/// let projection = MercatorProjection::new()
///     .scale(100.0)
///     .translate(400.0, 300.0);
///
/// let (x, y) = projection.project(0.0, 0.0); // Null Island
/// assert!((x - 400.0).abs() < 0.01);
/// assert!((y - 300.0).abs() < 0.01);
/// ```
#[derive(Clone, Debug)]
pub struct MercatorProjection {
    /// Scale factor
    scale: f64,
    /// Center longitude
    center_lon: f64,
    /// Center latitude
    center_lat: f64,
    /// Translation X
    translate_x: f64,
    /// Translation Y
    translate_y: f64,
    /// Maximum latitude (clips at ~85.05°)
    max_lat: f64,
}

impl Default for MercatorProjection {
    fn default() -> Self {
        Self::new()
    }
}

impl MercatorProjection {
    /// Create a new Mercator projection
    pub fn new() -> Self {
        Self {
            scale: 1.0,
            center_lon: 0.0,
            center_lat: 0.0,
            translate_x: 0.0,
            translate_y: 0.0,
            max_lat: 85.05113, // atan(sinh(π)) in degrees
        }
    }

    /// Set the maximum latitude
    pub fn max_lat(mut self, lat: f64) -> Self {
        self.max_lat = lat.abs().min(89.99);
        self
    }
}

impl ProjectionBuilder for MercatorProjection {
    fn scale(mut self, scale: f64) -> Self {
        self.scale = scale;
        self
    }

    fn center(mut self, lon: f64, lat: f64) -> Self {
        self.center_lon = lon;
        self.center_lat = lat;
        self
    }

    fn translate(mut self, x: f64, y: f64) -> Self {
        self.translate_x = x;
        self.translate_y = y;
        self
    }

    fn rotate(self, _lambda: f64, _phi: f64, _gamma: f64) -> Self {
        // Mercator doesn't support rotation in the traditional sense
        self
    }

    fn clip_angle(self, _angle: f64) -> Self {
        self
    }

    fn precision(self, _precision: f64) -> Self {
        self
    }
}

impl Projection for MercatorProjection {
    fn project(&self, lon: f64, lat: f64) -> (f64, f64) {
        // Clamp latitude to avoid infinity
        let lat = lat.clamp(-self.max_lat, self.max_lat);

        // Convert to radians
        let lambda = (lon - self.center_lon).to_radians();
        let phi = lat.to_radians();

        // Mercator projection formula
        let x = lambda;
        let y = (PI / 4.0 + phi / 2.0).tan().ln();

        // Apply scale and translate
        (
            x * self.scale + self.translate_x,
            -y * self.scale + self.translate_y, // Y is inverted
        )
    }

    fn invert(&self, x: f64, y: f64) -> (f64, f64) {
        // Remove scale and translate
        let px = (x - self.translate_x) / self.scale;
        let py = -(y - self.translate_y) / self.scale;

        // Inverse Mercator
        let lon = px.to_degrees() + self.center_lon;
        let lat = (2.0 * py.exp().atan() - PI / 2.0).to_degrees();

        (lon, lat)
    }

    fn projection_type(&self) -> &'static str {
        "mercator"
    }
}

/// Equirectangular projection (plate carrée)
///
/// The simplest projection - directly maps longitude to x and latitude to y.
/// Good for data visualization but has significant distortion at high latitudes.
///
/// # Example
///
/// ```
/// use makepad_d3::geo::{EquirectangularProjection, Projection, ProjectionBuilder};
///
/// let projection = EquirectangularProjection::new()
///     .scale(100.0)
///     .translate(400.0, 300.0);
///
/// // Project the center point (0, 0) - should map to translate coordinates
/// let (x, y) = projection.project(0.0, 0.0);
/// assert!((x - 400.0).abs() < 0.01);
/// assert!((y - 300.0).abs() < 0.01);
/// ```
#[derive(Clone, Debug)]
pub struct EquirectangularProjection {
    /// Scale factor
    scale: f64,
    /// Center longitude
    center_lon: f64,
    /// Center latitude
    center_lat: f64,
    /// Translation X
    translate_x: f64,
    /// Translation Y
    translate_y: f64,
}

impl Default for EquirectangularProjection {
    fn default() -> Self {
        Self::new()
    }
}

impl EquirectangularProjection {
    /// Create a new equirectangular projection
    pub fn new() -> Self {
        Self {
            scale: 1.0,
            center_lon: 0.0,
            center_lat: 0.0,
            translate_x: 0.0,
            translate_y: 0.0,
        }
    }
}

impl ProjectionBuilder for EquirectangularProjection {
    fn scale(mut self, scale: f64) -> Self {
        self.scale = scale;
        self
    }

    fn center(mut self, lon: f64, lat: f64) -> Self {
        self.center_lon = lon;
        self.center_lat = lat;
        self
    }

    fn translate(mut self, x: f64, y: f64) -> Self {
        self.translate_x = x;
        self.translate_y = y;
        self
    }

    fn rotate(self, _lambda: f64, _phi: f64, _gamma: f64) -> Self {
        self
    }

    fn clip_angle(self, _angle: f64) -> Self {
        self
    }

    fn precision(self, _precision: f64) -> Self {
        self
    }
}

impl Projection for EquirectangularProjection {
    fn project(&self, lon: f64, lat: f64) -> (f64, f64) {
        let x = (lon - self.center_lon).to_radians();
        let y = (lat - self.center_lat).to_radians();

        (
            x * self.scale + self.translate_x,
            -y * self.scale + self.translate_y,
        )
    }

    fn invert(&self, x: f64, y: f64) -> (f64, f64) {
        let px = (x - self.translate_x) / self.scale;
        let py = -(y - self.translate_y) / self.scale;

        let lon = px.to_degrees() + self.center_lon;
        let lat = py.to_degrees() + self.center_lat;

        (lon, lat)
    }

    fn projection_type(&self) -> &'static str {
        "equirectangular"
    }
}

/// Orthographic projection (azimuthal)
///
/// Shows the Earth as a globe viewed from space. Points on the far side
/// of the globe are not visible.
///
/// # Example
///
/// ```
/// use makepad_d3::geo::{OrthographicProjection, Projection, ProjectionBuilder};
///
/// let projection = OrthographicProjection::new()
///     .scale(200.0)
///     .translate(400.0, 300.0);
///
/// // Project the center (0, 0) - should map to translate coordinates
/// let (x, y) = projection.project(0.0, 0.0);
/// assert!((x - 400.0).abs() < 0.01);
/// assert!((y - 300.0).abs() < 0.01);
/// ```
#[derive(Clone, Debug)]
pub struct OrthographicProjection {
    /// Scale factor
    scale: f64,
    /// Translation X
    translate_x: f64,
    /// Translation Y
    translate_y: f64,
    /// Rotation: lambda (longitude)
    rotate_lambda: f64,
    /// Rotation: phi (latitude)
    rotate_phi: f64,
    /// Rotation: gamma (roll)
    rotate_gamma: f64,
    /// Clip angle in degrees (default 90°)
    clip_angle: f64,
}

impl Default for OrthographicProjection {
    fn default() -> Self {
        Self::new()
    }
}

impl OrthographicProjection {
    /// Create a new orthographic projection
    pub fn new() -> Self {
        Self {
            scale: 1.0,
            translate_x: 0.0,
            translate_y: 0.0,
            rotate_lambda: 0.0,
            rotate_phi: 0.0,
            rotate_gamma: 0.0,
            clip_angle: 90.0,
        }
    }

    /// Apply rotation to convert to rotated coordinates
    ///
    /// D3's rotation convention (from d3-geo/src/rotation.js):
    /// - λ (rotate_lambda): Add to longitude
    /// - φ (rotate_phi): Tilt the pole
    /// - γ (rotate_gamma): Roll around the viewing axis
    fn rotate_point(&self, lon: f64, lat: f64) -> (f64, f64) {
        // Step 1: Apply λ rotation (shift longitude)
        let lambda = lon.to_radians() + self.rotate_lambda.to_radians();
        let phi = lat.to_radians();

        // Step 2: Apply φ and γ rotation (D3's rotationPhiGamma formula)
        let delta_phi = self.rotate_phi.to_radians();
        let delta_gamma = self.rotate_gamma.to_radians();

        let cos_delta_phi = delta_phi.cos();
        let sin_delta_phi = delta_phi.sin();
        let cos_delta_gamma = delta_gamma.cos();
        let sin_delta_gamma = delta_gamma.sin();

        let cos_phi = phi.cos();
        let x = lambda.cos() * cos_phi;
        let y = lambda.sin() * cos_phi;
        let z = phi.sin();

        // D3's formula for combined φ and γ rotation
        let k = z * cos_delta_phi + x * sin_delta_phi;

        let new_lambda = (y * cos_delta_gamma - k * sin_delta_gamma)
            .atan2(x * cos_delta_phi - z * sin_delta_phi);
        let new_phi = (k * cos_delta_gamma + y * sin_delta_gamma)
            .clamp(-1.0, 1.0)
            .asin();

        (new_lambda, new_phi)
    }

    /// Apply inverse rotation to convert from rotated coordinates back to geographic
    ///
    /// The inverse applies the same D3 formula with negated angles,
    /// then subtracts the λ rotation.
    fn inverse_rotate_point(&self, lambda: f64, phi: f64) -> (f64, f64) {
        // Apply inverse φ,γ rotation using the same D3 formula with negated angles
        let delta_phi = -self.rotate_phi.to_radians();
        let delta_gamma = -self.rotate_gamma.to_radians();

        let cos_delta_phi = delta_phi.cos();
        let sin_delta_phi = delta_phi.sin();
        let cos_delta_gamma = delta_gamma.cos();
        let sin_delta_gamma = delta_gamma.sin();

        let cos_phi = phi.cos();
        let x = lambda.cos() * cos_phi;
        let y = lambda.sin() * cos_phi;
        let z = phi.sin();

        // D3's formula with negated angles
        let k = z * cos_delta_phi + x * sin_delta_phi;

        let new_lambda = (y * cos_delta_gamma - k * sin_delta_gamma)
            .atan2(x * cos_delta_phi - z * sin_delta_phi);
        let new_phi = (k * cos_delta_gamma + y * sin_delta_gamma)
            .clamp(-1.0, 1.0)
            .asin();

        // Apply inverse λ rotation
        let lon = new_lambda.to_degrees() - self.rotate_lambda;
        let lat = new_phi.to_degrees();

        (lon, lat)
    }
}

impl ProjectionBuilder for OrthographicProjection {
    fn scale(mut self, scale: f64) -> Self {
        self.scale = scale;
        self
    }

    fn center(self, _lon: f64, _lat: f64) -> Self {
        // For orthographic, use rotate instead
        self
    }

    fn translate(mut self, x: f64, y: f64) -> Self {
        self.translate_x = x;
        self.translate_y = y;
        self
    }

    fn rotate(mut self, lambda: f64, phi: f64, gamma: f64) -> Self {
        self.rotate_lambda = lambda;
        self.rotate_phi = phi;
        self.rotate_gamma = gamma;
        self
    }

    fn clip_angle(mut self, angle: f64) -> Self {
        self.clip_angle = angle.clamp(0.0, 180.0);
        self
    }

    fn precision(self, _precision: f64) -> Self {
        self
    }
}

impl Projection for OrthographicProjection {
    fn project(&self, lon: f64, lat: f64) -> (f64, f64) {
        let (lambda, phi) = self.rotate_point(lon, lat);

        // Orthographic projection
        let x = phi.cos() * lambda.sin();
        let y = phi.sin();

        (
            x * self.scale + self.translate_x,
            -y * self.scale + self.translate_y,
        )
    }

    fn invert(&self, x: f64, y: f64) -> (f64, f64) {
        let px = (x - self.translate_x) / self.scale;
        let py = -(y - self.translate_y) / self.scale;

        let rho = (px * px + py * py).sqrt();
        if rho > 1.0 {
            // Point is outside the globe
            return (f64::NAN, f64::NAN);
        }

        let c = rho.asin();
        let sin_c = c.sin();
        let cos_c = c.cos();

        // Compute spherical coordinates in rotated frame
        let phi = if rho == 0.0 {
            0.0
        } else {
            (py * sin_c / rho).asin()
        };

        let lambda = (px * sin_c).atan2(rho * cos_c);

        // Apply inverse rotation to get geographic coordinates
        self.inverse_rotate_point(lambda, phi)
    }

    fn projection_type(&self) -> &'static str {
        "orthographic"
    }

    fn is_visible(&self, lon: f64, lat: f64) -> bool {
        let (lambda, phi) = self.rotate_point(lon, lat);
        let cos_c = phi.cos() * lambda.cos();
        cos_c >= self.clip_angle.to_radians().cos()
    }
}

/// Albers equal-area conic projection
///
/// Good for maps of countries/regions at mid-latitudes (like the US).
/// Preserves area but not shape.
///
/// # Example
///
/// ```
/// use makepad_d3::geo::{AlbersProjection, Projection, ProjectionBuilder};
///
/// let projection = AlbersProjection::usa()
///     .scale(1000.0)
///     .translate(480.0, 300.0);
///
/// // Project a point in the continental US
/// let (x, y) = projection.project(-98.0, 39.0);
/// ```
#[derive(Clone, Debug)]
pub struct AlbersProjection {
    /// Scale factor
    scale: f64,
    /// Translation X
    translate_x: f64,
    /// Translation Y
    translate_y: f64,
    /// Center longitude
    center_lon: f64,
    /// Center latitude
    center_lat: f64,
    /// First standard parallel
    parallel1: f64,
    /// Second standard parallel
    parallel2: f64,
    // Precomputed values
    n: f64,
    c: f64,
    rho0: f64,
}

impl Default for AlbersProjection {
    fn default() -> Self {
        Self::new()
    }
}

impl AlbersProjection {
    /// Create a new Albers projection with default parameters
    pub fn new() -> Self {
        Self::with_parallels(29.5, 45.5)
    }

    /// Create an Albers projection optimized for the USA
    pub fn usa() -> Self {
        Self::with_parallels(29.5, 45.5).center(-98.0, 39.0)
    }

    /// Create with custom standard parallels
    pub fn with_parallels(parallel1: f64, parallel2: f64) -> Self {
        let mut proj = Self {
            scale: 1.0,
            translate_x: 0.0,
            translate_y: 0.0,
            center_lon: 0.0,
            center_lat: 0.0,
            parallel1,
            parallel2,
            n: 0.0,
            c: 0.0,
            rho0: 0.0,
        };
        proj.compute_constants();
        proj
    }

    /// Set standard parallels
    pub fn parallels(mut self, p1: f64, p2: f64) -> Self {
        self.parallel1 = p1;
        self.parallel2 = p2;
        self.compute_constants();
        self
    }

    fn compute_constants(&mut self) {
        let phi1 = self.parallel1.to_radians();
        let phi2 = self.parallel2.to_radians();
        let phi0 = self.center_lat.to_radians();

        let sin_phi1 = phi1.sin();
        let sin_phi2 = phi2.sin();
        let cos_phi1 = phi1.cos();

        self.n = (sin_phi1 + sin_phi2) / 2.0;
        self.c = cos_phi1 * cos_phi1 + 2.0 * self.n * sin_phi1;
        self.rho0 = (self.c - 2.0 * self.n * phi0.sin()).sqrt() / self.n;
    }
}

impl ProjectionBuilder for AlbersProjection {
    fn scale(mut self, scale: f64) -> Self {
        self.scale = scale;
        self
    }

    fn center(mut self, lon: f64, lat: f64) -> Self {
        self.center_lon = lon;
        self.center_lat = lat;
        self.compute_constants();
        self
    }

    fn translate(mut self, x: f64, y: f64) -> Self {
        self.translate_x = x;
        self.translate_y = y;
        self
    }

    fn rotate(self, _lambda: f64, _phi: f64, _gamma: f64) -> Self {
        self
    }

    fn clip_angle(self, _angle: f64) -> Self {
        self
    }

    fn precision(self, _precision: f64) -> Self {
        self
    }
}

impl Projection for AlbersProjection {
    fn project(&self, lon: f64, lat: f64) -> (f64, f64) {
        let lambda = (lon - self.center_lon).to_radians();
        let phi = lat.to_radians();

        let rho = (self.c - 2.0 * self.n * phi.sin()).sqrt() / self.n;
        let theta = self.n * lambda;

        let x = rho * theta.sin();
        let y = self.rho0 - rho * theta.cos();

        (
            x * self.scale + self.translate_x,
            y * self.scale + self.translate_y,
        )
    }

    fn invert(&self, x: f64, y: f64) -> (f64, f64) {
        let px = (x - self.translate_x) / self.scale;
        let py = (y - self.translate_y) / self.scale;

        let rho0_minus_y = self.rho0 - py;
        let rho = (px * px + rho0_minus_y * rho0_minus_y).sqrt();
        let rho = if self.n < 0.0 { -rho } else { rho };

        let theta = px.atan2(rho0_minus_y);

        let lon = (theta / self.n).to_degrees() + self.center_lon;
        let lat = ((self.c - rho * rho * self.n * self.n) / (2.0 * self.n))
            .asin()
            .to_degrees();

        (lon, lat)
    }

    fn projection_type(&self) -> &'static str {
        "albers"
    }
}

/// Calculate the bounds of a geometry when projected
///
/// Returns [[min_x, min_y], [max_x, max_y]] in projected coordinates.
pub fn project_bounds<P: Projection>(
    projection: &P,
    coordinates: &[[f64; 2]],
) -> Option<[[f64; 2]; 2]> {
    if coordinates.is_empty() {
        return None;
    }

    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for coord in coordinates {
        let (lon, lat) = (coord[0], coord[1]);
        if !projection.is_visible(lon, lat) {
            continue;
        }
        let (x, y) = projection.project(lon, lat);
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }

    if min_x.is_finite() && min_y.is_finite() && max_x.is_finite() && max_y.is_finite() {
        Some([[min_x, min_y], [max_x, max_y]])
    } else {
        None
    }
}

/// Compute scale and translate to fit coordinates within an extent.
///
/// This is equivalent to D3's `projection.fitExtent()`.
///
/// # Arguments
/// * `coordinates` - The geographic coordinates [[lon, lat], ...]
/// * `extent` - The target extent [[x0, y0], [x1, y1]]
///
/// # Returns
/// (scale, translate_x, translate_y) that would fit the geometry
///
/// # Example
///
/// ```
/// use makepad_d3::geo::{MercatorProjection, Projection, ProjectionBuilder, compute_fit_extent};
///
/// // US bounds (approximate)
/// let us_bounds = vec![
///     [-125.0, 24.0],  // SW corner
///     [-66.0, 24.0],   // SE corner
///     [-66.0, 50.0],   // NE corner
///     [-125.0, 50.0],  // NW corner
/// ];
///
/// // Fit to a 960x600 display
/// let projection = MercatorProjection::new();
/// let (scale, tx, ty) = compute_fit_extent(&projection, &us_bounds, [[0.0, 0.0], [960.0, 600.0]]);
///
/// // Apply the computed values
/// let fitted = MercatorProjection::new()
///     .scale(scale)
///     .translate(tx, ty);
/// ```
pub fn compute_fit_extent<P: Projection>(
    projection: &P,
    coordinates: &[[f64; 2]],
    extent: [[f64; 2]; 2],
) -> (f64, f64, f64) {
    // First, project all coordinates with scale=1, translate=0
    // to get the raw projection bounds
    if coordinates.is_empty() {
        return (1.0, 0.0, 0.0);
    }

    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for coord in coordinates {
        let (lon, lat) = (coord[0], coord[1]);
        if !projection.is_visible(lon, lat) {
            continue;
        }
        let (x, y) = projection.project(lon, lat);
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }

    if !min_x.is_finite() || !min_y.is_finite() || !max_x.is_finite() || !max_y.is_finite() {
        return (1.0, 0.0, 0.0);
    }

    let [[x0, y0], [x1, y1]] = extent;
    let extent_width = (x1 - x0).abs();
    let extent_height = (y1 - y0).abs();
    let proj_width = max_x - min_x;
    let proj_height = max_y - min_y;

    if proj_width <= 0.0 || proj_height <= 0.0 {
        return (1.0, (x0 + x1) / 2.0, (y0 + y1) / 2.0);
    }

    // Calculate scale to fit (use smaller scale to fit both dimensions)
    let scale_x = extent_width / proj_width;
    let scale_y = extent_height / proj_height;
    let scale = scale_x.min(scale_y);

    // Calculate translate to center the geometry
    let center_x = (min_x + max_x) / 2.0;
    let center_y = (min_y + max_y) / 2.0;
    let target_center_x = (x0 + x1) / 2.0;
    let target_center_y = (y0 + y1) / 2.0;

    // The translate needed to move projected center to target center
    // After scaling: new_x = x * scale + tx
    // We want: center_x * scale + tx = target_center_x
    // So: tx = target_center_x - center_x * scale
    // But this assumes projection has no existing translate...
    // For simplicity, return scale relative to unit projection

    let tx = target_center_x - center_x * scale;
    let ty = target_center_y - center_y * scale;

    (scale, tx, ty)
}

/// Compute scale and translate to fit coordinates within a size.
///
/// This is equivalent to D3's `projection.fitSize()`.
///
/// # Arguments
/// * `coordinates` - The geographic coordinates [[lon, lat], ...]
/// * `width` - Target width
/// * `height` - Target height
///
/// # Returns
/// (scale, translate_x, translate_y) that would fit the geometry
pub fn compute_fit_size<P: Projection>(
    projection: &P,
    coordinates: &[[f64; 2]],
    width: f64,
    height: f64,
) -> (f64, f64, f64) {
    compute_fit_extent(projection, coordinates, [[0.0, 0.0], [width, height]])
}

impl MercatorProjection {
    /// Fit the projection to display given coordinates within an extent.
    ///
    /// # Example
    ///
    /// ```
    /// use makepad_d3::geo::{MercatorProjection, Projection};
    ///
    /// // US bounds (approximate)
    /// let us_bounds = vec![
    ///     [-125.0, 24.0],
    ///     [-66.0, 24.0],
    ///     [-66.0, 50.0],
    ///     [-125.0, 50.0],
    /// ];
    ///
    /// let projection = MercatorProjection::new()
    ///     .fit_extent(&us_bounds, [[0.0, 0.0], [960.0, 600.0]]);
    /// ```
    pub fn fit_extent(self, coordinates: &[[f64; 2]], extent: [[f64; 2]; 2]) -> Self {
        // Create a base projection to compute bounds
        let base = MercatorProjection::new().center(self.center_lon, self.center_lat);
        let (scale, tx, ty) = compute_fit_extent(&base, coordinates, extent);
        Self {
            scale,
            translate_x: tx,
            translate_y: ty,
            ..self
        }
    }

    /// Fit the projection to display given coordinates within a size.
    pub fn fit_size(self, coordinates: &[[f64; 2]], width: f64, height: f64) -> Self {
        self.fit_extent(coordinates, [[0.0, 0.0], [width, height]])
    }
}

impl EquirectangularProjection {
    /// Fit the projection to display given coordinates within an extent.
    pub fn fit_extent(self, coordinates: &[[f64; 2]], extent: [[f64; 2]; 2]) -> Self {
        let base = EquirectangularProjection::new().center(self.center_lon, self.center_lat);
        let (scale, tx, ty) = compute_fit_extent(&base, coordinates, extent);
        Self {
            scale,
            translate_x: tx,
            translate_y: ty,
            ..self
        }
    }

    /// Fit the projection to display given coordinates within a size.
    pub fn fit_size(self, coordinates: &[[f64; 2]], width: f64, height: f64) -> Self {
        self.fit_extent(coordinates, [[0.0, 0.0], [width, height]])
    }
}

impl OrthographicProjection {
    /// Fit the projection to display given coordinates within an extent.
    pub fn fit_extent(self, coordinates: &[[f64; 2]], extent: [[f64; 2]; 2]) -> Self {
        let base = OrthographicProjection::new()
            .rotate(self.rotate_lambda, self.rotate_phi, self.rotate_gamma)
            .clip_angle(self.clip_angle);
        let (scale, tx, ty) = compute_fit_extent(&base, coordinates, extent);
        Self {
            scale,
            translate_x: tx,
            translate_y: ty,
            ..self
        }
    }

    /// Fit the projection to display given coordinates within a size.
    pub fn fit_size(self, coordinates: &[[f64; 2]], width: f64, height: f64) -> Self {
        self.fit_extent(coordinates, [[0.0, 0.0], [width, height]])
    }
}

impl AlbersProjection {
    /// Fit the projection to display given coordinates within an extent.
    pub fn fit_extent(mut self, coordinates: &[[f64; 2]], extent: [[f64; 2]; 2]) -> Self {
        // For Albers, we need to recalculate with the new scale
        let base = AlbersProjection::new()
            .parallels(self.parallel1, self.parallel2)
            .center(self.center_lon, self.center_lat);
        let (scale, tx, ty) = compute_fit_extent(&base, coordinates, extent);
        self.scale = scale;
        self.translate_x = tx;
        self.translate_y = ty;
        self.compute_constants();
        self
    }

    /// Fit the projection to display given coordinates within a size.
    pub fn fit_size(self, coordinates: &[[f64; 2]], width: f64, height: f64) -> Self {
        self.fit_extent(coordinates, [[0.0, 0.0], [width, height]])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mercator_new() {
        let proj = MercatorProjection::new();
        assert_eq!(proj.projection_type(), "mercator");
    }

    #[test]
    fn test_mercator_project_origin() {
        let proj = MercatorProjection::new()
            .scale(100.0)
            .translate(400.0, 300.0);

        let (x, y) = proj.project(0.0, 0.0);
        assert!((x - 400.0).abs() < 0.01);
        assert!((y - 300.0).abs() < 0.01);
    }

    #[test]
    fn test_mercator_roundtrip() {
        let proj = MercatorProjection::new()
            .scale(100.0)
            .translate(400.0, 300.0);

        let original = (-122.4, 37.8); // San Francisco
        let (x, y) = proj.project(original.0, original.1);
        let (lon, lat) = proj.invert(x, y);

        assert!((lon - original.0).abs() < 0.01);
        assert!((lat - original.1).abs() < 0.01);
    }

    #[test]
    fn test_equirectangular_new() {
        let proj = EquirectangularProjection::new();
        assert_eq!(proj.projection_type(), "equirectangular");
    }

    #[test]
    fn test_equirectangular_project() {
        let proj = EquirectangularProjection::new()
            .scale(1.0)
            .translate(0.0, 0.0);

        let (x, y) = proj.project(90.0, 0.0);
        assert!((x - PI / 2.0).abs() < 0.01);
        assert!(y.abs() < 0.01);
    }

    #[test]
    fn test_equirectangular_roundtrip() {
        let proj = EquirectangularProjection::new()
            .scale(100.0)
            .translate(200.0, 100.0);

        let original = (45.0, 30.0);
        let (x, y) = proj.project(original.0, original.1);
        let (lon, lat) = proj.invert(x, y);

        assert!((lon - original.0).abs() < 0.01);
        assert!((lat - original.1).abs() < 0.01);
    }

    #[test]
    fn test_orthographic_new() {
        let proj = OrthographicProjection::new();
        assert_eq!(proj.projection_type(), "orthographic");
    }

    #[test]
    fn test_orthographic_center() {
        let proj = OrthographicProjection::new()
            .scale(200.0)
            .translate(400.0, 300.0)
            .rotate(0.0, 0.0, 0.0);

        // Point at center should project to translate point
        let (x, y) = proj.project(0.0, 0.0);
        assert!((x - 400.0).abs() < 0.01);
        assert!((y - 300.0).abs() < 0.01);
    }

    #[test]
    fn test_orthographic_visibility() {
        let proj = OrthographicProjection::new().rotate(0.0, 0.0, 0.0);

        // Front of globe should be visible
        assert!(proj.is_visible(0.0, 0.0));

        // Back of globe should not be visible
        assert!(!proj.is_visible(180.0, 0.0));
    }

    #[test]
    fn test_orthographic_rotation_lambda() {
        // λ rotation shifts the center longitude
        let proj = OrthographicProjection::new()
            .scale(100.0)
            .translate(0.0, 0.0)
            .rotate(-90.0, 0.0, 0.0); // Rotate to center on 90°E

        // Point at 90°E should now be at the center
        let (x, y) = proj.project(90.0, 0.0);
        assert!(x.abs() < 0.01, "90°E should be at center, x={}", x);
        assert!(y.abs() < 0.01, "90°E should be at center, y={}", y);
    }

    #[test]
    fn test_orthographic_rotation_phi() {
        // φ rotation shifts the center latitude
        let proj = OrthographicProjection::new()
            .scale(100.0)
            .translate(0.0, 0.0)
            .rotate(0.0, -45.0, 0.0); // Rotate to look at 45°N

        // Point at 0°E, 45°N should be near the center
        let (x, y) = proj.project(0.0, 45.0);
        assert!(x.abs() < 1.0, "0°E 45°N should be near center, x={}", x);
        assert!(y.abs() < 1.0, "0°E 45°N should be near center, y={}", y);
    }

    #[test]
    fn test_orthographic_rotation_roundtrip() {
        // Test that project + invert returns the original point
        // D3 convention: rotate([λ, φ, γ]) centers the projection on (-λ, -φ)
        // So rotate(100, 40, 0) centers on (-100, -40)
        let proj = OrthographicProjection::new()
            .scale(200.0)
            .translate(300.0, 200.0)
            .rotate(100.0, 40.0, 0.0);

        // Test with the center point (which should be at translate after projection)
        let center = (-100.0, -40.0);
        let (x, y) = proj.project(center.0, center.1);

        // Center should project to translate point
        assert!(
            (x - 300.0).abs() < 1.0,
            "Center X should be at translate, got {}",
            x
        );
        assert!(
            (y - 200.0).abs() < 1.0,
            "Center Y should be at translate, got {}",
            y
        );

        // Invert should get back to center
        let (lon, lat) = proj.invert(x, y);
        assert!(
            (lon - center.0).abs() < 0.1,
            "Longitude roundtrip failed: {} -> {}",
            center.0,
            lon
        );
        assert!(
            (lat - center.1).abs() < 0.1,
            "Latitude roundtrip failed: {} -> {}",
            center.1,
            lat
        );

        // Also test a non-center point on the visible side
        let point = (-80.0, -30.0);
        let (px, py) = proj.project(point.0, point.1);
        let (plon, plat) = proj.invert(px, py);

        assert!(
            (plon - point.0).abs() < 0.5,
            "Off-center longitude roundtrip failed: {} -> {}",
            point.0,
            plon
        );
        assert!(
            (plat - point.1).abs() < 0.5,
            "Off-center latitude roundtrip failed: {} -> {}",
            point.1,
            plat
        );
    }

    #[test]
    fn test_orthographic_rotation_with_gamma() {
        // γ rotation applies roll after λ and φ
        let proj_no_roll = OrthographicProjection::new()
            .scale(100.0)
            .translate(0.0, 0.0)
            .rotate(-90.0, 0.0, 0.0);

        let proj_with_roll = OrthographicProjection::new()
            .scale(100.0)
            .translate(0.0, 0.0)
            .rotate(-90.0, 0.0, 45.0); // 45° roll

        // Point at the center should be the same
        let (x1, y1) = proj_no_roll.project(90.0, 0.0);
        let (x2, y2) = proj_with_roll.project(90.0, 0.0);
        assert!(x1.abs() < 0.01);
        assert!(y1.abs() < 0.01);
        assert!(x2.abs() < 0.01);
        assert!(y2.abs() < 0.01);

        // But off-center points should be different
        let (x1, y1) = proj_no_roll.project(90.0, 30.0);
        let (x2, y2) = proj_with_roll.project(90.0, 30.0);
        // With roll, the point should be rotated around the center
        assert!(
            (x1 - x2).abs() > 0.1 || (y1 - y2).abs() > 0.1,
            "Roll should affect off-center points"
        );
    }

    #[test]
    fn test_albers_new() {
        let proj = AlbersProjection::new();
        assert_eq!(proj.projection_type(), "albers");
    }

    #[test]
    fn test_albers_usa() {
        let proj = AlbersProjection::usa()
            .scale(1000.0)
            .translate(480.0, 300.0);

        // Center point should project near translate
        let (x, y) = proj.project(-98.0, 39.0);
        assert!((x - 480.0).abs() < 50.0);
        assert!((y - 300.0).abs() < 50.0);
    }

    #[test]
    fn test_albers_roundtrip() {
        let proj = AlbersProjection::usa()
            .scale(1000.0)
            .translate(480.0, 300.0);

        let original = (-100.0, 40.0);
        let (x, y) = proj.project(original.0, original.1);
        let (lon, lat) = proj.invert(x, y);

        assert!((lon - original.0).abs() < 0.1);
        assert!((lat - original.1).abs() < 0.1);
    }

    #[test]
    fn test_fit_extent_mercator() {
        // Simple bounding box
        let bounds = vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]];

        let proj = MercatorProjection::new().fit_extent(&bounds, [[0.0, 0.0], [100.0, 100.0]]);

        // All points should now project within the extent
        for coord in &bounds {
            let (x, y) = proj.project(coord[0], coord[1]);
            assert!(x >= -1.0 && x <= 101.0, "x={} out of bounds", x);
            assert!(y >= -1.0 && y <= 101.0, "y={} out of bounds", y);
        }
    }

    #[test]
    fn test_fit_size_equirectangular() {
        let bounds = vec![[-10.0, -10.0], [10.0, -10.0], [10.0, 10.0], [-10.0, 10.0]];

        let proj = EquirectangularProjection::new().fit_size(&bounds, 200.0, 200.0);

        // Center should project near center of extent
        let (x, y) = proj.project(0.0, 0.0);
        assert!((x - 100.0).abs() < 10.0, "Center x={} not near 100", x);
        assert!((y - 100.0).abs() < 10.0, "Center y={} not near 100", y);
    }

    #[test]
    fn test_compute_fit_extent_empty() {
        let proj = MercatorProjection::new();
        let (scale, tx, ty) = compute_fit_extent(&proj, &[], [[0.0, 0.0], [100.0, 100.0]]);

        // Should return defaults for empty input
        assert_eq!(scale, 1.0);
        assert_eq!(tx, 0.0);
        assert_eq!(ty, 0.0);
    }

    #[test]
    fn test_project_bounds() {
        let proj = EquirectangularProjection::new()
            .scale(1.0)
            .translate(0.0, 0.0);

        let coords = vec![[0.0, 0.0], [90.0, 0.0], [90.0, 45.0], [0.0, 45.0]];

        let bounds = project_bounds(&proj, &coords);
        assert!(bounds.is_some());

        let [[min_x, min_y], [max_x, max_y]] = bounds.unwrap();
        assert!(min_x < max_x);
        assert!(min_y < max_y);
    }
}

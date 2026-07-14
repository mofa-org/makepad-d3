//! Cubehelix color scheme
//!
//! Cubehelix is a color scheme designed by Dave Green that spirals through
//! the color cube with monotonically increasing lightness. This makes it
//! ideal for representing continuous data while remaining readable when
//! printed in grayscale.
//!
//! # Reference
//! Green, D. A., 2011, "A colour scheme for the display of astronomical
//! intensity images", Bulletin of the Astronomical Society of India, 39, 289
//!
//! # Example
//! ```
//! use makepad_d3::color::cubehelix::{Cubehelix, cubehelix_default};
//!
//! // Get color at position 0.5
//! let color = cubehelix_default(0.5);
//!
//! // Create custom cubehelix
//! let custom = Cubehelix::new()
//!     .start(200.0)
//!     .rotations(-1.5)
//!     .hue(1.2)
//!     .gamma(1.0);
//! let color = custom.color(0.5);
//! ```

use super::types::Rgba;
use std::f32::consts::PI;
use std::sync::Arc;

/// Cubehelix interpolator
///
/// Generates colors along a helix through the RGB color cube,
/// with monotonically increasing lightness.
#[derive(Clone, Debug)]
pub struct Cubehelix {
    /// Starting hue angle in degrees
    h: f32,
    /// Number of rotations (can be negative)
    s: f32,
    /// Saturation multiplier (0-1 typical)
    gamma: f32,
    /// Lightness range start
    l0: f32,
    /// Lightness range end
    l1: f32,
    /// Amplitude/saturation
    amp: f32,
}

impl Default for Cubehelix {
    fn default() -> Self {
        Self::new()
    }
}

impl Cubehelix {
    /// Create a new cubehelix with default parameters
    ///
    /// Default: start=300, rotations=-1.5, hue=1.0, gamma=1.0
    pub fn new() -> Self {
        Self {
            h: 300.0,
            s: -1.5,
            gamma: 1.0,
            l0: 0.0,
            l1: 1.0,
            amp: 1.0,
        }
    }

    /// Set the starting hue angle (in degrees)
    pub fn start(mut self, h: f32) -> Self {
        self.h = h;
        self
    }

    /// Set the number of rotations through the color space
    ///
    /// Positive values rotate through red-green-blue.
    /// Negative values rotate the opposite direction.
    pub fn rotations(mut self, s: f32) -> Self {
        self.s = s;
        self
    }

    /// Set the saturation/hue intensity (0 = grayscale, 1 = full saturation)
    pub fn hue(mut self, amp: f32) -> Self {
        self.amp = amp.max(0.0);
        self
    }

    /// Set the gamma correction factor
    ///
    /// Values < 1 emphasize low-intensity values.
    /// Values > 1 emphasize high-intensity values.
    pub fn gamma(mut self, gamma: f32) -> Self {
        self.gamma = gamma.max(0.01);
        self
    }

    /// Set the lightness range (default: 0.0 to 1.0)
    pub fn light_range(mut self, l0: f32, l1: f32) -> Self {
        self.l0 = l0.clamp(0.0, 1.0);
        self.l1 = l1.clamp(0.0, 1.0);
        self
    }

    /// Get color at position t (0 to 1)
    pub fn color(&self, t: f64) -> Rgba {
        let t = t.clamp(0.0, 1.0) as f32;

        // Apply gamma
        let t_gamma = t.powf(self.gamma);

        // Calculate lightness
        let l = self.l0 + t_gamma * (self.l1 - self.l0);

        // Calculate hue angle
        let angle = 2.0 * PI * (self.h / 360.0 + self.s * t);

        // Calculate amplitude based on lightness
        // (reduces at extremes to stay in gamut)
        let amp = self.amp * l * (1.0 - l);

        // Cubehelix coefficients
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        // RGB coefficients for cubehelix
        const A: f32 = -0.14861;
        const B: f32 = 1.78277;
        const C: f32 = -0.29227;
        const D: f32 = -0.90649;
        const E: f32 = 1.97294;

        let r = l + amp * (A * cos_a + B * sin_a);
        let g = l + amp * (C * cos_a + D * sin_a);
        let b = l + amp * (E * cos_a);

        Rgba::rgb(r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0))
    }

    /// Create an interpolator function
    pub fn interpolator(&self) -> Arc<dyn Fn(f64) -> Rgba + Send + Sync> {
        let cubehelix = self.clone();
        Arc::new(move |t| cubehelix.color(t))
    }

    /// Generate a vector of n colors
    pub fn colors(&self, n: usize) -> Vec<Rgba> {
        if n == 0 {
            return Vec::new();
        }
        if n == 1 {
            return vec![self.color(0.5)];
        }

        (0..n)
            .map(|i| self.color(i as f64 / (n - 1) as f64))
            .collect()
    }
}

/// Default cubehelix color (Dave Green's original)
pub fn cubehelix_default(t: f64) -> Rgba {
    Cubehelix::new().color(t)
}

/// "Cool" cubehelix variant (cyan to magenta)
pub fn cubehelix_cool(t: f64) -> Rgba {
    Cubehelix::new()
        .start(260.0)
        .rotations(1.5)
        .hue(0.8)
        .color(t)
}

/// "Warm" cubehelix variant (magenta to orange)
pub fn cubehelix_warm(t: f64) -> Rgba {
    Cubehelix::new()
        .start(80.0)
        .rotations(-1.5)
        .hue(1.0)
        .color(t)
}

/// Rainbow cubehelix (full rotation)
pub fn cubehelix_rainbow(t: f64) -> Rgba {
    Cubehelix::new()
        .start(0.0)
        .rotations(1.0)
        .hue(1.5)
        .gamma(1.0)
        .color(t)
}

/// Sinebow (sinusoidal rainbow) color scheme
///
/// Similar to cubehelix but uses sine waves for each channel.
pub fn sinebow(t: f64) -> Rgba {
    let t = t.clamp(0.0, 1.0) as f32;
    let t = (0.5 - t) * PI;

    Rgba::rgb(
        (t.sin()).powi(2),
        ((t + PI / 3.0).sin()).powi(2),
        ((t + 2.0 * PI / 3.0).sin()).powi(2),
    )
}

/// Turbo color scheme (like jet but perceptually uniform)
///
/// Google's turbo colormap - starts dark blue, goes through cyan, green, yellow to red.
/// Uses a simplified lookup with interpolation for efficiency.
pub fn turbo(t: f64) -> Rgba {
    let t = t.clamp(0.0, 1.0) as f32;

    // Key points from Google's turbo colormap (simplified 9-point version)
    const TURBO_COLORS: [(f32, f32, f32); 9] = [
        (0.190, 0.072, 0.232), // t=0.0    - dark blue/purple
        (0.129, 0.298, 0.697), // t=0.125  - blue
        (0.094, 0.539, 0.750), // t=0.25   - cyan-blue
        (0.137, 0.718, 0.565), // t=0.375  - cyan-green
        (0.365, 0.827, 0.322), // t=0.5    - green
        (0.631, 0.854, 0.133), // t=0.625  - yellow-green
        (0.892, 0.737, 0.099), // t=0.75   - yellow-orange
        (0.989, 0.460, 0.157), // t=0.875  - orange
        (0.479, 0.016, 0.011), // t=1.0    - dark red
    ];

    // Find segment and interpolate
    let scaled = t * 8.0;
    let i = (scaled.floor() as usize).min(7);
    let local_t = scaled - i as f32;

    let (r1, g1, b1) = TURBO_COLORS[i];
    let (r2, g2, b2) = TURBO_COLORS[i + 1];

    Rgba::rgb(
        r1 + (r2 - r1) * local_t,
        g1 + (g2 - g1) * local_t,
        b1 + (b2 - b1) * local_t,
    )
}

/// Create a cubehelix interpolator function
pub fn interpolator_cubehelix(
    start: f32,
    rotations: f32,
    hue: f32,
    gamma: f32,
) -> Arc<dyn Fn(f64) -> Rgba + Send + Sync> {
    let ch = Cubehelix::new()
        .start(start)
        .rotations(rotations)
        .hue(hue)
        .gamma(gamma);
    ch.interpolator()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cubehelix_default() {
        let ch = Cubehelix::new();

        // Start should be dark
        let start = ch.color(0.0);
        assert!(start.r < 0.1 && start.g < 0.1 && start.b < 0.1);

        // End should be bright
        let end = ch.color(1.0);
        assert!(end.r > 0.9 && end.g > 0.9 && end.b > 0.9);

        // Middle should be colorful
        let mid = ch.color(0.5);
        assert!(mid.r >= 0.0 && mid.r <= 1.0);
    }

    #[test]
    fn test_cubehelix_monotonic_lightness() {
        let ch = Cubehelix::new();

        let mut prev_l = 0.0;
        for i in 0..=10 {
            let t = i as f64 / 10.0;
            let c = ch.color(t);

            // Approximate lightness from RGB
            let l = 0.299 * c.r + 0.587 * c.g + 0.114 * c.b;

            assert!(l >= prev_l - 0.01, "Lightness should be monotonic");
            prev_l = l;
        }
    }

    #[test]
    fn test_cubehelix_builder() {
        let ch = Cubehelix::new()
            .start(180.0)
            .rotations(2.0)
            .hue(1.5)
            .gamma(0.8);

        let mid = ch.color(0.5);
        assert!(mid.r >= 0.0 && mid.r <= 1.0);
        assert!(mid.g >= 0.0 && mid.g <= 1.0);
        assert!(mid.b >= 0.0 && mid.b <= 1.0);
    }

    #[test]
    fn test_cubehelix_grayscale() {
        // With hue=0, should produce grayscale
        let ch = Cubehelix::new().hue(0.0);

        let mid = ch.color(0.5);
        // All channels should be equal (gray)
        assert!((mid.r - mid.g).abs() < 0.01);
        assert!((mid.g - mid.b).abs() < 0.01);
    }

    #[test]
    fn test_cubehelix_colors() {
        let ch = Cubehelix::new();
        let colors = ch.colors(5);

        assert_eq!(colors.len(), 5);

        // First should be darkest
        let first_l = 0.299 * colors[0].r + 0.587 * colors[0].g + 0.114 * colors[0].b;
        let last_l = 0.299 * colors[4].r + 0.587 * colors[4].g + 0.114 * colors[4].b;
        assert!(last_l > first_l);
    }

    #[test]
    fn test_sinebow() {
        let start = sinebow(0.0);
        let mid = sinebow(0.5);
        let end = sinebow(1.0);

        // All should be valid colors
        assert!(start.r >= 0.0 && start.r <= 1.0);
        assert!(mid.r >= 0.0 && mid.r <= 1.0);
        assert!(end.r >= 0.0 && end.r <= 1.0);

        // Start and end should be similar (cyclic)
        assert!((start.r - end.r).abs() < 0.1);
    }

    #[test]
    fn test_turbo() {
        let start = turbo(0.0);
        let mid = turbo(0.5);
        let end = turbo(1.0);

        // All should be valid colors
        assert!(start.r >= 0.0 && start.r <= 1.0);
        assert!(mid.g >= 0.0 && mid.g <= 1.0);
        assert!(end.b >= 0.0 && end.b <= 1.0);

        // Turbo starts blue-ish and ends red-ish
        assert!(start.b > start.r);
        assert!(end.r > end.b);
    }

    #[test]
    fn test_cubehelix_light_range() {
        let ch = Cubehelix::new().light_range(0.2, 0.8);

        let start = ch.color(0.0);
        let end = ch.color(1.0);

        // Start should not be pure black
        let start_l = 0.299 * start.r + 0.587 * start.g + 0.114 * start.b;
        assert!(start_l > 0.1);

        // End should not be pure white
        let end_l = 0.299 * end.r + 0.587 * end.g + 0.114 * end.b;
        assert!(end_l < 0.9);
    }
}

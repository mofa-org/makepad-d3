//! Color interpolation in multiple color spaces
//!
//! This module provides interpolation functions for creating smooth color
//! transitions. Different color spaces produce different visual results:
//!
//! - **RGB**: Simple but can produce muddy colors at midpoints
//! - **HSL**: Maintains saturation but can have hue jumps
//! - **Lab**: Perceptually uniform, good for scientific visualizations
//! - **HCL**: Like Lab but with intuitive hue control
//!
//! # Example
//! ```
//! use makepad_d3::color::{Rgba, interpolate_rgb, interpolate_lab};
//!
//! let red = Rgba::RED;
//! let blue = Rgba::BLUE;
//!
//! // RGB interpolation
//! let mid_rgb = interpolate_rgb(&red, &blue, 0.5);
//!
//! // Lab interpolation (perceptually uniform)
//! let mid_lab = interpolate_lab(&red, &blue, 0.5);
//! ```

use super::hcl::{Hcl, HueInterpolation};
use super::lab::Lab;
use super::types::{Hsl, Rgba};
use std::sync::Arc;

/// Interpolation function type
pub type InterpolateFn = Arc<dyn Fn(f64) -> Rgba + Send + Sync>;

/// Color space for interpolation
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ColorSpace {
    /// RGB color space (linear interpolation)
    #[default]
    Rgb,
    /// HSL color space (hue-based)
    Hsl,
    /// Lab color space (perceptually uniform)
    Lab,
    /// HCL color space (perceptually uniform with hue)
    Hcl,
}

/// Interpolate between two colors in RGB space
pub fn interpolate_rgb(a: &Rgba, b: &Rgba, t: f64) -> Rgba {
    a.lerp(b, t as f32)
}

/// Interpolate between two colors in HSL space
///
/// Preserves alpha channel by interpolating it separately.
pub fn interpolate_hsl(a: &Rgba, b: &Rgba, t: f64) -> Rgba {
    let hsl_a = Hsl::from_rgba(a);
    let hsl_b = Hsl::from_rgba(b);

    let t = t.clamp(0.0, 1.0) as f32;

    // Handle hue interpolation (shortest path)
    let mut h_diff = hsl_b.h - hsl_a.h;
    if h_diff > 180.0 {
        h_diff -= 360.0;
    } else if h_diff < -180.0 {
        h_diff += 360.0;
    }

    let h = (hsl_a.h + h_diff * t) % 360.0;
    let h = if h < 0.0 { h + 360.0 } else { h };

    let s = hsl_a.s + (hsl_b.s - hsl_a.s) * t;
    let l = hsl_a.l + (hsl_b.l - hsl_a.l) * t;

    // Interpolate alpha separately (HSL doesn't have alpha)
    let alpha = a.a + (b.a - a.a) * t;

    Hsl::new(h, s, l).to_rgba().with_alpha(alpha)
}

/// Interpolate between two colors in Lab space (perceptually uniform)
pub fn interpolate_lab(a: &Rgba, b: &Rgba, t: f64) -> Rgba {
    let lab_a = Lab::from_rgba(a);
    let lab_b = Lab::from_rgba(b);
    lab_a.lerp(&lab_b, t as f32).to_rgba()
}

/// Interpolate between two colors in HCL space
pub fn interpolate_hcl(a: &Rgba, b: &Rgba, t: f64) -> Rgba {
    let hcl_a = Hcl::from_rgba(a);
    let hcl_b = Hcl::from_rgba(b);
    hcl_a.lerp(&hcl_b, t as f32).to_rgba()
}

/// Interpolate between two colors in HCL space with specified hue mode
pub fn interpolate_hcl_long(a: &Rgba, b: &Rgba, t: f64) -> Rgba {
    let hcl_a = Hcl::from_rgba(a);
    let hcl_b = Hcl::from_rgba(b);
    hcl_a
        .lerp_with_hue_mode(&hcl_b, t as f32, HueInterpolation::Longer)
        .to_rgba()
}

/// Interpolate in the specified color space
pub fn interpolate(a: &Rgba, b: &Rgba, t: f64, space: ColorSpace) -> Rgba {
    match space {
        ColorSpace::Rgb => interpolate_rgb(a, b, t),
        ColorSpace::Hsl => interpolate_hsl(a, b, t),
        ColorSpace::Lab => interpolate_lab(a, b, t),
        ColorSpace::Hcl => interpolate_hcl(a, b, t),
    }
}

/// Create an interpolator function between two colors
pub fn interpolator(a: Rgba, b: Rgba, space: ColorSpace) -> InterpolateFn {
    Arc::new(move |t| interpolate(&a, &b, t, space))
}

/// Create an interpolator through multiple colors
pub fn interpolator_multi(colors: Vec<Rgba>, space: ColorSpace) -> InterpolateFn {
    Arc::new(move |t| {
        let t = t.clamp(0.0, 1.0);

        if colors.is_empty() {
            return Rgba::BLACK;
        }
        if colors.len() == 1 {
            return colors[0];
        }

        let n = colors.len() - 1;
        let scaled = t * n as f64;
        let i = (scaled.floor() as usize).min(n - 1);
        let local_t = scaled - i as f64;

        interpolate(&colors[i], &colors[i + 1], local_t, space)
    })
}

/// Gamma-corrected RGB interpolation
///
/// Interpolates in linear RGB space then applies gamma correction.
/// This can produce more natural-looking gradients than standard RGB.
pub fn interpolate_rgb_gamma(a: &Rgba, b: &Rgba, t: f64, gamma: f32) -> Rgba {
    let t = t.clamp(0.0, 1.0) as f32;

    // Convert to linear space
    let a_lin = (a.r.powf(gamma), a.g.powf(gamma), a.b.powf(gamma));
    let b_lin = (b.r.powf(gamma), b.g.powf(gamma), b.b.powf(gamma));

    // Interpolate in linear space
    let r = a_lin.0 + (b_lin.0 - a_lin.0) * t;
    let g = a_lin.1 + (b_lin.1 - a_lin.1) * t;
    let bl = a_lin.2 + (b_lin.2 - a_lin.2) * t;
    let alpha = a.a + (b.a - a.a) * t;

    // Convert back to gamma space
    let inv_gamma = 1.0 / gamma;
    Rgba::new(
        r.powf(inv_gamma).clamp(0.0, 1.0),
        g.powf(inv_gamma).clamp(0.0, 1.0),
        bl.powf(inv_gamma).clamp(0.0, 1.0),
        alpha,
    )
}

/// Create an RGB gamma interpolator (commonly gamma=2.2 for sRGB)
pub fn interpolator_rgb_gamma(a: Rgba, b: Rgba, gamma: f32) -> InterpolateFn {
    Arc::new(move |t| interpolate_rgb_gamma(&a, &b, t, gamma))
}

/// Basis spline interpolation through colors
///
/// Creates smooth curves through the control points, useful for
/// natural-looking gradients.
pub fn interpolate_basis(colors: &[Rgba], t: f64) -> Rgba {
    if colors.is_empty() {
        return Rgba::BLACK;
    }
    if colors.len() == 1 {
        return colors[0];
    }
    if colors.len() == 2 {
        return interpolate_rgb(&colors[0], &colors[1], t);
    }

    let t = t.clamp(0.0, 1.0);
    let n = colors.len() - 1;
    let i = ((t * n as f64).floor() as usize).min(n - 1);

    // Get four control points for cubic basis spline
    let p0 = if i > 0 { colors[i - 1] } else { colors[0] };
    let p1 = colors[i];
    let p2 = colors[i + 1];
    let p3 = if i + 2 < colors.len() {
        colors[i + 2]
    } else {
        colors[colors.len() - 1]
    };

    let local_t = (t * n as f64) - i as f64;
    basis_spline_point(p0, p1, p2, p3, local_t as f32)
}

/// Compute basis spline point
fn basis_spline_point(p0: Rgba, p1: Rgba, p2: Rgba, p3: Rgba, t: f32) -> Rgba {
    let t2 = t * t;
    let t3 = t2 * t;

    // Basis function coefficients
    let b0 = (1.0 - t).powi(3) / 6.0;
    let b1 = (3.0 * t3 - 6.0 * t2 + 4.0) / 6.0;
    let b2 = (-3.0 * t3 + 3.0 * t2 + 3.0 * t + 1.0) / 6.0;
    let b3 = t3 / 6.0;

    Rgba::new(
        (p0.r * b0 + p1.r * b1 + p2.r * b2 + p3.r * b3).clamp(0.0, 1.0),
        (p0.g * b0 + p1.g * b1 + p2.g * b2 + p3.g * b3).clamp(0.0, 1.0),
        (p0.b * b0 + p1.b * b1 + p2.b * b2 + p3.b * b3).clamp(0.0, 1.0),
        (p0.a * b0 + p1.a * b1 + p2.a * b2 + p3.a * b3).clamp(0.0, 1.0),
    )
}

/// Discrete/quantize interpolation (step function)
pub fn interpolate_discrete(colors: &[Rgba], t: f64) -> Rgba {
    if colors.is_empty() {
        return Rgba::BLACK;
    }

    let t = t.clamp(0.0, 1.0);
    let i = ((t * colors.len() as f64).floor() as usize).min(colors.len() - 1);
    colors[i]
}

/// Create a discrete interpolator
pub fn interpolator_discrete(colors: Vec<Rgba>) -> InterpolateFn {
    Arc::new(move |t| interpolate_discrete(&colors, t))
}

/// Piecewise interpolation with custom positions
///
/// Colors are positioned at specific t values, allowing non-uniform gradients.
pub fn interpolate_piecewise(colors: &[(f64, Rgba)], t: f64, space: ColorSpace) -> Rgba {
    if colors.is_empty() {
        return Rgba::BLACK;
    }
    if colors.len() == 1 {
        return colors[0].1;
    }

    let t = t.clamp(0.0, 1.0);

    // Find the segment
    for i in 0..colors.len() - 1 {
        let (t0, c0) = colors[i];
        let (t1, c1) = colors[i + 1];

        if t >= t0 && t <= t1 {
            let local_t = if (t1 - t0).abs() < f64::EPSILON {
                0.0
            } else {
                (t - t0) / (t1 - t0)
            };
            return interpolate(&c0, &c1, local_t, space);
        }
    }

    // Return last color if t > all positions
    colors.last().map(|(_, c)| *c).unwrap_or(Rgba::BLACK)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpolate_rgb() {
        let black = Rgba::BLACK;
        let white = Rgba::WHITE;

        let mid = interpolate_rgb(&black, &white, 0.5);
        assert!((mid.r - 0.5).abs() < 0.01);
        assert!((mid.g - 0.5).abs() < 0.01);
        assert!((mid.b - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_interpolate_lab() {
        let black = Rgba::BLACK;
        let white = Rgba::WHITE;

        let mid = interpolate_lab(&black, &white, 0.5);

        // Lab midpoint should be perceptually middle gray
        // (which is slightly different from RGB 0.5)
        assert!(mid.r > 0.1 && mid.r < 0.9);
    }

    #[test]
    fn test_interpolate_hsl() {
        let red = Rgba::RED;
        let green = Rgba::GREEN;

        // HSL interpolation should maintain saturation
        let mid = interpolate_hsl(&red, &green, 0.5);
        let mid_hsl = Hsl::from_rgba(&mid);

        // Mid should have high saturation
        assert!(mid_hsl.s > 0.5);
    }

    #[test]
    fn test_interpolate_hcl() {
        let red = Rgba::RED;
        let blue = Rgba::BLUE;

        let mid = interpolate_hcl(&red, &blue, 0.5);

        // Should produce a valid color
        assert!(mid.r >= 0.0 && mid.r <= 1.0);
        assert!(mid.g >= 0.0 && mid.g <= 1.0);
        assert!(mid.b >= 0.0 && mid.b <= 1.0);
    }

    #[test]
    fn test_interpolator_multi() {
        let colors = vec![Rgba::RED, Rgba::GREEN, Rgba::BLUE];
        let interp = interpolator_multi(colors, ColorSpace::Rgb);

        let start = interp(0.0);
        let mid = interp(0.5);
        let end = interp(1.0);

        // Start should be red-ish
        assert!(start.r > 0.9);
        // End should be blue-ish
        assert!(end.b > 0.9);
        // Mid should be around green
        assert!(mid.g > mid.r && mid.g > mid.b);
    }

    #[test]
    fn test_interpolate_discrete() {
        let colors = vec![Rgba::RED, Rgba::GREEN, Rgba::BLUE];

        let c0 = interpolate_discrete(&colors, 0.0);
        let c1 = interpolate_discrete(&colors, 0.33);
        let c2 = interpolate_discrete(&colors, 0.34);
        let c3 = interpolate_discrete(&colors, 0.67);
        let c4 = interpolate_discrete(&colors, 1.0);

        // Should step, not interpolate
        assert_eq!(c0.to_hex(), Rgba::RED.to_hex());
        assert_eq!(c1.to_hex(), Rgba::RED.to_hex());
        assert_eq!(c2.to_hex(), Rgba::GREEN.to_hex());
        assert_eq!(c3.to_hex(), Rgba::BLUE.to_hex());
        assert_eq!(c4.to_hex(), Rgba::BLUE.to_hex());
    }

    #[test]
    fn test_interpolate_basis() {
        let colors = vec![Rgba::RED, Rgba::GREEN, Rgba::BLUE];

        let start = interpolate_basis(&colors, 0.0);
        let end = interpolate_basis(&colors, 1.0);

        // Should produce valid colors
        assert!(start.r >= 0.0 && start.r <= 1.0);
        assert!(end.b >= 0.0 && end.b <= 1.0);
    }

    #[test]
    fn test_rgb_gamma() {
        let black = Rgba::BLACK;
        let white = Rgba::WHITE;

        let mid_linear = interpolate_rgb(&black, &white, 0.5);
        let mid_gamma = interpolate_rgb_gamma(&black, &white, 0.5, 2.2);

        // Gamma-corrected should be slightly different
        // (darker than linear at midpoint for gamma > 1)
        assert!((mid_linear.r - mid_gamma.r).abs() > 0.01);
    }

    #[test]
    fn test_piecewise() {
        let colors = vec![(0.0, Rgba::RED), (0.3, Rgba::GREEN), (1.0, Rgba::BLUE)];

        let at_0 = interpolate_piecewise(&colors, 0.0, ColorSpace::Rgb);
        let at_03 = interpolate_piecewise(&colors, 0.3, ColorSpace::Rgb);
        let at_1 = interpolate_piecewise(&colors, 1.0, ColorSpace::Rgb);

        assert_eq!(at_0.to_hex(), Rgba::RED.to_hex());
        assert_eq!(at_03.to_hex(), Rgba::GREEN.to_hex());
        assert_eq!(at_1.to_hex(), Rgba::BLUE.to_hex());
    }

    #[test]
    fn test_interpolator_function() {
        let interp = interpolator(Rgba::BLACK, Rgba::WHITE, ColorSpace::Rgb);

        let mid = interp(0.5);
        assert!((mid.r - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_hsl_preserves_alpha() {
        let color_a = Rgba::new(1.0, 0.0, 0.0, 0.2); // Red with 20% alpha
        let color_b = Rgba::new(0.0, 1.0, 0.0, 0.8); // Green with 80% alpha

        // Test at various t values
        let start = interpolate_hsl(&color_a, &color_b, 0.0);
        assert!(
            (start.a - 0.2).abs() < 0.01,
            "Start alpha should be 0.2, got {}",
            start.a
        );

        let mid = interpolate_hsl(&color_a, &color_b, 0.5);
        assert!(
            (mid.a - 0.5).abs() < 0.01,
            "Mid alpha should be 0.5, got {}",
            mid.a
        );

        let end = interpolate_hsl(&color_a, &color_b, 1.0);
        assert!(
            (end.a - 0.8).abs() < 0.01,
            "End alpha should be 0.8, got {}",
            end.a
        );
    }

    #[test]
    fn test_rgb_preserves_alpha() {
        let color_a = Rgba::new(0.0, 0.0, 0.0, 0.0); // Transparent black
        let color_b = Rgba::new(1.0, 1.0, 1.0, 1.0); // Opaque white

        let mid = interpolate_rgb(&color_a, &color_b, 0.5);
        assert!(
            (mid.a - 0.5).abs() < 0.01,
            "Alpha should be 0.5, got {}",
            mid.a
        );
    }
}

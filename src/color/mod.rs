//! Color system for data visualization
//!
//! This module provides comprehensive color functionality for data visualization:
//!
//! # Color Types
//!
//! - [`Rgba`]: RGBA color with f32 components
//! - [`Hsl`]: HSL color representation
//! - [`Lab`]: CIELAB perceptually uniform color space
//! - [`Hcl`]: HCL (polar Lab) for intuitive hue manipulation
//!
//! # Color Scales
//!
//! - [`SequentialScale`]: Continuous interpolation for quantitative data
//! - [`DivergingScale`]: Two-sided scales for data with a midpoint
//! - [`CategoricalScale`]: Distinct colors for categorical data
//!
//! # Color Interpolation
//!
//! - RGB, HSL, Lab, HCL interpolation methods
//! - Gamma-corrected RGB interpolation
//! - Basis spline interpolation for smooth gradients
//!
//! # Special Color Schemes
//!
//! - [`Cubehelix`]: Monotonic lightness with color variation
//! - `sinebow`, `turbo`: Perceptually uniform rainbow schemes
//!
//! # Color Operations
//!
//! - Blending modes (multiply, screen, overlay, etc.)
//! - Color mixing in RGB and Lab spaces
//! - Tint, shade, tone adjustments
//! - WCAG contrast checking
//!
//! # Example
//!
//! ```
//! use makepad_d3::color::{ColorScale, SequentialScale, CategoricalScale, Rgba};
//!
//! // Sequential scale for continuous data
//! let heat = SequentialScale::yellow_orange_red();
//! let low = heat.color(0.0);   // Yellow
//! let high = heat.color(1.0);  // Red
//!
//! // Categorical scale for discrete data
//! let categories = CategoricalScale::category10();
//! let color_a = categories.get(0);
//! let color_b = categories.get(1);
//! ```
//!
//! # Lab/HCL Example
//!
//! ```
//! use makepad_d3::color::{Lab, Hcl, Rgba, interpolate_lab};
//!
//! // Perceptually uniform interpolation
//! let red = Rgba::RED;
//! let blue = Rgba::BLUE;
//! let mid = interpolate_lab(&red, &blue, 0.5);
//!
//! // HCL for hue manipulation
//! let hcl = Hcl::from_rgba(&red);
//! let complement = hcl.complement().to_rgba();
//! ```

mod blend;
mod cubehelix;
mod hcl;
mod interpolate;
mod lab;
mod oklab;
mod scale;
mod types;

// Core color types
pub use types::{Hsl, Rgba};

// Color scales
pub use scale::{CategoricalScale, ColorScale, DivergingScale, SequentialScale};

// Perceptually uniform color spaces
pub use hcl::{Hcl, HueInterpolation};
pub use lab::Lab;
pub use oklab::{interpolate_oklab, interpolate_oklch, Oklab, Oklch};

// Interpolation functions
pub use interpolate::{
    interpolate, interpolate_basis, interpolate_discrete, interpolate_hcl, interpolate_hcl_long,
    interpolate_hsl, interpolate_lab, interpolate_piecewise, interpolate_rgb,
    interpolate_rgb_gamma, interpolator, interpolator_discrete, interpolator_multi,
    interpolator_rgb_gamma, ColorSpace, InterpolateFn,
};

// Cubehelix and special color schemes
pub use cubehelix::{
    cubehelix_cool, cubehelix_default, cubehelix_rainbow, cubehelix_warm, interpolator_cubehelix,
    sinebow, turbo, Cubehelix,
};

// Color blending and operations
pub use blend::{
    blend, blend_with_opacity, brightness, composite_over, contrast, contrast_ratio, grayscale,
    invert, luminance, meets_wcag_aa, meets_wcag_aaa, mix, mix_lab, mix_weighted, sepia, shade,
    tint, tone, BlendMode,
};

/// Interpolate between two colors
pub fn lerp_color(a: Rgba, b: Rgba, t: f32) -> Rgba {
    a.lerp(&b, t)
}

/// Create a color from hex value
pub fn hex(value: u32) -> Rgba {
    Rgba::from_hex(value)
}

/// Create an RGB color from components (0.0 to 1.0)
pub fn rgb(r: f32, g: f32, b: f32) -> Rgba {
    Rgba::rgb(r, g, b)
}

/// Create an RGBA color from components (0.0 to 1.0)
pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Rgba {
    Rgba::new(r, g, b, a)
}

/// Create an HSL color and convert to RGBA
pub fn hsl(h: f32, s: f32, l: f32) -> Rgba {
    Hsl::new(h, s, l).to_rgba()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_helper() {
        let c = hex(0xFF0000);
        assert!((c.r - 1.0).abs() < 0.01);
        assert!((c.g - 0.0).abs() < 0.01);
        assert!((c.b - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_rgb_helper() {
        let c = rgb(1.0, 0.5, 0.0);
        assert!((c.r - 1.0).abs() < 0.01);
        assert!((c.g - 0.5).abs() < 0.01);
        assert!((c.b - 0.0).abs() < 0.01);
        assert!((c.a - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_hsl_helper() {
        let c = hsl(0.0, 1.0, 0.5); // Red
        assert!((c.r - 1.0).abs() < 0.01);
        assert!((c.g - 0.0).abs() < 0.01);
        assert!((c.b - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_lerp_color_helper() {
        let black = Rgba::BLACK;
        let white = Rgba::WHITE;
        let mid = lerp_color(black, white, 0.5);

        assert!((mid.r - 0.5).abs() < 0.01);
        assert!((mid.g - 0.5).abs() < 0.01);
        assert!((mid.b - 0.5).abs() < 0.01);
    }
}

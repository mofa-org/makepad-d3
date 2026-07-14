//! Oklab color space implementation
//!
//! Oklab is a perceptually uniform color space created by Björn Ottosson.
//! It provides better perceptual uniformity than CIELAB and is simpler to compute.
//!
//! # References
//! - https://bottosson.github.io/posts/oklab/

use super::types::Rgba;

/// Oklab color representation
///
/// A perceptually uniform color space where:
/// - `l`: Lightness (0.0 = black, 1.0 = white)
/// - `a`: Green-red axis (negative = green, positive = red)
/// - `b`: Blue-yellow axis (negative = blue, positive = yellow)
/// - `alpha`: Alpha/opacity (0.0 = transparent, 1.0 = opaque)
///
/// # Example
///
/// ```
/// use makepad_d3::color::{Rgba, Oklab};
///
/// let red = Rgba::RED;
/// let oklab = Oklab::from_rgba(&red);
///
/// // Oklab provides perceptually uniform distances
/// let blue = Rgba::BLUE;
/// let oklab_blue = Oklab::from_rgba(&blue);
///
/// // Interpolation in Oklab space is perceptually uniform
/// let mid = oklab.lerp(&oklab_blue, 0.5);
/// let mid_rgb = mid.to_rgba();
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Oklab {
    /// Lightness (0.0 to 1.0)
    pub l: f32,
    /// Green-red axis (approximately -0.4 to 0.4)
    pub a: f32,
    /// Blue-yellow axis (approximately -0.4 to 0.4)
    pub b: f32,
    /// Alpha/opacity (0.0 to 1.0)
    pub alpha: f32,
}

impl Oklab {
    /// Create a new Oklab color with alpha = 1.0
    pub fn new(l: f32, a: f32, b: f32) -> Self {
        Self {
            l,
            a,
            b,
            alpha: 1.0,
        }
    }

    /// Create a new Oklab color with specified alpha
    pub fn with_alpha(l: f32, a: f32, b: f32, alpha: f32) -> Self {
        Self { l, a, b, alpha }
    }

    /// Convert from RGBA to Oklab
    pub fn from_rgba(rgba: &Rgba) -> Self {
        // Convert sRGB to linear RGB
        let r = srgb_to_linear(rgba.r);
        let g = srgb_to_linear(rgba.g);
        let b = srgb_to_linear(rgba.b);

        // Convert linear RGB to LMS
        let l = 0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b;
        let m = 0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b;
        let s = 0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b;

        // Apply cube root
        let l_ = l.cbrt();
        let m_ = m.cbrt();
        let s_ = s.cbrt();

        // Convert to Oklab, preserving alpha
        Self {
            l: 0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_,
            a: 1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_,
            b: 0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_,
            alpha: rgba.a,
        }
    }

    /// Convert from Oklab to RGBA
    pub fn to_rgba(&self) -> Rgba {
        // Convert Oklab to LMS'
        let l_ = self.l + 0.3963377774 * self.a + 0.2158037573 * self.b;
        let m_ = self.l - 0.1055613458 * self.a - 0.0638541728 * self.b;
        let s_ = self.l - 0.0894841775 * self.a - 1.2914855480 * self.b;

        // Apply cube
        let l = l_ * l_ * l_;
        let m = m_ * m_ * m_;
        let s = s_ * s_ * s_;

        // Convert LMS to linear RGB
        let r = 4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s;
        let g = -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s;
        let b = -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s;

        // Convert linear RGB to sRGB, preserving alpha
        Rgba::new(
            linear_to_srgb(r),
            linear_to_srgb(g),
            linear_to_srgb(b),
            self.alpha,
        )
    }

    /// Linear interpolation between two Oklab colors
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        Self {
            l: self.l + (other.l - self.l) * t,
            a: self.a + (other.a - self.a) * t,
            b: self.b + (other.b - self.b) * t,
            alpha: self.alpha + (other.alpha - self.alpha) * t,
        }
    }

    /// Calculate the perceptual distance between two colors
    ///
    /// This is the Euclidean distance in Oklab space, which correlates
    /// well with perceived color difference.
    pub fn distance(&self, other: &Self) -> f32 {
        let dl = self.l - other.l;
        let da = self.a - other.a;
        let db = self.b - other.b;
        (dl * dl + da * da + db * db).sqrt()
    }

    /// Adjust lightness while preserving hue, chroma, and alpha
    pub fn with_lightness(&self, l: f32) -> Self {
        Self { l, ..*self }
    }

    /// Set alpha while preserving color
    pub fn set_alpha(&self, alpha: f32) -> Self {
        Self { alpha, ..*self }
    }

    /// Scale the chroma (colorfulness) while preserving lightness, hue, and alpha
    pub fn scale_chroma(&self, factor: f32) -> Self {
        Self {
            l: self.l,
            a: self.a * factor,
            b: self.b * factor,
            alpha: self.alpha,
        }
    }

    /// Get the chroma (colorfulness) of the color
    pub fn chroma(&self) -> f32 {
        (self.a * self.a + self.b * self.b).sqrt()
    }

    /// Get the hue angle in radians
    pub fn hue(&self) -> f32 {
        self.b.atan2(self.a)
    }
}

/// Oklch color representation (polar form of Oklab)
///
/// - `l`: Lightness (0.0 to 1.0)
/// - `c`: Chroma (0.0 to ~0.4)
/// - `h`: Hue in radians (0 to 2π)
/// - `alpha`: Alpha/opacity (0.0 to 1.0)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Oklch {
    /// Lightness (0.0 to 1.0)
    pub l: f32,
    /// Chroma (colorfulness)
    pub c: f32,
    /// Hue in radians
    pub h: f32,
    /// Alpha/opacity (0.0 to 1.0)
    pub alpha: f32,
}

impl Oklch {
    /// Create a new Oklch color with alpha = 1.0
    pub fn new(l: f32, c: f32, h: f32) -> Self {
        Self {
            l,
            c,
            h,
            alpha: 1.0,
        }
    }

    /// Create a new Oklch color with specified alpha
    pub fn with_alpha(l: f32, c: f32, h: f32, alpha: f32) -> Self {
        Self { l, c, h, alpha }
    }

    /// Convert from Oklab to Oklch
    pub fn from_oklab(oklab: &Oklab) -> Self {
        Self {
            l: oklab.l,
            c: oklab.chroma(),
            h: oklab.hue(),
            alpha: oklab.alpha,
        }
    }

    /// Convert from RGBA to Oklch
    pub fn from_rgba(rgba: &Rgba) -> Self {
        Self::from_oklab(&Oklab::from_rgba(rgba))
    }

    /// Convert to Oklab
    pub fn to_oklab(&self) -> Oklab {
        Oklab {
            l: self.l,
            a: self.c * self.h.cos(),
            b: self.c * self.h.sin(),
            alpha: self.alpha,
        }
    }

    /// Convert to RGBA
    pub fn to_rgba(&self) -> Rgba {
        self.to_oklab().to_rgba()
    }

    /// Linear interpolation with shortest hue path
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        // Interpolate hue along shortest path
        let mut dh = other.h - self.h;
        if dh > std::f32::consts::PI {
            dh -= 2.0 * std::f32::consts::PI;
        } else if dh < -std::f32::consts::PI {
            dh += 2.0 * std::f32::consts::PI;
        }

        Self {
            l: self.l + (other.l - self.l) * t,
            c: self.c + (other.c - self.c) * t,
            h: normalize_hue(self.h + dh * t),
            alpha: self.alpha + (other.alpha - self.alpha) * t,
        }
    }

    /// Adjust hue while preserving lightness, chroma, and alpha
    pub fn with_hue(&self, h: f32) -> Self {
        Self {
            h: normalize_hue(h),
            ..*self
        }
    }

    /// Set alpha while preserving color
    pub fn set_alpha(&self, alpha: f32) -> Self {
        Self { alpha, ..*self }
    }

    /// Get the complementary color (opposite hue)
    pub fn complement(&self) -> Self {
        Self {
            h: normalize_hue(self.h + std::f32::consts::PI),
            ..*self
        }
    }
}

/// Normalize hue to [0, 2π) range
fn normalize_hue(h: f32) -> f32 {
    let two_pi = 2.0 * std::f32::consts::PI;
    let mut h = h % two_pi;
    if h < 0.0 {
        h += two_pi;
    }
    h
}

/// Convert sRGB component to linear RGB
fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Convert linear RGB component to sRGB
fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
    .clamp(0.0, 1.0)
}

/// Interpolate between two RGBA colors in Oklab space
///
/// This provides perceptually uniform interpolation.
pub fn interpolate_oklab(a: &Rgba, b: &Rgba, t: f32) -> Rgba {
    let oklab_a = Oklab::from_rgba(a);
    let oklab_b = Oklab::from_rgba(b);
    oklab_a.lerp(&oklab_b, t).to_rgba()
}

/// Interpolate between two RGBA colors in Oklch space
///
/// This provides perceptually uniform interpolation with hue interpolation
/// along the shortest path.
pub fn interpolate_oklch(a: &Rgba, b: &Rgba, t: f32) -> Rgba {
    let oklch_a = Oklch::from_rgba(a);
    let oklch_b = Oklch::from_rgba(b);
    oklch_a.lerp(&oklch_b, t).to_rgba()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oklab_from_rgba_black() {
        let black = Rgba::BLACK;
        let oklab = Oklab::from_rgba(&black);
        assert!((oklab.l - 0.0).abs() < 0.01);
        assert!((oklab.a - 0.0).abs() < 0.01);
        assert!((oklab.b - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_oklab_from_rgba_white() {
        let white = Rgba::WHITE;
        let oklab = Oklab::from_rgba(&white);
        assert!((oklab.l - 1.0).abs() < 0.01);
        assert!((oklab.a - 0.0).abs() < 0.01);
        assert!((oklab.b - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_oklab_roundtrip() {
        let colors = [
            Rgba::RED,
            Rgba::GREEN,
            Rgba::BLUE,
            Rgba::rgb(0.5, 0.3, 0.7),
            Rgba::rgb(1.0, 0.5, 0.0),
        ];

        for color in &colors {
            let oklab = Oklab::from_rgba(color);
            let back = oklab.to_rgba();
            assert!(
                (color.r - back.r).abs() < 0.01,
                "Red mismatch for {:?}",
                color
            );
            assert!(
                (color.g - back.g).abs() < 0.01,
                "Green mismatch for {:?}",
                color
            );
            assert!(
                (color.b - back.b).abs() < 0.01,
                "Blue mismatch for {:?}",
                color
            );
        }
    }

    #[test]
    fn test_oklab_lerp() {
        let black = Rgba::BLACK;
        let white = Rgba::WHITE;
        let mid = interpolate_oklab(&black, &white, 0.5);

        // Mid-gray in Oklab should have L ≈ 0.5
        let oklab_mid = Oklab::from_rgba(&mid);
        assert!((oklab_mid.l - 0.5).abs() < 0.1);
    }

    #[test]
    fn test_oklab_distance() {
        let red = Oklab::from_rgba(&Rgba::RED);
        let blue = Oklab::from_rgba(&Rgba::BLUE);
        let green = Oklab::from_rgba(&Rgba::GREEN);

        let dist_rb = red.distance(&blue);
        let dist_rg = red.distance(&green);

        // Both should be positive
        assert!(dist_rb > 0.0);
        assert!(dist_rg > 0.0);
    }

    #[test]
    fn test_oklch_roundtrip() {
        let colors = [Rgba::RED, Rgba::GREEN, Rgba::BLUE];

        for color in &colors {
            let oklch = Oklch::from_rgba(color);
            let back = oklch.to_rgba();
            assert!(
                (color.r - back.r).abs() < 0.02,
                "Red mismatch for {:?}",
                color
            );
            assert!(
                (color.g - back.g).abs() < 0.02,
                "Green mismatch for {:?}",
                color
            );
            assert!(
                (color.b - back.b).abs() < 0.02,
                "Blue mismatch for {:?}",
                color
            );
        }
    }

    #[test]
    fn test_oklch_complement() {
        let red = Oklch::from_rgba(&Rgba::RED);
        let complement = red.complement();

        // Hue should differ by π
        let dh = (complement.h - red.h).abs();
        assert!((dh - std::f32::consts::PI).abs() < 0.01);
    }

    #[test]
    fn test_oklch_lerp_hue() {
        // Test that hue interpolation takes shortest path
        let oklch1 = Oklch::new(0.5, 0.1, 0.1); // Near 0
        let oklch2 = Oklch::new(0.5, 0.1, 2.0 * std::f32::consts::PI - 0.1); // Near 2π

        let mid = oklch1.lerp(&oklch2, 0.5);

        // Should go through 0 (shortest path), not through π
        assert!(mid.h.abs() < 0.5 || (mid.h - 2.0 * std::f32::consts::PI).abs() < 0.5);
    }

    #[test]
    fn test_chroma() {
        let gray = Oklab::from_rgba(&Rgba::rgb(0.5, 0.5, 0.5));
        let red = Oklab::from_rgba(&Rgba::RED);

        // Gray should have near-zero chroma
        assert!(gray.chroma() < 0.01);
        // Red should have positive chroma
        assert!(red.chroma() > 0.1);
    }

    #[test]
    fn test_interpolate_oklch() {
        let red = Rgba::RED;
        let blue = Rgba::BLUE;
        let mid = interpolate_oklch(&red, &blue, 0.5);

        // Mid should be a valid color
        assert!(mid.r >= 0.0 && mid.r <= 1.0);
        assert!(mid.g >= 0.0 && mid.g <= 1.0);
        assert!(mid.b >= 0.0 && mid.b <= 1.0);
    }

    #[test]
    fn test_alpha_preservation() {
        let semi_transparent = Rgba::new(1.0, 0.0, 0.0, 0.5);
        let oklab = Oklab::from_rgba(&semi_transparent);
        assert!((oklab.alpha - 0.5).abs() < 0.01);

        let back = oklab.to_rgba();
        assert!((back.a - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_alpha_interpolation() {
        let opaque = Rgba::new(1.0, 0.0, 0.0, 1.0);
        let transparent = Rgba::new(0.0, 0.0, 1.0, 0.0);
        let mid = interpolate_oklab(&opaque, &transparent, 0.5);

        // Alpha should be interpolated
        assert!((mid.a - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_hue_normalization() {
        let oklch = Oklch::new(0.5, 0.1, 0.0);
        let rotated = oklch.with_hue(3.0 * std::f32::consts::PI); // 3π should normalize to π
        assert!(rotated.h >= 0.0);
        assert!(rotated.h < 2.0 * std::f32::consts::PI);
    }
}

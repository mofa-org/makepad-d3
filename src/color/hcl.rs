//! HCL (Hue-Chroma-Lightness) color space
//!
//! HCL is the polar form of Lab color space, making it intuitive to work with
//! for hue-based operations. It's also known as LCh (Lightness-Chroma-hue).
//!
//! # Components
//! - H: Hue angle (0-360 degrees)
//! - C: Chroma (colorfulness, 0-~134)
//! - L: Lightness (0-100)
//!
//! HCL is preferred for color interpolation when you want to maintain
//! perceptual uniformity while moving through hue space.

use super::lab::Lab;
use super::types::Rgba;
use std::f32::consts::PI;

/// HCL (Hue-Chroma-Lightness) color representation
///
/// This is the cylindrical (polar) form of Lab color space.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Hcl {
    /// Hue angle in degrees (0-360)
    pub h: f32,
    /// Chroma (colorfulness, 0 to ~134)
    pub c: f32,
    /// Lightness (0-100)
    pub l: f32,
    /// Alpha (0-1)
    pub alpha: f32,
}

impl Hcl {
    /// Create a new HCL color
    pub fn new(h: f32, c: f32, l: f32) -> Self {
        Self {
            h: normalize_hue(h),
            c: c.max(0.0),
            l: l.clamp(0.0, 100.0),
            alpha: 1.0,
        }
    }

    /// Create a new HCL color with alpha
    pub fn with_alpha(h: f32, c: f32, l: f32, alpha: f32) -> Self {
        Self {
            h: normalize_hue(h),
            c: c.max(0.0),
            l: l.clamp(0.0, 100.0),
            alpha: alpha.clamp(0.0, 1.0),
        }
    }

    /// Convert from Lab to HCL
    pub fn from_lab(lab: &Lab) -> Self {
        let c = (lab.a * lab.a + lab.b * lab.b).sqrt();
        let h = if c < 0.0001 {
            0.0 // Achromatic
        } else {
            let h = lab.b.atan2(lab.a) * 180.0 / PI;
            normalize_hue(h)
        };

        Self {
            h,
            c,
            l: lab.l,
            alpha: lab.alpha,
        }
    }

    /// Convert HCL to Lab
    pub fn to_lab(&self) -> Lab {
        let h_rad = self.h * PI / 180.0;
        Lab {
            l: self.l,
            a: self.c * h_rad.cos(),
            b: self.c * h_rad.sin(),
            alpha: self.alpha,
        }
    }

    /// Convert from RGBA to HCL
    pub fn from_rgba(rgba: &Rgba) -> Self {
        let lab = Lab::from_rgba(rgba);
        Self::from_lab(&lab)
    }

    /// Convert HCL to RGBA
    pub fn to_rgba(&self) -> Rgba {
        self.to_lab().to_rgba()
    }

    /// Linear interpolation in HCL space
    ///
    /// Uses shortest path around the hue circle by default.
    pub fn lerp(&self, other: &Hcl, t: f32) -> Hcl {
        self.lerp_with_hue_mode(other, t, HueInterpolation::Shorter)
    }

    /// Interpolation with specified hue interpolation mode
    pub fn lerp_with_hue_mode(&self, other: &Hcl, t: f32, mode: HueInterpolation) -> Hcl {
        let t = t.clamp(0.0, 1.0);

        // Handle achromatic colors (near-zero chroma)
        let h = if self.c < 0.0001 {
            other.h
        } else if other.c < 0.0001 {
            self.h
        } else {
            interpolate_hue(self.h, other.h, t, mode)
        };

        Hcl {
            h,
            c: self.c + (other.c - self.c) * t,
            l: self.l + (other.l - self.l) * t,
            alpha: self.alpha + (other.alpha - self.alpha) * t,
        }
    }

    /// Rotate hue by given degrees
    pub fn rotate(&self, degrees: f32) -> Hcl {
        Hcl::with_alpha(self.h + degrees, self.c, self.l, self.alpha)
    }

    /// Set chroma (colorfulness)
    pub fn with_chroma(&self, c: f32) -> Hcl {
        Hcl::with_alpha(self.h, c, self.l, self.alpha)
    }

    /// Set lightness
    pub fn with_lightness(&self, l: f32) -> Hcl {
        Hcl::with_alpha(self.h, self.c, l, self.alpha)
    }

    /// Darken the color
    pub fn darken(&self, amount: f32) -> Hcl {
        Hcl::with_alpha(
            self.h,
            self.c,
            (self.l - self.l * amount).max(0.0),
            self.alpha,
        )
    }

    /// Lighten the color
    pub fn lighten(&self, amount: f32) -> Hcl {
        Hcl::with_alpha(
            self.h,
            self.c,
            (self.l + (100.0 - self.l) * amount).min(100.0),
            self.alpha,
        )
    }

    /// Desaturate (reduce chroma)
    pub fn desaturate(&self, amount: f32) -> Hcl {
        Hcl::with_alpha(
            self.h,
            (self.c * (1.0 - amount)).max(0.0),
            self.l,
            self.alpha,
        )
    }

    /// Saturate (increase chroma)
    pub fn saturate(&self, amount: f32) -> Hcl {
        Hcl::with_alpha(self.h, self.c * (1.0 + amount), self.l, self.alpha)
    }

    /// Get complementary color (opposite hue)
    pub fn complement(&self) -> Hcl {
        self.rotate(180.0)
    }

    /// Get triadic colors (120 degrees apart)
    pub fn triadic(&self) -> [Hcl; 3] {
        [*self, self.rotate(120.0), self.rotate(240.0)]
    }

    /// Get analogous colors (30 degrees apart)
    pub fn analogous(&self) -> [Hcl; 3] {
        [self.rotate(-30.0), *self, self.rotate(30.0)]
    }

    /// Get split complementary colors
    pub fn split_complementary(&self) -> [Hcl; 3] {
        [*self, self.rotate(150.0), self.rotate(210.0)]
    }
}

/// How to interpolate hue values
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HueInterpolation {
    /// Take the shorter path around the hue circle
    #[default]
    Shorter,
    /// Take the longer path around the hue circle
    Longer,
    /// Always increase hue (counterclockwise)
    Increasing,
    /// Always decrease hue (clockwise)
    Decreasing,
    /// Linear interpolation (may jump across 0/360)
    Raw,
}

/// Normalize hue to 0-360 range
fn normalize_hue(h: f32) -> f32 {
    let h = h % 360.0;
    if h < 0.0 {
        h + 360.0
    } else {
        h
    }
}

/// Interpolate between two hue values
fn interpolate_hue(h1: f32, h2: f32, t: f32, mode: HueInterpolation) -> f32 {
    let h1 = normalize_hue(h1);
    let h2 = normalize_hue(h2);

    match mode {
        HueInterpolation::Raw => normalize_hue(h1 + (h2 - h1) * t),
        HueInterpolation::Shorter => {
            let diff = h2 - h1;
            let adjusted_diff = if diff > 180.0 {
                diff - 360.0
            } else if diff < -180.0 {
                diff + 360.0
            } else {
                diff
            };
            normalize_hue(h1 + adjusted_diff * t)
        }
        HueInterpolation::Longer => {
            let diff = h2 - h1;
            let adjusted_diff = if diff > 0.0 && diff < 180.0 {
                diff - 360.0
            } else if diff < 0.0 && diff > -180.0 {
                diff + 360.0
            } else {
                diff
            };
            normalize_hue(h1 + adjusted_diff * t)
        }
        HueInterpolation::Increasing => {
            let diff = if h2 >= h1 { h2 - h1 } else { h2 + 360.0 - h1 };
            normalize_hue(h1 + diff * t)
        }
        HueInterpolation::Decreasing => {
            let diff = if h1 >= h2 { h1 - h2 } else { h1 + 360.0 - h2 };
            normalize_hue(h1 - diff * t)
        }
    }
}

impl From<Rgba> for Hcl {
    fn from(rgba: Rgba) -> Self {
        Hcl::from_rgba(&rgba)
    }
}

impl From<Hcl> for Rgba {
    fn from(hcl: Hcl) -> Self {
        hcl.to_rgba()
    }
}

impl From<Lab> for Hcl {
    fn from(lab: Lab) -> Self {
        Hcl::from_lab(&lab)
    }
}

impl From<Hcl> for Lab {
    fn from(hcl: Hcl) -> Self {
        hcl.to_lab()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hcl_from_rgba() {
        let red = Rgba::RED;
        let hcl = Hcl::from_rgba(&red);

        // Red should have hue near 0 or 360
        assert!(hcl.h < 50.0 || hcl.h > 310.0);
        // Should have positive chroma
        assert!(hcl.c > 0.0);
        // Should have medium-high lightness
        assert!(hcl.l > 40.0 && hcl.l < 70.0);
    }

    #[test]
    fn test_roundtrip() {
        let colors = [Rgba::RED, Rgba::GREEN, Rgba::BLUE, Rgba::from_hex(0x4285F4)];

        for original in colors {
            let hcl = Hcl::from_rgba(&original);
            let back = hcl.to_rgba();

            assert!(
                (original.r - back.r).abs() < 0.03,
                "R mismatch for {:?}",
                original
            );
            assert!(
                (original.g - back.g).abs() < 0.03,
                "G mismatch for {:?}",
                original
            );
            assert!(
                (original.b - back.b).abs() < 0.03,
                "B mismatch for {:?}",
                original
            );
        }
    }

    #[test]
    fn test_achromatic() {
        let gray = Rgba::rgb(0.5, 0.5, 0.5);
        let hcl = Hcl::from_rgba(&gray);

        // Gray should have near-zero chroma
        assert!(hcl.c < 1.0);
    }

    #[test]
    fn test_rotate() {
        let hcl = Hcl::new(30.0, 50.0, 60.0);
        let rotated = hcl.rotate(120.0);

        assert!((rotated.h - 150.0).abs() < 0.1);
        assert_eq!(rotated.c, hcl.c);
        assert_eq!(rotated.l, hcl.l);
    }

    #[test]
    fn test_complement() {
        let hcl = Hcl::new(60.0, 50.0, 60.0);
        let comp = hcl.complement();

        assert!((comp.h - 240.0).abs() < 0.1);
    }

    #[test]
    fn test_lerp_same_hue() {
        let light = Hcl::new(180.0, 50.0, 80.0);
        let dark = Hcl::new(180.0, 50.0, 20.0);

        let mid = light.lerp(&dark, 0.5);

        assert!((mid.h - 180.0).abs() < 1.0);
        assert!((mid.l - 50.0).abs() < 1.0);
    }

    #[test]
    fn test_hue_interpolation_shorter() {
        // Test crossing 0/360 boundary
        let h1 = Hcl::new(350.0, 50.0, 50.0);
        let h2 = Hcl::new(10.0, 50.0, 50.0);

        let mid = h1.lerp(&h2, 0.5);

        // Should go through 0/360, not through 180
        assert!(mid.h < 30.0 || mid.h > 330.0);
    }

    #[test]
    fn test_triadic() {
        let hcl = Hcl::new(0.0, 50.0, 50.0);
        let triadic = hcl.triadic();

        assert!((triadic[0].h - 0.0).abs() < 0.1);
        assert!((triadic[1].h - 120.0).abs() < 0.1);
        assert!((triadic[2].h - 240.0).abs() < 0.1);
    }

    #[test]
    fn test_normalize_hue() {
        assert!((normalize_hue(370.0) - 10.0).abs() < 0.001);
        assert!((normalize_hue(-30.0) - 330.0).abs() < 0.001);
        assert!((normalize_hue(180.0) - 180.0).abs() < 0.001);
    }

    #[test]
    fn test_darken_lighten() {
        let hcl = Hcl::new(180.0, 50.0, 50.0);

        let darker = hcl.darken(0.2);
        assert!(darker.l < hcl.l);

        let lighter = hcl.lighten(0.2);
        assert!(lighter.l > hcl.l);
    }

    #[test]
    fn test_saturate_desaturate() {
        let hcl = Hcl::new(180.0, 50.0, 50.0);

        let desaturated = hcl.desaturate(0.5);
        assert!((desaturated.c - 25.0).abs() < 0.1);

        let saturated = hcl.saturate(0.5);
        assert!((saturated.c - 75.0).abs() < 0.1);
    }
}

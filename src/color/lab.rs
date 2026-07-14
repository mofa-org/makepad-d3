//! Lab (CIELAB) color space
//!
//! Lab is a perceptually uniform color space where Euclidean distance
//! corresponds to perceived color difference. This makes it ideal for
//! color interpolation and creating perceptually uniform gradients.
//!
//! # Components
//! - L*: Lightness (0 = black, 100 = white)
//! - a*: Green-Red axis (negative = green, positive = red)
//! - b*: Blue-Yellow axis (negative = blue, positive = yellow)

use super::types::Rgba;

/// D65 standard illuminant reference white
const REF_X: f32 = 95.047;
const REF_Y: f32 = 100.000;
const REF_Z: f32 = 108.883;

/// Lab epsilon for linear/non-linear transition
const EPSILON: f32 = 0.008856;
/// Lab kappa constant
const KAPPA: f32 = 903.3;

/// Lab (CIELAB) color representation
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Lab {
    /// Lightness (0-100)
    pub l: f32,
    /// Green-Red axis (typically -128 to 128)
    pub a: f32,
    /// Blue-Yellow axis (typically -128 to 128)
    pub b: f32,
    /// Alpha (0-1)
    pub alpha: f32,
}

impl Lab {
    /// Create a new Lab color
    pub fn new(l: f32, a: f32, b: f32) -> Self {
        Self {
            l,
            a,
            b,
            alpha: 1.0,
        }
    }

    /// Create a new Lab color with alpha
    pub fn with_alpha(l: f32, a: f32, b: f32, alpha: f32) -> Self {
        Self {
            l,
            a,
            b,
            alpha: alpha.clamp(0.0, 1.0),
        }
    }

    /// Convert from RGBA to Lab
    pub fn from_rgba(rgba: &Rgba) -> Self {
        // First convert RGB to XYZ
        let xyz = rgb_to_xyz(rgba.r, rgba.g, rgba.b);

        // Then convert XYZ to Lab
        xyz_to_lab(xyz.0, xyz.1, xyz.2, rgba.a)
    }

    /// Convert Lab to RGBA
    pub fn to_rgba(&self) -> Rgba {
        // First convert Lab to XYZ
        let xyz = lab_to_xyz(self.l, self.a, self.b);

        // Then convert XYZ to RGB
        let (r, g, b) = xyz_to_rgb(xyz.0, xyz.1, xyz.2);

        Rgba::new(r, g, b, self.alpha)
    }

    /// Linear interpolation in Lab space
    pub fn lerp(&self, other: &Lab, t: f32) -> Lab {
        let t = t.clamp(0.0, 1.0);
        Lab {
            l: self.l + (other.l - self.l) * t,
            a: self.a + (other.a - self.a) * t,
            b: self.b + (other.b - self.b) * t,
            alpha: self.alpha + (other.alpha - self.alpha) * t,
        }
    }

    /// Calculate perceptual distance to another Lab color (Delta E)
    pub fn distance(&self, other: &Lab) -> f32 {
        let dl = self.l - other.l;
        let da = self.a - other.a;
        let db = self.b - other.b;
        (dl * dl + da * da + db * db).sqrt()
    }

    /// Darken the color by a factor (0-1)
    pub fn darken(&self, amount: f32) -> Lab {
        Lab {
            l: (self.l - self.l * amount).max(0.0),
            a: self.a,
            b: self.b,
            alpha: self.alpha,
        }
    }

    /// Lighten the color by a factor (0-1)
    pub fn lighten(&self, amount: f32) -> Lab {
        Lab {
            l: (self.l + (100.0 - self.l) * amount).min(100.0),
            a: self.a,
            b: self.b,
            alpha: self.alpha,
        }
    }

    /// Get the chroma (colorfulness)
    pub fn chroma(&self) -> f32 {
        (self.a * self.a + self.b * self.b).sqrt()
    }

    /// Get the hue angle in degrees (0-360)
    pub fn hue(&self) -> f32 {
        let h = self.b.atan2(self.a).to_degrees();
        if h < 0.0 {
            h + 360.0
        } else {
            h
        }
    }
}

/// Convert sRGB to XYZ
fn rgb_to_xyz(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    // Convert sRGB to linear RGB
    let r = srgb_to_linear(r);
    let g = srgb_to_linear(g);
    let b = srgb_to_linear(b);

    // Convert linear RGB to XYZ using sRGB matrix
    let x = r * 41.24564 + g * 35.75761 + b * 18.04375;
    let y = r * 21.26729 + g * 71.51522 + b * 7.21750;
    let z = r * 1.93339 + g * 11.91920 + b * 95.03041;

    (x, y, z)
}

/// Convert XYZ to sRGB
fn xyz_to_rgb(x: f32, y: f32, z: f32) -> (f32, f32, f32) {
    // Convert XYZ to linear RGB
    let r = x * 0.032406255 + y * -0.015372232 + z * -0.004986429;
    let g = x * -0.009689307 + y * 0.018760108 + z * 0.000415604;
    let b = x * 0.000557101 + y * -0.002040211 + z * 0.010570489;

    // Convert linear RGB to sRGB
    (
        linear_to_srgb(r).clamp(0.0, 1.0),
        linear_to_srgb(g).clamp(0.0, 1.0),
        linear_to_srgb(b).clamp(0.0, 1.0),
    )
}

/// Convert XYZ to Lab
fn xyz_to_lab(x: f32, y: f32, z: f32, alpha: f32) -> Lab {
    let x = x / REF_X;
    let y = y / REF_Y;
    let z = z / REF_Z;

    let x = lab_f(x);
    let y = lab_f(y);
    let z = lab_f(z);

    Lab {
        l: 116.0 * y - 16.0,
        a: 500.0 * (x - y),
        b: 200.0 * (y - z),
        alpha,
    }
}

/// Convert Lab to XYZ
fn lab_to_xyz(l: f32, a: f32, b: f32) -> (f32, f32, f32) {
    let y = (l + 16.0) / 116.0;
    let x = a / 500.0 + y;
    let z = y - b / 200.0;

    let x = lab_f_inv(x) * REF_X;
    let y = lab_f_inv(y) * REF_Y;
    let z = lab_f_inv(z) * REF_Z;

    (x, y, z)
}

/// Lab f function (cube root with linear segment)
fn lab_f(t: f32) -> f32 {
    if t > EPSILON {
        t.powf(1.0 / 3.0)
    } else {
        (KAPPA * t + 16.0) / 116.0
    }
}

/// Lab f inverse function
fn lab_f_inv(t: f32) -> f32 {
    let t3 = t * t * t;
    if t3 > EPSILON {
        t3
    } else {
        (116.0 * t - 16.0) / KAPPA
    }
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
}

impl From<Rgba> for Lab {
    fn from(rgba: Rgba) -> Self {
        Lab::from_rgba(&rgba)
    }
}

impl From<Lab> for Rgba {
    fn from(lab: Lab) -> Self {
        lab.to_rgba()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_white() {
        let white = Rgba::WHITE;
        let lab = Lab::from_rgba(&white);

        // White should have L* close to 100, a* and b* close to 0
        assert!((lab.l - 100.0).abs() < 1.0);
        assert!(lab.a.abs() < 1.0);
        assert!(lab.b.abs() < 1.0);
    }

    #[test]
    fn test_black() {
        let black = Rgba::BLACK;
        let lab = Lab::from_rgba(&black);

        // Black should have L* close to 0
        assert!(lab.l.abs() < 1.0);
    }

    #[test]
    fn test_red() {
        let red = Rgba::RED;
        let lab = Lab::from_rgba(&red);

        // Red should have positive a* (red-green axis)
        assert!(lab.a > 0.0);
    }

    #[test]
    fn test_roundtrip() {
        let colors = [
            Rgba::RED,
            Rgba::GREEN,
            Rgba::BLUE,
            Rgba::from_hex(0x4285F4),
            Rgba::from_hex(0xFF6B6B),
        ];

        for original in colors {
            let lab = Lab::from_rgba(&original);
            let back = lab.to_rgba();

            assert!((original.r - back.r).abs() < 0.02, "R mismatch");
            assert!((original.g - back.g).abs() < 0.02, "G mismatch");
            assert!((original.b - back.b).abs() < 0.02, "B mismatch");
        }
    }

    #[test]
    fn test_lerp() {
        let black = Lab::from_rgba(&Rgba::BLACK);
        let white = Lab::from_rgba(&Rgba::WHITE);

        let mid = black.lerp(&white, 0.5);

        // Mid-gray should have L* around 50
        assert!((mid.l - 50.0).abs() < 5.0);
    }

    #[test]
    fn test_distance() {
        let red = Lab::from_rgba(&Rgba::RED);
        let green = Lab::from_rgba(&Rgba::GREEN);
        let red2 = Lab::from_rgba(&Rgba::RED);

        // Same color should have zero distance
        assert!(red.distance(&red2) < 0.1);

        // Different colors should have positive distance
        assert!(red.distance(&green) > 0.0);
    }

    #[test]
    fn test_darken_lighten() {
        let gray = Lab::new(50.0, 0.0, 0.0);

        let darker = gray.darken(0.2);
        assert!(darker.l < gray.l);

        let lighter = gray.lighten(0.2);
        assert!(lighter.l > gray.l);
    }

    #[test]
    fn test_chroma_and_hue() {
        let lab = Lab::new(50.0, 30.0, 40.0);

        let chroma = lab.chroma();
        assert!((chroma - 50.0).abs() < 0.1); // sqrt(30^2 + 40^2) = 50

        let hue = lab.hue();
        assert!(hue >= 0.0 && hue < 360.0);
    }
}

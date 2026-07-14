//! Color blending and mixing operations
//!
//! This module provides various color blending modes commonly used in
//! graphics software, as well as utilities for mixing multiple colors.
//!
//! # Blend Modes
//!
//! - Normal: Simple alpha compositing
//! - Multiply: Darkening blend
//! - Screen: Lightening blend
//! - Overlay: Combination of multiply and screen
//! - Soft Light: Gentle lighting effect
//! - Hard Light: Strong lighting effect
//! - Difference: Absolute difference
//! - Dodge/Burn: Extreme lightening/darkening

use super::lab::Lab;
use super::types::Rgba;

/// Blend mode for combining colors
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BlendMode {
    /// Normal alpha compositing
    #[default]
    Normal,
    /// Darkening blend (multiply)
    Multiply,
    /// Lightening blend (screen)
    Screen,
    /// Overlay (combination of multiply and screen)
    Overlay,
    /// Soft light effect
    SoftLight,
    /// Hard light effect
    HardLight,
    /// Color dodge (extreme lighten)
    ColorDodge,
    /// Color burn (extreme darken)
    ColorBurn,
    /// Absolute difference
    Difference,
    /// Exclusion (softer difference)
    Exclusion,
    /// Darken (min)
    Darken,
    /// Lighten (max)
    Lighten,
}

impl BlendMode {
    /// Blend two color components
    fn blend_component(&self, base: f32, blend: f32) -> f32 {
        match self {
            BlendMode::Normal => blend,
            BlendMode::Multiply => base * blend,
            BlendMode::Screen => 1.0 - (1.0 - base) * (1.0 - blend),
            BlendMode::Overlay => {
                if base < 0.5 {
                    2.0 * base * blend
                } else {
                    1.0 - 2.0 * (1.0 - base) * (1.0 - blend)
                }
            }
            BlendMode::SoftLight => {
                if blend < 0.5 {
                    base - (1.0 - 2.0 * blend) * base * (1.0 - base)
                } else {
                    let d = if base <= 0.25 {
                        ((16.0 * base - 12.0) * base + 4.0) * base
                    } else {
                        base.sqrt()
                    };
                    base + (2.0 * blend - 1.0) * (d - base)
                }
            }
            BlendMode::HardLight => {
                if blend < 0.5 {
                    2.0 * base * blend
                } else {
                    1.0 - 2.0 * (1.0 - base) * (1.0 - blend)
                }
            }
            BlendMode::ColorDodge => {
                if blend >= 1.0 {
                    1.0
                } else {
                    (base / (1.0 - blend)).min(1.0)
                }
            }
            BlendMode::ColorBurn => {
                if blend <= 0.0 {
                    0.0
                } else {
                    (1.0 - (1.0 - base) / blend).max(0.0)
                }
            }
            BlendMode::Difference => (base - blend).abs(),
            BlendMode::Exclusion => base + blend - 2.0 * base * blend,
            BlendMode::Darken => base.min(blend),
            BlendMode::Lighten => base.max(blend),
        }
    }
}

/// Blend two colors using the specified blend mode
pub fn blend(base: &Rgba, blend_color: &Rgba, mode: BlendMode) -> Rgba {
    let r = mode.blend_component(base.r, blend_color.r);
    let g = mode.blend_component(base.g, blend_color.g);
    let b = mode.blend_component(base.b, blend_color.b);

    // Alpha compositing
    let a = base.a + blend_color.a * (1.0 - base.a);

    Rgba::new(
        r.clamp(0.0, 1.0),
        g.clamp(0.0, 1.0),
        b.clamp(0.0, 1.0),
        a.clamp(0.0, 1.0),
    )
}

/// Blend with opacity (t=0 returns base, t=1 returns full blend)
pub fn blend_with_opacity(base: &Rgba, blend_color: &Rgba, mode: BlendMode, opacity: f32) -> Rgba {
    let blended = blend(base, blend_color, mode);
    base.lerp(&blended, opacity)
}

/// Alpha composite two colors (Porter-Duff "over" operation)
pub fn composite_over(base: &Rgba, overlay: &Rgba) -> Rgba {
    let a_out = overlay.a + base.a * (1.0 - overlay.a);

    if a_out < f32::EPSILON {
        return Rgba::TRANSPARENT;
    }

    let r = (overlay.r * overlay.a + base.r * base.a * (1.0 - overlay.a)) / a_out;
    let g = (overlay.g * overlay.a + base.g * base.a * (1.0 - overlay.a)) / a_out;
    let b = (overlay.b * overlay.a + base.b * base.a * (1.0 - overlay.a)) / a_out;

    Rgba::new(r, g, b, a_out)
}

/// Mix multiple colors with equal weights
pub fn mix(colors: &[Rgba]) -> Rgba {
    if colors.is_empty() {
        return Rgba::BLACK;
    }

    let n = colors.len() as f32;
    let mut r = 0.0;
    let mut g = 0.0;
    let mut b = 0.0;
    let mut a = 0.0;

    for c in colors {
        r += c.r;
        g += c.g;
        b += c.b;
        a += c.a;
    }

    Rgba::new(r / n, g / n, b / n, a / n)
}

/// Mix colors with specified weights
pub fn mix_weighted(colors: &[Rgba], weights: &[f32]) -> Rgba {
    if colors.is_empty() || weights.is_empty() {
        return Rgba::BLACK;
    }

    let total_weight: f32 = weights.iter().sum();
    if total_weight < f32::EPSILON {
        return Rgba::BLACK;
    }

    let mut r = 0.0;
    let mut g = 0.0;
    let mut b = 0.0;
    let mut a = 0.0;

    for (c, &w) in colors.iter().zip(weights.iter()) {
        r += c.r * w;
        g += c.g * w;
        b += c.b * w;
        a += c.a * w;
    }

    Rgba::new(
        r / total_weight,
        g / total_weight,
        b / total_weight,
        a / total_weight,
    )
}

/// Mix colors in Lab space (perceptually uniform)
pub fn mix_lab(colors: &[Rgba]) -> Rgba {
    if colors.is_empty() {
        return Rgba::BLACK;
    }

    let labs: Vec<Lab> = colors.iter().map(Lab::from_rgba).collect();

    let n = labs.len() as f32;
    let mut l = 0.0;
    let mut a = 0.0;
    let mut b = 0.0;
    let mut alpha = 0.0;

    for lab in &labs {
        l += lab.l;
        a += lab.a;
        b += lab.b;
        alpha += lab.alpha;
    }

    Lab::with_alpha(l / n, a / n, b / n, alpha / n).to_rgba()
}

/// Tint a color (mix with white)
pub fn tint(color: &Rgba, amount: f32) -> Rgba {
    color.lerp(&Rgba::WHITE, amount.clamp(0.0, 1.0))
}

/// Shade a color (mix with black)
pub fn shade(color: &Rgba, amount: f32) -> Rgba {
    color.lerp(&Rgba::BLACK, amount.clamp(0.0, 1.0))
}

/// Tone a color (mix with gray)
pub fn tone(color: &Rgba, amount: f32) -> Rgba {
    let gray = Rgba::rgb(0.5, 0.5, 0.5);
    color.lerp(&gray, amount.clamp(0.0, 1.0))
}

/// Adjust color brightness
pub fn brightness(color: &Rgba, amount: f32) -> Rgba {
    let lab = Lab::from_rgba(color);
    let new_l = (lab.l + amount * 100.0).clamp(0.0, 100.0);
    Lab::with_alpha(new_l, lab.a, lab.b, lab.alpha).to_rgba()
}

/// Adjust color contrast
pub fn contrast(color: &Rgba, amount: f32) -> Rgba {
    let factor = (1.0 + amount).max(0.0);

    Rgba::new(
        ((color.r - 0.5) * factor + 0.5).clamp(0.0, 1.0),
        ((color.g - 0.5) * factor + 0.5).clamp(0.0, 1.0),
        ((color.b - 0.5) * factor + 0.5).clamp(0.0, 1.0),
        color.a,
    )
}

/// Invert a color
pub fn invert(color: &Rgba) -> Rgba {
    Rgba::new(1.0 - color.r, 1.0 - color.g, 1.0 - color.b, color.a)
}

/// Grayscale conversion using luminance weights
pub fn grayscale(color: &Rgba) -> Rgba {
    // ITU-R BT.709 luminance coefficients
    let l = 0.2126 * color.r + 0.7152 * color.g + 0.0722 * color.b;
    Rgba::new(l, l, l, color.a)
}

/// Sepia tone effect
pub fn sepia(color: &Rgba) -> Rgba {
    let r = (color.r * 0.393 + color.g * 0.769 + color.b * 0.189).min(1.0);
    let g = (color.r * 0.349 + color.g * 0.686 + color.b * 0.168).min(1.0);
    let b = (color.r * 0.272 + color.g * 0.534 + color.b * 0.131).min(1.0);
    Rgba::new(r, g, b, color.a)
}

/// Calculate color luminance (perceived brightness)
pub fn luminance(color: &Rgba) -> f32 {
    0.2126 * color.r + 0.7152 * color.g + 0.0722 * color.b
}

/// Calculate contrast ratio between two colors (WCAG)
pub fn contrast_ratio(color1: &Rgba, color2: &Rgba) -> f32 {
    let l1 = luminance(color1) + 0.05;
    let l2 = luminance(color2) + 0.05;

    if l1 > l2 {
        l1 / l2
    } else {
        l2 / l1
    }
}

/// Check if contrast ratio meets WCAG AA standard (4.5:1 for normal text)
pub fn meets_wcag_aa(foreground: &Rgba, background: &Rgba) -> bool {
    contrast_ratio(foreground, background) >= 4.5
}

/// Check if contrast ratio meets WCAG AAA standard (7:1 for normal text)
pub fn meets_wcag_aaa(foreground: &Rgba, background: &Rgba) -> bool {
    contrast_ratio(foreground, background) >= 7.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blend_multiply() {
        let white = Rgba::WHITE;
        let gray = Rgba::rgb(0.5, 0.5, 0.5);

        let result = blend(&white, &gray, BlendMode::Multiply);

        // Multiply with white should return the blend color
        assert!((result.r - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_blend_screen() {
        let black = Rgba::BLACK;
        let gray = Rgba::rgb(0.5, 0.5, 0.5);

        let result = blend(&black, &gray, BlendMode::Screen);

        // Screen with black should return the blend color
        assert!((result.r - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_blend_overlay() {
        let gray = Rgba::rgb(0.5, 0.5, 0.5);
        let result = blend(&gray, &gray, BlendMode::Overlay);

        // Overlay of gray on gray should be close to gray
        assert!((result.r - 0.5).abs() < 0.1);
    }

    #[test]
    fn test_composite_over() {
        let base = Rgba::RED;
        let overlay = Rgba::BLUE.with_alpha(0.5);

        let result = composite_over(&base, &overlay);

        // Should be a blend of red and blue
        assert!(result.r > 0.0 && result.b > 0.0);
    }

    #[test]
    fn test_mix() {
        let colors = vec![Rgba::RED, Rgba::GREEN, Rgba::BLUE];
        let mixed = mix(&colors);

        // Average should be grayish
        assert!((mixed.r - 0.333).abs() < 0.1);
        assert!((mixed.g - 0.333).abs() < 0.1);
        assert!((mixed.b - 0.333).abs() < 0.1);
    }

    #[test]
    fn test_mix_weighted() {
        let colors = vec![Rgba::RED, Rgba::BLUE];
        let weights = vec![3.0, 1.0];

        let mixed = mix_weighted(&colors, &weights);

        // Should be more red than blue
        assert!(mixed.r > mixed.b);
    }

    #[test]
    fn test_tint_shade() {
        let red = Rgba::RED;

        let tinted = tint(&red, 0.5);
        let shaded = shade(&red, 0.5);

        // Tinted should be lighter
        assert!(luminance(&tinted) > luminance(&red));
        // Shaded should be darker
        assert!(luminance(&shaded) < luminance(&red));
    }

    #[test]
    fn test_invert() {
        let black = Rgba::BLACK;
        let white = Rgba::WHITE;

        let inverted_black = invert(&black);
        let inverted_white = invert(&white);

        assert_eq!(inverted_black.to_hex(), white.to_hex());
        assert_eq!(inverted_white.to_hex(), black.to_hex());
    }

    #[test]
    fn test_grayscale() {
        let red = Rgba::RED;
        let gray = grayscale(&red);

        // All channels should be equal
        assert!((gray.r - gray.g).abs() < 0.001);
        assert!((gray.g - gray.b).abs() < 0.001);
    }

    #[test]
    fn test_contrast_ratio() {
        let black = Rgba::BLACK;
        let white = Rgba::WHITE;

        let ratio = contrast_ratio(&black, &white);

        // Black/white should have maximum contrast (21:1)
        assert!(ratio > 20.0);
    }

    #[test]
    fn test_wcag() {
        let black = Rgba::BLACK;
        let white = Rgba::WHITE;
        let gray = Rgba::rgb(0.5, 0.5, 0.5);

        // Black on white should pass all WCAG
        assert!(meets_wcag_aa(&black, &white));
        assert!(meets_wcag_aaa(&black, &white));

        // Gray on white might not pass
        let gray_on_white_ratio = contrast_ratio(&gray, &white);
        assert!(gray_on_white_ratio < 4.5);
    }

    #[test]
    fn test_brightness() {
        let gray = Rgba::rgb(0.5, 0.5, 0.5);

        let brighter = brightness(&gray, 0.2);
        let darker = brightness(&gray, -0.2);

        assert!(luminance(&brighter) > luminance(&gray));
        assert!(luminance(&darker) < luminance(&gray));
    }

    #[test]
    fn test_contrast_adjustment() {
        let gray = Rgba::rgb(0.5, 0.5, 0.5);
        let light_gray = Rgba::rgb(0.6, 0.6, 0.6);

        let high_contrast = contrast(&light_gray, 1.0);
        let low_contrast = contrast(&light_gray, -0.5);

        // Higher contrast should push away from middle gray
        assert!(high_contrast.r > light_gray.r);
        // Lower contrast should pull toward middle gray
        assert!((low_contrast.r - 0.5).abs() < (light_gray.r - 0.5).abs());
    }
}

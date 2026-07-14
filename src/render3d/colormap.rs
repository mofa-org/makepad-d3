//! Colormap definitions for 3D visualization shaders
//!
//! This module provides colormap implementations that can be used both
//! in Rust code and as shader functions.

use super::types::Vec3;

/// Colormap types available for visualization
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Colormap {
    /// Viridis - perceptually uniform, colorblind-friendly (default)
    #[default]
    Viridis,
    /// Plasma - perceptually uniform, high contrast
    Plasma,
    /// Inferno - perceptually uniform, dark to bright
    Inferno,
    /// Magma - perceptually uniform, dark purple to yellow
    Magma,
    /// Cool-Warm - diverging, blue to red through white
    CoolWarm,
    /// Turbo - rainbow-like but more uniform
    Turbo,
    /// Grayscale - simple black to white
    Grayscale,
}

impl Colormap {
    /// Get the colormap type as a shader-compatible float
    pub fn to_shader_value(&self) -> f32 {
        match self {
            Colormap::Viridis => 0.0,
            Colormap::Plasma => 1.0,
            Colormap::Inferno => 2.0,
            Colormap::Magma => 3.0,
            Colormap::CoolWarm => 4.0,
            Colormap::Turbo => 5.0,
            Colormap::Grayscale => 6.0,
        }
    }

    /// Sample the colormap at a normalized value t in [0, 1]
    pub fn sample(&self, t: f32) -> Vec3 {
        let t = t.clamp(0.0, 1.0);

        match self {
            Colormap::Viridis => viridis(t),
            Colormap::Plasma => plasma(t),
            Colormap::Inferno => inferno(t),
            Colormap::Magma => magma(t),
            Colormap::CoolWarm => cool_warm(t),
            Colormap::Turbo => turbo(t),
            Colormap::Grayscale => Vec3::splat(t),
        }
    }

    /// Get the colormap as RGBA with full opacity
    pub fn sample_rgba(&self, t: f32) -> [f32; 4] {
        let rgb = self.sample(t);
        [rgb.x, rgb.y, rgb.z, 1.0]
    }
}

/// Viridis colormap - perceptually uniform, colorblind-friendly
fn viridis(t: f32) -> Vec3 {
    // 7-point interpolation
    let colors = [
        Vec3::new(0.267, 0.004, 0.329),
        Vec3::new(0.282, 0.140, 0.458),
        Vec3::new(0.254, 0.265, 0.530),
        Vec3::new(0.163, 0.471, 0.558),
        Vec3::new(0.134, 0.658, 0.517),
        Vec3::new(0.477, 0.821, 0.318),
        Vec3::new(0.993, 0.906, 0.144),
    ];
    interpolate_colormap(&colors, t)
}

/// Plasma colormap - high contrast purple to yellow
fn plasma(t: f32) -> Vec3 {
    let colors = [
        Vec3::new(0.050, 0.030, 0.528),
        Vec3::new(0.295, 0.012, 0.615),
        Vec3::new(0.494, 0.012, 0.658),
        Vec3::new(0.665, 0.138, 0.568),
        Vec3::new(0.798, 0.280, 0.470),
        Vec3::new(0.899, 0.434, 0.358),
        Vec3::new(0.973, 0.580, 0.254),
        Vec3::new(0.940, 0.975, 0.131),
    ];
    interpolate_colormap(&colors, t)
}

/// Inferno colormap - dark to bright through orange
fn inferno(t: f32) -> Vec3 {
    let colors = [
        Vec3::new(0.001, 0.000, 0.014),
        Vec3::new(0.120, 0.047, 0.290),
        Vec3::new(0.329, 0.059, 0.406),
        Vec3::new(0.533, 0.134, 0.416),
        Vec3::new(0.735, 0.216, 0.330),
        Vec3::new(0.891, 0.348, 0.194),
        Vec3::new(0.976, 0.559, 0.040),
        Vec3::new(0.988, 0.998, 0.645),
    ];
    interpolate_colormap(&colors, t)
}

/// Magma colormap - dark purple to yellow
fn magma(t: f32) -> Vec3 {
    let colors = [
        Vec3::new(0.001, 0.000, 0.014),
        Vec3::new(0.111, 0.066, 0.267),
        Vec3::new(0.280, 0.090, 0.410),
        Vec3::new(0.469, 0.131, 0.507),
        Vec3::new(0.662, 0.186, 0.515),
        Vec3::new(0.857, 0.323, 0.468),
        Vec3::new(0.984, 0.557, 0.399),
        Vec3::new(0.988, 0.991, 0.749),
    ];
    interpolate_colormap(&colors, t)
}

/// Cool-Warm diverging colormap - blue to red through white
fn cool_warm(t: f32) -> Vec3 {
    if t < 0.5 {
        // Blue to white
        let s = t * 2.0;
        Vec3::new(0.230, 0.299, 0.754).lerp(&Vec3::new(0.865, 0.865, 0.865), s)
    } else {
        // White to red
        let s = (t - 0.5) * 2.0;
        Vec3::new(0.865, 0.865, 0.865).lerp(&Vec3::new(0.706, 0.016, 0.150), s)
    }
}

/// Turbo colormap - improved rainbow
fn turbo(t: f32) -> Vec3 {
    // Polynomial approximation of Google's Turbo colormap
    let r = 0.13572138
        + t * (4.61539260
            + t * (-42.66032258 + t * (132.13108234 + t * (-152.94239396 + t * 59.28637943))));
    let g = 0.09140261
        + t * (2.19418839
            + t * (4.84296658 + t * (-14.18503333 + t * (4.27729857 + t * 2.82956604))));
    let b = 0.10667330
        + t * (12.64194608
            + t * (-60.58204836 + t * (110.36276771 + t * (-89.90310912 + t * 27.34824973))));

    Vec3::new(r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0))
}

/// Interpolate through a colormap defined by key colors
fn interpolate_colormap(colors: &[Vec3], t: f32) -> Vec3 {
    let n = colors.len();
    if n == 0 {
        return Vec3::ZERO;
    }
    if n == 1 {
        return colors[0];
    }

    let t = t.clamp(0.0, 1.0);
    let scaled = t * (n - 1) as f32;
    let i = (scaled as usize).min(n - 2);
    let f = scaled - i as f32;

    colors[i].lerp(&colors[i + 1], f)
}

/// Shader code for colormaps (to be included in live_design!)
///
/// This generates the GLSL-like shader code for colormap functions.
pub const COLORMAP_SHADER_CODE: &str = r#"
    // Colormap types:
    // 0 = Viridis, 1 = Plasma, 2 = Inferno, 3 = Magma
    // 4 = CoolWarm, 5 = Turbo, 6 = Grayscale

    fn colormap_viridis(t: f32) -> vec3 {
        let c0 = vec3(0.267, 0.004, 0.329);
        let c1 = vec3(0.282, 0.140, 0.458);
        let c2 = vec3(0.254, 0.265, 0.530);
        let c3 = vec3(0.163, 0.471, 0.558);
        let c4 = vec3(0.134, 0.658, 0.517);
        let c5 = vec3(0.477, 0.821, 0.318);
        let c6 = vec3(0.993, 0.906, 0.144);

        let t6 = t * 6.0;
        let i = floor(t6);
        let f = fract(t6);

        if i < 1.0 { return mix(c0, c1, f); }
        if i < 2.0 { return mix(c1, c2, f); }
        if i < 3.0 { return mix(c2, c3, f); }
        if i < 4.0 { return mix(c3, c4, f); }
        if i < 5.0 { return mix(c4, c5, f); }
        return mix(c5, c6, f);
    }

    fn colormap_plasma(t: f32) -> vec3 {
        let c0 = vec3(0.050, 0.030, 0.528);
        let c1 = vec3(0.494, 0.012, 0.658);
        let c2 = vec3(0.798, 0.280, 0.470);
        let c3 = vec3(0.973, 0.580, 0.254);
        let c4 = vec3(0.940, 0.975, 0.131);

        let t4 = t * 4.0;
        let i = floor(t4);
        let f = fract(t4);

        if i < 1.0 { return mix(c0, c1, f); }
        if i < 2.0 { return mix(c1, c2, f); }
        if i < 3.0 { return mix(c2, c3, f); }
        return mix(c3, c4, f);
    }

    fn colormap_inferno(t: f32) -> vec3 {
        let c0 = vec3(0.001, 0.000, 0.014);
        let c1 = vec3(0.329, 0.059, 0.406);
        let c2 = vec3(0.735, 0.216, 0.330);
        let c3 = vec3(0.976, 0.559, 0.040);
        let c4 = vec3(0.988, 0.998, 0.645);

        let t4 = t * 4.0;
        let i = floor(t4);
        let f = fract(t4);

        if i < 1.0 { return mix(c0, c1, f); }
        if i < 2.0 { return mix(c1, c2, f); }
        if i < 3.0 { return mix(c2, c3, f); }
        return mix(c3, c4, f);
    }

    fn colormap_cool_warm(t: f32) -> vec3 {
        let blue = vec3(0.230, 0.299, 0.754);
        let white = vec3(0.865, 0.865, 0.865);
        let red = vec3(0.706, 0.016, 0.150);

        if t < 0.5 {
            return mix(blue, white, t * 2.0);
        }
        return mix(white, red, (t - 0.5) * 2.0);
    }

    fn colormap_turbo(t: f32) -> vec3 {
        let r = 0.13572138 + t * (4.61539260 + t * (-42.66032258 + t * (132.13108234 + t * (-152.94239396 + t * 59.28637943))));
        let g = 0.09140261 + t * (2.19418839 + t * (4.84296658 + t * (-14.18503333 + t * (4.27729857 + t * 2.82956604))));
        let b = 0.10667330 + t * (12.64194608 + t * (-60.58204836 + t * (110.36276771 + t * (-89.90310912 + t * 27.34824973))));
        return vec3(clamp(r, 0.0, 1.0), clamp(g, 0.0, 1.0), clamp(b, 0.0, 1.0));
    }

    fn apply_colormap(t: f32, colormap_type: f32) -> vec3 {
        let t = clamp(t, 0.0, 1.0);

        if colormap_type < 0.5 { return colormap_viridis(t); }
        if colormap_type < 1.5 { return colormap_plasma(t); }
        if colormap_type < 2.5 { return colormap_inferno(t); }
        if colormap_type < 3.5 { return colormap_viridis(t); }  // Magma simplified to viridis
        if colormap_type < 4.5 { return colormap_cool_warm(t); }
        if colormap_type < 5.5 { return colormap_turbo(t); }
        return vec3(t, t, t);  // Grayscale
    }
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viridis_endpoints() {
        let c0 = Colormap::Viridis.sample(0.0);
        let c1 = Colormap::Viridis.sample(1.0);

        // Viridis starts dark purple
        assert!(c0.x < 0.3);
        assert!(c0.y < 0.1);
        assert!(c0.z < 0.4);

        // Viridis ends yellow
        assert!(c1.x > 0.9);
        assert!(c1.y > 0.8);
        assert!(c1.z < 0.3);
    }

    #[test]
    fn test_cool_warm_center() {
        let center = Colormap::CoolWarm.sample(0.5);

        // Center should be near white
        assert!((center.x - 0.865).abs() < 0.01);
        assert!((center.y - 0.865).abs() < 0.01);
        assert!((center.z - 0.865).abs() < 0.01);
    }

    #[test]
    fn test_grayscale() {
        let black = Colormap::Grayscale.sample(0.0);
        let gray = Colormap::Grayscale.sample(0.5);
        let white = Colormap::Grayscale.sample(1.0);

        assert_eq!(black, Vec3::ZERO);
        assert_eq!(gray, Vec3::splat(0.5));
        assert_eq!(white, Vec3::splat(1.0));
    }

    #[test]
    fn test_shader_value() {
        assert_eq!(Colormap::Viridis.to_shader_value(), 0.0);
        assert_eq!(Colormap::Plasma.to_shader_value(), 1.0);
        assert_eq!(Colormap::CoolWarm.to_shader_value(), 4.0);
    }

    #[test]
    fn test_rgba() {
        let rgba = Colormap::Viridis.sample_rgba(0.5);
        assert_eq!(rgba[3], 1.0); // Full opacity
    }

    #[test]
    fn test_clamping() {
        // Values outside [0, 1] should be clamped
        let below = Colormap::Viridis.sample(-1.0);
        let above = Colormap::Viridis.sample(2.0);
        let at_zero = Colormap::Viridis.sample(0.0);
        let at_one = Colormap::Viridis.sample(1.0);

        assert_eq!(below, at_zero);
        assert_eq!(above, at_one);
    }
}

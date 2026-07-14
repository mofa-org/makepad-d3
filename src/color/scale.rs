//! Color scale implementations
//!
//! Provides sequential, diverging, and categorical color scales for
//! mapping data values to colors.

use super::types::Rgba;

/// Trait for color scales that map values to colors
pub trait ColorScale: Send + Sync {
    /// Get a color for a normalized value (0.0 to 1.0)
    fn color(&self, t: f64) -> Rgba;

    /// Get the scale type name
    fn scale_type(&self) -> &'static str;
}

/// Sequential color scale for continuous data
///
/// Interpolates between a sequence of colors based on input value.
///
/// # Example
/// ```
/// use makepad_d3::color::{ColorScale, SequentialScale};
///
/// let scale = SequentialScale::viridis();
/// let color = scale.color(0.5);
/// ```
#[derive(Clone, Debug)]
pub struct SequentialScale {
    /// Color stops
    colors: Vec<Rgba>,
}

impl SequentialScale {
    /// Create a new sequential scale from colors
    pub fn new(colors: Vec<Rgba>) -> Self {
        assert!(
            !colors.is_empty(),
            "Sequential scale requires at least one color"
        );
        Self { colors }
    }

    /// Create from hex colors
    pub fn from_hex(hex_colors: &[u32]) -> Self {
        Self::new(hex_colors.iter().map(|&h| Rgba::from_hex(h)).collect())
    }

    /// Get the colors
    pub fn colors(&self) -> &[Rgba] {
        &self.colors
    }

    // ==================== D3 Sequential Scales ====================

    /// Viridis color scheme (perceptually uniform, colorblind safe)
    pub fn viridis() -> Self {
        Self::from_hex(&[
            0x440154, 0x482878, 0x3E4A89, 0x31688E, 0x26838E, 0x1F9E89, 0x35B779, 0x6DCD59,
            0xB4DE2C, 0xFDE725,
        ])
    }

    /// Plasma color scheme
    pub fn plasma() -> Self {
        Self::from_hex(&[
            0x0D0887, 0x46039F, 0x7201A8, 0x9C179E, 0xBD3786, 0xD8576B, 0xED7953, 0xFB9F3A,
            0xFDCA26, 0xF0F921,
        ])
    }

    /// Inferno color scheme
    pub fn inferno() -> Self {
        Self::from_hex(&[
            0x000004, 0x1B0C41, 0x4A0C6B, 0x781C6D, 0xA52C60, 0xCF4446, 0xED6925, 0xFB9A06,
            0xF7D03C, 0xFCFFA4,
        ])
    }

    /// Magma color scheme
    pub fn magma() -> Self {
        Self::from_hex(&[
            0x000004, 0x180F3D, 0x440F76, 0x721F81, 0x9E2F7F, 0xCD4071, 0xF1605D, 0xFD9668,
            0xFECA8E, 0xFCFDBF,
        ])
    }

    /// Cividis color scheme (colorblind safe)
    pub fn cividis() -> Self {
        Self::from_hex(&[
            0x002051, 0x0A326A, 0x2B446E, 0x4A5568, 0x636763, 0x7C7B57, 0x97904B, 0xB4A73E,
            0xD3C038, 0xFDEA45,
        ])
    }

    /// Blues single-hue sequential
    pub fn blues() -> Self {
        Self::from_hex(&[
            0xF7FBFF, 0xDEEBF7, 0xC6DBEF, 0x9ECAE1, 0x6BAED6, 0x4292C6, 0x2171B5, 0x08519C,
            0x08306B,
        ])
    }

    /// Greens single-hue sequential
    pub fn greens() -> Self {
        Self::from_hex(&[
            0xF7FCF5, 0xE5F5E0, 0xC7E9C0, 0xA1D99B, 0x74C476, 0x41AB5D, 0x238B45, 0x006D2C,
            0x00441B,
        ])
    }

    /// Oranges single-hue sequential
    pub fn oranges() -> Self {
        Self::from_hex(&[
            0xFFF5EB, 0xFEE6CE, 0xFDD0A2, 0xFDAE6B, 0xFD8D3C, 0xF16913, 0xD94801, 0xA63603,
            0x7F2704,
        ])
    }

    /// Purples single-hue sequential
    pub fn purples() -> Self {
        Self::from_hex(&[
            0xFCFBFD, 0xEFEDF5, 0xDADAEB, 0xBCBDDC, 0x9E9AC8, 0x807DBA, 0x6A51A3, 0x54278F,
            0x3F007D,
        ])
    }

    /// Reds single-hue sequential
    pub fn reds() -> Self {
        Self::from_hex(&[
            0xFFF5F0, 0xFEE0D2, 0xFCBBA1, 0xFC9272, 0xFB6A4A, 0xEF3B2C, 0xCB181D, 0xA50F15,
            0x67000D,
        ])
    }

    /// Greys single-hue sequential
    pub fn greys() -> Self {
        Self::from_hex(&[
            0xFFFFFF, 0xF0F0F0, 0xD9D9D9, 0xBDBDBD, 0x969696, 0x737373, 0x525252, 0x252525,
            0x000000,
        ])
    }

    /// Blue-Green sequential
    pub fn blue_green() -> Self {
        Self::from_hex(&[
            0xF7FCFD, 0xE5F5F9, 0xCCECE6, 0x99D8C9, 0x66C2A4, 0x41AE76, 0x238B45, 0x006D2C,
            0x00441B,
        ])
    }

    /// Blue-Purple sequential
    pub fn blue_purple() -> Self {
        Self::from_hex(&[
            0xF7FCFD, 0xE0ECF4, 0xBFD3E6, 0x9EBCDA, 0x8C96C6, 0x8C6BB1, 0x88419D, 0x810F7C,
            0x4D004B,
        ])
    }

    /// Yellow-Green sequential
    pub fn yellow_green() -> Self {
        Self::from_hex(&[
            0xFFFFE5, 0xF7FCB9, 0xD9F0A3, 0xADDD8E, 0x78C679, 0x41AB5D, 0x238B45, 0x006837,
            0x004529,
        ])
    }

    /// Yellow-Orange-Red sequential (heat)
    pub fn yellow_orange_red() -> Self {
        Self::from_hex(&[
            0xFFFFCC, 0xFFEDA0, 0xFED976, 0xFEB24C, 0xFD8D3C, 0xFC4E2A, 0xE31A1C, 0xBD0026,
            0x800026,
        ])
    }

    /// Warm sequential (brown to yellow)
    pub fn warm() -> Self {
        Self::from_hex(&[0x6E40AA, 0xBF3CAF, 0xFE4B83, 0xFF7847, 0xE2B72F, 0xAFF05B])
    }

    /// Cool sequential (cyan to purple)
    pub fn cool() -> Self {
        Self::from_hex(&[0x6E40AA, 0x4C6EDB, 0x32A0D7, 0x4DC7A9, 0xAFF05B])
    }
}

impl ColorScale for SequentialScale {
    fn color(&self, t: f64) -> Rgba {
        let t = t.clamp(0.0, 1.0) as f32;

        if self.colors.len() == 1 {
            return self.colors[0];
        }

        let n = self.colors.len() - 1;
        let scaled = t * n as f32;
        let i = (scaled.floor() as usize).min(n - 1);
        let local_t = scaled - i as f32;

        self.colors[i].lerp(&self.colors[i + 1], local_t)
    }

    fn scale_type(&self) -> &'static str {
        "sequential"
    }
}

/// Diverging color scale for data with a meaningful midpoint
///
/// Uses different color ramps for values below and above the midpoint.
///
/// # Example
/// ```
/// use makepad_d3::color::{ColorScale, DivergingScale};
///
/// let scale = DivergingScale::red_blue();
/// let negative = scale.color(0.0);  // Red
/// let neutral = scale.color(0.5);   // White
/// let positive = scale.color(1.0);  // Blue
/// ```
#[derive(Clone, Debug)]
pub struct DivergingScale {
    /// Colors for negative values (0.0 to 0.5)
    negative: Vec<Rgba>,
    /// Color at midpoint
    mid: Rgba,
    /// Colors for positive values (0.5 to 1.0)
    positive: Vec<Rgba>,
}

impl DivergingScale {
    /// Create a new diverging scale
    pub fn new(negative: Vec<Rgba>, mid: Rgba, positive: Vec<Rgba>) -> Self {
        Self {
            negative,
            mid,
            positive,
        }
    }

    /// Create from hex colors
    pub fn from_hex(negative: &[u32], mid: u32, positive: &[u32]) -> Self {
        Self::new(
            negative.iter().map(|&h| Rgba::from_hex(h)).collect(),
            Rgba::from_hex(mid),
            positive.iter().map(|&h| Rgba::from_hex(h)).collect(),
        )
    }

    // ==================== D3 Diverging Scales ====================

    /// Red-White-Blue diverging
    pub fn red_blue() -> Self {
        Self::from_hex(
            &[0xB2182B, 0xD6604D, 0xF4A582, 0xFDDBC7],
            0xF7F7F7,
            &[0xD1E5F0, 0x92C5DE, 0x4393C3, 0x2166AC],
        )
    }

    /// Red-Yellow-Green diverging
    pub fn red_yellow_green() -> Self {
        Self::from_hex(
            &[0xD73027, 0xF46D43, 0xFDAE61, 0xFEE08B],
            0xFFFFBF,
            &[0xD9EF8B, 0xA6D96A, 0x66BD63, 0x1A9850],
        )
    }

    /// Purple-Orange diverging
    pub fn purple_orange() -> Self {
        Self::from_hex(
            &[0x7F3B08, 0xB35806, 0xE08214, 0xFDB863],
            0xF7F7F7,
            &[0xD8DAEB, 0xB2ABD2, 0x8073AC, 0x542788],
        )
    }

    /// Brown-Blue-Green diverging
    pub fn brown_blue_green() -> Self {
        Self::from_hex(
            &[0x8C510A, 0xBF812D, 0xDFC27D, 0xF6E8C3],
            0xF5F5F5,
            &[0xC7EAE5, 0x80CDC1, 0x35978F, 0x01665E],
        )
    }

    /// Pink-Yellow-Green diverging
    pub fn pink_green() -> Self {
        Self::from_hex(
            &[0xC51B7D, 0xDE77AE, 0xF1B6DA, 0xFDE0EF],
            0xF7F7F7,
            &[0xE6F5D0, 0xB8E186, 0x7FBC41, 0x4D9221],
        )
    }

    /// Spectral (rainbow) diverging
    pub fn spectral() -> Self {
        Self::from_hex(
            &[0xD53E4F, 0xF46D43, 0xFDAE61, 0xFEE08B],
            0xFFFFBF,
            &[0xE6F598, 0xABDDA4, 0x66C2A5, 0x3288BD],
        )
    }
}

impl ColorScale for DivergingScale {
    fn color(&self, t: f64) -> Rgba {
        let t = t.clamp(0.0, 1.0) as f32;

        if (t - 0.5).abs() < 0.001 {
            return self.mid;
        }

        if t < 0.5 {
            // Negative side: map 0.0-0.5 to colors
            let local_t = t * 2.0; // Map to 0.0-1.0
            let n = self.negative.len();

            if n == 0 {
                return self.mid;
            }

            let scaled = local_t * n as f32;
            let i = (scaled.floor() as usize).min(n - 1);

            if i == n - 1 {
                self.negative[i].lerp(&self.mid, scaled - i as f32)
            } else {
                self.negative[i].lerp(&self.negative[i + 1], scaled - i as f32)
            }
        } else {
            // Positive side: map 0.5-1.0 to colors
            let local_t = (t - 0.5) * 2.0; // Map to 0.0-1.0
            let n = self.positive.len();

            if n == 0 {
                return self.mid;
            }

            let scaled = local_t * n as f32;
            let i = (scaled.floor() as usize).min(n - 1);

            if i == 0 && scaled < 1.0 {
                self.mid.lerp(&self.positive[0], scaled)
            } else if i >= n - 1 {
                self.positive[n - 1]
            } else {
                self.positive[i].lerp(&self.positive[i + 1], scaled - i as f32)
            }
        }
    }

    fn scale_type(&self) -> &'static str {
        "diverging"
    }
}

/// Categorical color scale for discrete categories
///
/// Returns distinct colors for each category index.
///
/// # Example
/// ```
/// use makepad_d3::color::{ColorScale, CategoricalScale};
///
/// let scale = CategoricalScale::category10();
/// let color0 = scale.color(0.0);  // First category
/// let color1 = scale.color(0.1);  // Second category (if 10 categories)
/// ```
#[derive(Clone, Debug)]
pub struct CategoricalScale {
    /// Category colors
    colors: Vec<Rgba>,
}

impl CategoricalScale {
    /// Create a new categorical scale
    pub fn new(colors: Vec<Rgba>) -> Self {
        assert!(
            !colors.is_empty(),
            "Categorical scale requires at least one color"
        );
        Self { colors }
    }

    /// Create from hex colors
    pub fn from_hex(hex_colors: &[u32]) -> Self {
        Self::new(hex_colors.iter().map(|&h| Rgba::from_hex(h)).collect())
    }

    /// Get color by index (wraps around)
    pub fn get(&self, index: usize) -> Rgba {
        self.colors[index % self.colors.len()]
    }

    /// Get the number of colors
    pub fn len(&self) -> usize {
        self.colors.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.colors.is_empty()
    }

    /// Get all colors
    pub fn colors(&self) -> &[Rgba] {
        &self.colors
    }

    // ==================== D3 Categorical Scales ====================

    /// D3 Category10 - 10 distinct colors
    pub fn category10() -> Self {
        Self::from_hex(&[
            0x1F77B4, // Blue
            0xFF7F0E, // Orange
            0x2CA02C, // Green
            0xD62728, // Red
            0x9467BD, // Purple
            0x8C564B, // Brown
            0xE377C2, // Pink
            0x7F7F7F, // Gray
            0xBCBD22, // Olive
            0x17BECF, // Cyan
        ])
    }

    /// Tableau 10 colors
    pub fn tableau10() -> Self {
        Self::from_hex(&[
            0x4E79A7, // Blue
            0xF28E2B, // Orange
            0xE15759, // Red
            0x76B7B2, // Teal
            0x59A14F, // Green
            0xEDC949, // Yellow
            0xAF7AA1, // Purple
            0xFF9DA7, // Pink
            0x9C755F, // Brown
            0xBAB0AC, // Gray
        ])
    }

    /// D3 Category20 - 20 paired colors
    pub fn category20() -> Self {
        Self::from_hex(&[
            0x1F77B4, 0xAEC7E8, 0xFF7F0E, 0xFFBB78, 0x2CA02C, 0x98DF8A, 0xD62728, 0xFF9896,
            0x9467BD, 0xC5B0D5, 0x8C564B, 0xC49C94, 0xE377C2, 0xF7B6D2, 0x7F7F7F, 0xC7C7C7,
            0xBCBD22, 0xDBDB8D, 0x17BECF, 0x9EDAE5,
        ])
    }

    /// Set1 - Qualitative colorblind-safe
    pub fn set1() -> Self {
        Self::from_hex(&[
            0xE41A1C, 0x377EB8, 0x4DAF4A, 0x984EA3, 0xFF7F00, 0xFFFF33, 0xA65628, 0xF781BF,
            0x999999,
        ])
    }

    /// Set2 - Qualitative pastel
    pub fn set2() -> Self {
        Self::from_hex(&[
            0x66C2A5, 0xFC8D62, 0x8DA0CB, 0xE78AC3, 0xA6D854, 0xFFD92F, 0xE5C494, 0xB3B3B3,
        ])
    }

    /// Set3 - Qualitative
    pub fn set3() -> Self {
        Self::from_hex(&[
            0x8DD3C7, 0xFFFFB3, 0xBEBADA, 0xFB8072, 0x80B1D3, 0xFDB462, 0xB3DE69, 0xFCCDE5,
            0xD9D9D9, 0xBC80BD, 0xCCEBC5, 0xFFED6F,
        ])
    }

    /// Pastel1 colors
    pub fn pastel1() -> Self {
        Self::from_hex(&[
            0xFBB4AE, 0xB3CDE3, 0xCCEBC5, 0xDECBE4, 0xFED9A6, 0xFFFFCC, 0xE5D8BD, 0xFDDAEC,
            0xF2F2F2,
        ])
    }

    /// Pastel2 colors
    pub fn pastel2() -> Self {
        Self::from_hex(&[
            0xB3E2CD, 0xFDCDAC, 0xCBD5E8, 0xF4CAE4, 0xE6F5C9, 0xFFF2AE, 0xF1E2CC, 0xCCCCCC,
        ])
    }

    /// Dark2 colors
    pub fn dark2() -> Self {
        Self::from_hex(&[
            0x1B9E77, 0xD95F02, 0x7570B3, 0xE7298A, 0x66A61E, 0xE6AB02, 0xA6761D, 0x666666,
        ])
    }

    /// Paired colors (light/dark pairs)
    pub fn paired() -> Self {
        Self::from_hex(&[
            0xA6CEE3, 0x1F78B4, 0xB2DF8A, 0x33A02C, 0xFB9A99, 0xE31A1C, 0xFDBF6F, 0xFF7F00,
            0xCAB2D6, 0x6A3D9A, 0xFFFF99, 0xB15928,
        ])
    }

    /// Accent colors
    pub fn accent() -> Self {
        Self::from_hex(&[
            0x7FC97F, 0xBEAED4, 0xFDC086, 0xFFFF99, 0x386CB0, 0xF0027F, 0xBF5B17, 0x666666,
        ])
    }
}

impl ColorScale for CategoricalScale {
    fn color(&self, t: f64) -> Rgba {
        // Map t to an index
        let index = ((t.clamp(0.0, 1.0) * self.colors.len() as f64).floor() as usize)
            .min(self.colors.len() - 1);
        self.colors[index]
    }

    fn scale_type(&self) -> &'static str {
        "categorical"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequential_viridis() {
        let scale = SequentialScale::viridis();

        let start = scale.color(0.0);
        let mid = scale.color(0.5);
        let end = scale.color(1.0);

        // Should be different colors
        assert_ne!(start.to_hex(), mid.to_hex());
        assert_ne!(mid.to_hex(), end.to_hex());
    }

    #[test]
    fn test_sequential_interpolation() {
        let scale = SequentialScale::from_hex(&[0x000000, 0xFFFFFF]);

        let mid = scale.color(0.5);
        assert!((mid.r - 0.5).abs() < 0.01);
        assert!((mid.g - 0.5).abs() < 0.01);
        assert!((mid.b - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_diverging_red_blue() {
        let scale = DivergingScale::red_blue();

        let negative = scale.color(0.0);
        let mid = scale.color(0.5);
        let positive = scale.color(1.0);

        // Negative should be reddish
        assert!(negative.r > negative.b);
        // Positive should be bluish
        assert!(positive.b > positive.r);
        // Mid should be neutral (grayish)
        assert!((mid.r - mid.g).abs() < 0.1);
    }

    #[test]
    fn test_categorical_category10() {
        let scale = CategoricalScale::category10();

        assert_eq!(scale.len(), 10);

        // All colors should be distinct
        for i in 0..10 {
            for j in (i + 1)..10 {
                assert_ne!(scale.get(i).to_hex(), scale.get(j).to_hex());
            }
        }
    }

    #[test]
    fn test_categorical_wrapping() {
        let scale = CategoricalScale::category10();

        // Index 10 should wrap to 0
        assert_eq!(scale.get(0).to_hex(), scale.get(10).to_hex());
        assert_eq!(scale.get(1).to_hex(), scale.get(11).to_hex());
    }

    #[test]
    fn test_sequential_clamping() {
        let scale = SequentialScale::viridis();

        // Values outside [0, 1] should be clamped
        let below = scale.color(-0.5);
        let at_zero = scale.color(0.0);
        assert_eq!(below.to_hex(), at_zero.to_hex());

        let above = scale.color(1.5);
        let at_one = scale.color(1.0);
        assert_eq!(above.to_hex(), at_one.to_hex());
    }
}

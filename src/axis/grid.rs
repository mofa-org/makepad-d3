//! Grid line configuration and styling
//!
//! This module provides configuration for grid lines that extend from axis tick marks
//! across the chart area.

/// Grid line style
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GridLineStyle {
    /// Solid line
    #[default]
    Solid,
    /// Dashed line
    Dashed,
    /// Dotted line
    Dotted,
    /// Dash-dot pattern
    DashDot,
}

impl GridLineStyle {
    /// Get the dash pattern for this style (dash length, gap length)
    /// Returns None for solid lines
    pub fn dash_pattern(&self) -> Option<(f64, f64)> {
        match self {
            Self::Solid => None,
            Self::Dashed => Some((5.0, 3.0)),
            Self::Dotted => Some((1.0, 2.0)),
            Self::DashDot => Some((5.0, 2.0)), // Note: true dash-dot needs more complex pattern
        }
    }
}

/// Configuration for grid lines
#[derive(Clone, Debug)]
pub struct GridConfig {
    /// Whether grid is enabled
    pub enabled: bool,
    /// Grid line style
    pub style: GridLineStyle,
    /// Grid line width in pixels
    pub line_width: f64,
    /// Grid line color (RGBA, 0.0-1.0)
    pub color: [f64; 4],
    /// Grid line opacity (0.0-1.0)
    pub opacity: f64,
    /// Show grid lines at zero value
    pub zero_line: bool,
    /// Zero line uses different styling
    pub zero_line_width: f64,
    /// Zero line color (RGBA)
    pub zero_line_color: [f64; 4],
    /// Offset from tick position (for aligning with bands)
    pub offset: f64,
    /// Whether to show minor grid lines
    pub show_minor: bool,
    /// Minor grid line opacity multiplier
    pub minor_opacity: f64,
    /// Minor grid line width multiplier
    pub minor_width: f64,
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            style: GridLineStyle::Solid,
            line_width: 1.0,
            color: [0.8, 0.8, 0.8, 1.0], // Light gray
            opacity: 1.0,
            zero_line: false,
            zero_line_width: 2.0,
            zero_line_color: [0.5, 0.5, 0.5, 1.0], // Darker gray
            offset: 0.0,
            show_minor: false,
            minor_opacity: 0.5,
            minor_width: 0.5,
        }
    }
}

impl GridConfig {
    /// Create a new grid configuration (disabled by default)
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable grid lines
    pub fn enabled(mut self) -> Self {
        self.enabled = true;
        self
    }

    /// Set grid line style
    pub fn with_style(mut self, style: GridLineStyle) -> Self {
        self.style = style;
        self
    }

    /// Use dashed grid lines
    pub fn dashed(mut self) -> Self {
        self.style = GridLineStyle::Dashed;
        self
    }

    /// Use dotted grid lines
    pub fn dotted(mut self) -> Self {
        self.style = GridLineStyle::Dotted;
        self
    }

    /// Set grid line width
    pub fn with_line_width(mut self, width: f64) -> Self {
        self.line_width = width;
        self
    }

    /// Set grid line color (RGB)
    pub fn with_color(mut self, r: f64, g: f64, b: f64) -> Self {
        self.color = [r, g, b, 1.0];
        self
    }

    /// Set grid line color with alpha (RGBA)
    pub fn with_color_rgba(mut self, r: f64, g: f64, b: f64, a: f64) -> Self {
        self.color = [r, g, b, a];
        self
    }

    /// Set grid line opacity
    pub fn with_opacity(mut self, opacity: f64) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    /// Enable zero line with emphasis
    pub fn with_zero_line(mut self) -> Self {
        self.zero_line = true;
        self
    }

    /// Set zero line width
    pub fn with_zero_line_width(mut self, width: f64) -> Self {
        self.zero_line_width = width;
        self
    }

    /// Set zero line color
    pub fn with_zero_line_color(mut self, r: f64, g: f64, b: f64) -> Self {
        self.zero_line_color = [r, g, b, 1.0];
        self
    }

    /// Set grid offset (useful for band scales)
    pub fn with_offset(mut self, offset: f64) -> Self {
        self.offset = offset;
        self
    }

    /// Enable minor grid lines
    pub fn with_minor_grid(mut self) -> Self {
        self.show_minor = true;
        self
    }

    /// Set minor grid opacity multiplier (relative to major grid)
    pub fn with_minor_opacity(mut self, multiplier: f64) -> Self {
        self.minor_opacity = multiplier.clamp(0.0, 1.0);
        self
    }

    /// Set minor grid width multiplier (relative to major grid)
    pub fn with_minor_width(mut self, multiplier: f64) -> Self {
        self.minor_width = multiplier.clamp(0.0, 1.0);
        self
    }

    /// Check if grid is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get effective color with opacity applied
    pub fn effective_color(&self) -> [f64; 4] {
        [
            self.color[0],
            self.color[1],
            self.color[2],
            self.color[3] * self.opacity,
        ]
    }

    /// Get effective minor grid color
    pub fn effective_minor_color(&self) -> [f64; 4] {
        [
            self.color[0],
            self.color[1],
            self.color[2],
            self.color[3] * self.opacity * self.minor_opacity,
        ]
    }

    /// Get effective minor grid line width
    pub fn effective_minor_width(&self) -> f64 {
        self.line_width * self.minor_width
    }

    /// Check if a value represents the zero line
    pub fn is_zero_value(&self, value: f64) -> bool {
        self.zero_line && value.abs() < f64::EPSILON
    }

    /// Get styling for a specific value
    pub fn get_line_style(&self, value: f64, is_minor: bool) -> GridLineParams {
        if self.is_zero_value(value) {
            GridLineParams {
                width: self.zero_line_width,
                color: self.zero_line_color,
                style: GridLineStyle::Solid, // Zero line always solid
            }
        } else if is_minor {
            GridLineParams {
                width: self.effective_minor_width(),
                color: self.effective_minor_color(),
                style: self.style,
            }
        } else {
            GridLineParams {
                width: self.line_width,
                color: self.effective_color(),
                style: self.style,
            }
        }
    }
}

/// Computed grid line parameters for rendering
#[derive(Clone, Debug)]
pub struct GridLineParams {
    /// Line width
    pub width: f64,
    /// Line color (RGBA)
    pub color: [f64; 4],
    /// Line style
    pub style: GridLineStyle,
}

/// A computed grid line ready for rendering
#[derive(Clone, Debug)]
pub struct GridLine {
    /// Start point (x, y)
    pub start: (f64, f64),
    /// End point (x, y)
    pub end: (f64, f64),
    /// Rendering parameters
    pub params: GridLineParams,
    /// The tick value this grid line corresponds to
    pub value: f64,
    /// Whether this is a minor grid line
    pub is_minor: bool,
}

/// Presets for common grid configurations
impl GridConfig {
    /// Light gray dashed grid
    pub fn light_dashed() -> Self {
        Self::new().enabled().dashed().with_color(0.9, 0.9, 0.9)
    }

    /// Subtle dotted grid
    pub fn subtle_dotted() -> Self {
        Self::new()
            .enabled()
            .dotted()
            .with_color(0.85, 0.85, 0.85)
            .with_opacity(0.7)
    }

    /// Bold solid grid
    pub fn bold() -> Self {
        Self::new()
            .enabled()
            .with_line_width(1.5)
            .with_color(0.7, 0.7, 0.7)
    }

    /// Minimal grid (very light)
    pub fn minimal() -> Self {
        Self::new()
            .enabled()
            .with_color(0.95, 0.95, 0.95)
            .with_line_width(0.5)
    }

    /// Professional grid with zero line emphasis
    pub fn professional() -> Self {
        Self::new()
            .enabled()
            .dashed()
            .with_color(0.85, 0.85, 0.85)
            .with_zero_line()
            .with_zero_line_color(0.5, 0.5, 0.5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_config_default() {
        let config = GridConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.style, GridLineStyle::Solid);
        assert_eq!(config.line_width, 1.0);
    }

    #[test]
    fn test_grid_config_builder() {
        let config = GridConfig::new()
            .enabled()
            .dashed()
            .with_line_width(2.0)
            .with_color(0.5, 0.5, 0.5)
            .with_opacity(0.8);

        assert!(config.enabled);
        assert_eq!(config.style, GridLineStyle::Dashed);
        assert_eq!(config.line_width, 2.0);
        assert_eq!(config.color[0], 0.5);
        assert_eq!(config.opacity, 0.8);
    }

    #[test]
    fn test_dash_pattern() {
        assert!(GridLineStyle::Solid.dash_pattern().is_none());
        assert!(GridLineStyle::Dashed.dash_pattern().is_some());
        assert!(GridLineStyle::Dotted.dash_pattern().is_some());
    }

    #[test]
    fn test_effective_color() {
        let config = GridConfig::new()
            .with_color(1.0, 0.0, 0.0)
            .with_opacity(0.5);

        let effective = config.effective_color();
        assert_eq!(effective[0], 1.0);
        assert_eq!(effective[1], 0.0);
        assert_eq!(effective[2], 0.0);
        assert_eq!(effective[3], 0.5);
    }

    #[test]
    fn test_zero_line_detection() {
        let config = GridConfig::new().with_zero_line();
        assert!(config.is_zero_value(0.0));
        assert!(!config.is_zero_value(1.0));
        assert!(!config.is_zero_value(-1.0));

        let no_zero = GridConfig::new();
        assert!(!no_zero.is_zero_value(0.0));
    }

    #[test]
    fn test_minor_grid() {
        let config = GridConfig::new()
            .with_line_width(2.0)
            .with_minor_grid()
            .with_minor_opacity(0.5)
            .with_minor_width(0.5);

        assert!(config.show_minor);
        assert_eq!(config.effective_minor_width(), 1.0);
    }

    #[test]
    fn test_get_line_style() {
        let config = GridConfig::new()
            .with_line_width(2.0)
            .with_zero_line()
            .with_zero_line_width(3.0);

        // Zero line
        let zero_style = config.get_line_style(0.0, false);
        assert_eq!(zero_style.width, 3.0);

        // Regular line
        let regular_style = config.get_line_style(50.0, false);
        assert_eq!(regular_style.width, 2.0);
    }

    #[test]
    fn test_presets() {
        let light = GridConfig::light_dashed();
        assert!(light.enabled);
        assert_eq!(light.style, GridLineStyle::Dashed);

        let subtle = GridConfig::subtle_dotted();
        assert_eq!(subtle.style, GridLineStyle::Dotted);

        let pro = GridConfig::professional();
        assert!(pro.zero_line);
    }

    #[test]
    fn test_grid_line_params() {
        let config = GridConfig::new()
            .with_line_width(1.5)
            .with_color(0.8, 0.8, 0.8);

        let params = config.get_line_style(10.0, false);
        assert_eq!(params.width, 1.5);
        assert_eq!(params.color[0], 0.8);
    }
}

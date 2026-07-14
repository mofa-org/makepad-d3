//! Core axis implementation
//!
//! Provides axis configuration and layout computation for chart axes.

use super::format::NumberFormat;
use super::grid::GridConfig;
use crate::scale::{BandScale, DiscreteScale, PointScale, Scale, Tick, TickOptions};

/// Axis orientation
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AxisOrientation {
    /// Axis at the bottom of the chart (x-axis, labels below)
    #[default]
    Bottom,
    /// Axis at the top of the chart (x-axis, labels above)
    Top,
    /// Axis at the left of the chart (y-axis, labels left)
    Left,
    /// Axis at the right of the chart (y-axis, labels right)
    Right,
}

impl AxisOrientation {
    /// Check if this is a horizontal axis (Bottom or Top)
    pub fn is_horizontal(&self) -> bool {
        matches!(self, Self::Bottom | Self::Top)
    }

    /// Check if this is a vertical axis (Left or Right)
    pub fn is_vertical(&self) -> bool {
        matches!(self, Self::Left | Self::Right)
    }

    /// Get the default text anchor for this orientation
    pub fn default_text_anchor(&self) -> TextAnchor {
        match self {
            Self::Bottom | Self::Top => TextAnchor::Middle,
            Self::Left => TextAnchor::End,
            Self::Right => TextAnchor::Start,
        }
    }

    /// Get the default label alignment for this orientation
    pub fn default_label_align(&self) -> LabelAlign {
        match self {
            Self::Bottom => LabelAlign::Top,
            Self::Top => LabelAlign::Bottom,
            Self::Left => LabelAlign::Right,
            Self::Right => LabelAlign::Left,
        }
    }
}

/// Text anchor for label positioning
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextAnchor {
    /// Anchor at the start (left for horizontal text)
    Start,
    /// Anchor at the middle (centered)
    #[default]
    Middle,
    /// Anchor at the end (right for horizontal text)
    End,
}

/// Vertical alignment for labels
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LabelAlign {
    /// Align to top
    Top,
    /// Align to middle (vertically centered)
    #[default]
    Middle,
    /// Align to bottom
    Bottom,
    /// Align to left (for vertical axes)
    Left,
    /// Align to right (for vertical axes)
    Right,
}

/// Label rotation specification
#[derive(Clone, Copy, Debug, Default)]
pub struct LabelRotation {
    /// Rotation angle in degrees (positive = clockwise)
    pub angle: f64,
    /// Text anchor after rotation
    pub anchor: Option<TextAnchor>,
    /// Vertical alignment after rotation
    pub align: Option<LabelAlign>,
}

impl LabelRotation {
    /// No rotation
    pub fn none() -> Self {
        Self {
            angle: 0.0,
            anchor: None,
            align: None,
        }
    }

    /// Rotate labels by a given angle
    pub fn degrees(angle: f64) -> Self {
        Self {
            angle,
            anchor: None,
            align: None,
        }
    }

    /// Common rotation for crowded bottom axis (-45 degrees)
    pub fn diagonal() -> Self {
        Self {
            angle: -45.0,
            anchor: Some(TextAnchor::End),
            align: Some(LabelAlign::Top),
        }
    }

    /// Vertical rotation for very long labels (-90 degrees)
    pub fn vertical() -> Self {
        Self {
            angle: -90.0,
            anchor: Some(TextAnchor::End),
            align: Some(LabelAlign::Middle),
        }
    }

    /// Set the text anchor
    pub fn with_anchor(mut self, anchor: TextAnchor) -> Self {
        self.anchor = Some(anchor);
        self
    }

    /// Set the vertical alignment
    pub fn with_align(mut self, align: LabelAlign) -> Self {
        self.align = Some(align);
        self
    }

    /// Check if rotation is applied
    pub fn is_rotated(&self) -> bool {
        self.angle.abs() > f64::EPSILON
    }

    /// Get angle in radians
    pub fn radians(&self) -> f64 {
        self.angle.to_radians()
    }
}

/// Configuration for axis appearance
#[derive(Clone, Debug)]
pub struct AxisConfig {
    /// Axis orientation
    pub orientation: AxisOrientation,
    /// Length of tick marks in pixels
    pub tick_size: f64,
    /// Inner tick size (between axis and labels)
    pub tick_size_inner: f64,
    /// Outer tick size (extends beyond axis line)
    pub tick_size_outer: f64,
    /// Padding between tick marks and labels
    pub tick_padding: f64,
    /// Additional offset for labels
    pub label_offset: f64,
    /// Whether to show the domain line
    pub show_domain_line: bool,
    /// Whether to show grid lines
    pub show_grid: bool,
    /// Length of grid lines
    pub grid_length: f64,
    /// Number format for labels
    pub format: NumberFormat,
    /// Tick generation options
    pub tick_options: TickOptions,
    /// Label rotation
    pub label_rotation: LabelRotation,
    /// Text anchor for labels
    pub text_anchor: Option<TextAnchor>,
    /// Offset for band/point scale ticks (half bandwidth)
    pub band_offset: f64,
    /// Enhanced grid configuration
    pub grid_config: GridConfig,
}

impl Default for AxisConfig {
    fn default() -> Self {
        Self {
            orientation: AxisOrientation::Bottom,
            tick_size: 6.0,
            tick_size_inner: 6.0,
            tick_size_outer: 6.0,
            tick_padding: 3.0,
            label_offset: 0.0,
            show_domain_line: true,
            show_grid: false,
            grid_length: 0.0,
            format: NumberFormat::Auto,
            tick_options: TickOptions::default(),
            label_rotation: LabelRotation::none(),
            text_anchor: None,
            band_offset: 0.0,
            grid_config: GridConfig::default(),
        }
    }
}

impl AxisConfig {
    /// Create a new axis configuration with given orientation
    pub fn new(orientation: AxisOrientation) -> Self {
        Self {
            orientation,
            ..Default::default()
        }
    }

    /// Create a bottom axis configuration
    pub fn bottom() -> Self {
        Self::new(AxisOrientation::Bottom)
    }

    /// Create a top axis configuration
    pub fn top() -> Self {
        Self::new(AxisOrientation::Top)
    }

    /// Create a left axis configuration
    pub fn left() -> Self {
        Self::new(AxisOrientation::Left)
    }

    /// Create a right axis configuration
    pub fn right() -> Self {
        Self::new(AxisOrientation::Right)
    }

    /// Set tick size (affects both inner and outer)
    pub fn with_tick_size(mut self, size: f64) -> Self {
        self.tick_size = size;
        self.tick_size_inner = size;
        self.tick_size_outer = size;
        self
    }

    /// Set inner tick size only
    pub fn with_tick_size_inner(mut self, size: f64) -> Self {
        self.tick_size_inner = size;
        self
    }

    /// Set outer tick size only
    pub fn with_tick_size_outer(mut self, size: f64) -> Self {
        self.tick_size_outer = size;
        self
    }

    /// Set tick padding
    pub fn with_tick_padding(mut self, padding: f64) -> Self {
        self.tick_padding = padding;
        self
    }

    /// Set label offset
    pub fn with_label_offset(mut self, offset: f64) -> Self {
        self.label_offset = offset;
        self
    }

    /// Show or hide the domain line
    pub fn with_domain_line(mut self, show: bool) -> Self {
        self.show_domain_line = show;
        self
    }

    /// Enable grid lines with specified length
    pub fn with_grid(mut self, length: f64) -> Self {
        self.show_grid = true;
        self.grid_length = length;
        self
    }

    /// Disable grid lines
    pub fn without_grid(mut self) -> Self {
        self.show_grid = false;
        self
    }

    /// Set the number format
    pub fn with_format(mut self, format: NumberFormat) -> Self {
        self.format = format;
        self
    }

    /// Set tick options
    pub fn with_tick_options(mut self, options: TickOptions) -> Self {
        self.tick_options = options;
        self
    }

    /// Set the number of ticks
    pub fn with_tick_count(mut self, count: usize) -> Self {
        self.tick_options.count = count;
        self
    }

    /// Set label rotation
    pub fn with_label_rotation(mut self, rotation: LabelRotation) -> Self {
        self.label_rotation = rotation;
        self
    }

    /// Set diagonal label rotation (common for crowded x-axis)
    pub fn with_diagonal_labels(mut self) -> Self {
        self.label_rotation = LabelRotation::diagonal();
        self
    }

    /// Set vertical label rotation (for very long labels)
    pub fn with_vertical_labels(mut self) -> Self {
        self.label_rotation = LabelRotation::vertical();
        self
    }

    /// Set text anchor for labels
    pub fn with_text_anchor(mut self, anchor: TextAnchor) -> Self {
        self.text_anchor = Some(anchor);
        self
    }

    /// Set band offset (for centering ticks on bands)
    pub fn with_band_offset(mut self, offset: f64) -> Self {
        self.band_offset = offset;
        self
    }

    /// Set enhanced grid configuration
    pub fn with_grid_config(mut self, config: GridConfig) -> Self {
        self.show_grid = config.is_enabled();
        self.grid_config = config;
        self
    }

    /// Get effective text anchor (explicit or default for orientation)
    pub fn effective_text_anchor(&self) -> TextAnchor {
        if self.label_rotation.is_rotated() {
            self.label_rotation
                .anchor
                .unwrap_or(self.orientation.default_text_anchor())
        } else {
            self.text_anchor
                .unwrap_or_else(|| self.orientation.default_text_anchor())
        }
    }

    /// Get effective label alignment
    pub fn effective_label_align(&self) -> LabelAlign {
        if self.label_rotation.is_rotated() {
            self.label_rotation
                .align
                .unwrap_or(self.orientation.default_label_align())
        } else {
            self.orientation.default_label_align()
        }
    }
}

/// A positioned tick mark with computed layout information
#[derive(Clone, Debug)]
pub struct AxisTick {
    /// The underlying tick data
    pub tick: Tick,
    /// Formatted label string
    pub label: String,
    /// Position along the axis (in pixels)
    pub position: f64,
    /// Start point of tick line
    pub tick_start: (f64, f64),
    /// End point of tick line
    pub tick_end: (f64, f64),
    /// Position for the label
    pub label_position: (f64, f64),
    /// Grid line end point (if grid enabled)
    pub grid_end: Option<(f64, f64)>,
    /// Label rotation angle in degrees
    pub label_rotation: f64,
    /// Text anchor for this tick's label
    pub text_anchor: TextAnchor,
    /// Whether this is a minor tick
    pub is_minor: bool,
}

/// Computed axis layout ready for rendering
#[derive(Clone, Debug)]
pub struct AxisLayout {
    /// Axis orientation
    pub orientation: AxisOrientation,
    /// Scale range (start, end)
    pub range: (f64, f64),
    /// Domain line start point
    pub domain_start: (f64, f64),
    /// Domain line end point
    pub domain_end: (f64, f64),
    /// Whether to show domain line
    pub show_domain_line: bool,
    /// Computed tick layouts
    pub ticks: Vec<AxisTick>,
    /// Label rotation angle for all ticks
    pub label_rotation: f64,
    /// Default text anchor
    pub text_anchor: TextAnchor,
    /// Label alignment
    pub label_align: LabelAlign,
    /// Grid configuration
    pub grid_config: GridConfig,
}

/// Axis instance that computes layout from scale
#[derive(Clone, Debug)]
pub struct Axis {
    config: AxisConfig,
    ticks: Vec<Tick>,
    range: (f64, f64),
    /// Bandwidth for discrete scales (0 for continuous)
    bandwidth: f64,
}

impl Default for Axis {
    fn default() -> Self {
        Self::new()
    }
}

impl Axis {
    /// Create a new axis with default configuration
    pub fn new() -> Self {
        Self {
            config: AxisConfig::default(),
            ticks: Vec::new(),
            range: (0.0, 1.0),
            bandwidth: 0.0,
        }
    }

    /// Create an axis with specific configuration
    pub fn with_config(config: AxisConfig) -> Self {
        Self {
            config,
            ticks: Vec::new(),
            range: (0.0, 1.0),
            bandwidth: 0.0,
        }
    }

    /// Get the axis configuration
    pub fn config(&self) -> &AxisConfig {
        &self.config
    }

    /// Get mutable access to the configuration
    pub fn config_mut(&mut self) -> &mut AxisConfig {
        &mut self.config
    }

    /// Set the configuration
    pub fn set_config(&mut self, config: AxisConfig) {
        self.config = config;
    }

    /// Set ticks directly
    pub fn set_ticks(&mut self, ticks: Vec<Tick>) {
        self.ticks = ticks;
    }

    /// Get the current ticks
    pub fn ticks(&self) -> &[Tick] {
        &self.ticks
    }

    /// Set the scale range
    pub fn set_range(&mut self, range: (f64, f64)) {
        self.range = range;
    }

    /// Update axis from a scale
    pub fn set_scale<S: Scale>(&mut self, scale: &S) {
        self.ticks = scale.ticks(&self.config.tick_options);
        self.range = scale.range();
    }

    /// Update axis from a scale with custom tick options
    pub fn set_scale_with_options<S: Scale>(&mut self, scale: &S, options: &TickOptions) {
        self.ticks = scale.ticks(options);
        self.range = scale.range();
    }

    /// Update axis from a band scale
    pub fn set_band_scale(&mut self, scale: &BandScale) {
        self.ticks = scale.ticks(&self.config.tick_options);
        self.range = scale.range();
        self.bandwidth = scale.bandwidth();
        // For band scales, center ticks on bands by default
        self.config.band_offset = self.bandwidth / 2.0;
    }

    /// Update axis from a point scale
    pub fn set_point_scale(&mut self, scale: &PointScale) {
        self.ticks = scale.ticks(&self.config.tick_options);
        self.range = scale.range();
        self.bandwidth = 0.0; // Point scales have zero bandwidth
    }

    /// Get the bandwidth (for discrete scales)
    pub fn bandwidth(&self) -> f64 {
        self.bandwidth
    }

    /// Set bandwidth directly
    pub fn set_bandwidth(&mut self, bandwidth: f64) {
        self.bandwidth = bandwidth;
    }

    /// Compute axis layout at a given position
    ///
    /// For horizontal axes (Bottom/Top), `axis_position` is the Y coordinate.
    /// For vertical axes (Left/Right), `axis_position` is the X coordinate.
    pub fn compute_layout(&self, axis_position: f64) -> AxisLayout {
        let orientation = self.config.orientation;
        let range = self.range;

        // Compute domain line endpoints
        let (domain_start, domain_end) = match orientation {
            AxisOrientation::Bottom | AxisOrientation::Top => {
                ((range.0, axis_position), (range.1, axis_position))
            }
            AxisOrientation::Left | AxisOrientation::Right => {
                ((axis_position, range.0), (axis_position, range.1))
            }
        };

        // Compute tick layouts
        let ticks: Vec<AxisTick> = self
            .ticks
            .iter()
            .map(|tick| self.compute_tick_layout(tick, axis_position, false))
            .collect();

        AxisLayout {
            orientation,
            range,
            domain_start,
            domain_end,
            show_domain_line: self.config.show_domain_line,
            ticks,
            label_rotation: self.config.label_rotation.angle,
            text_anchor: self.config.effective_text_anchor(),
            label_align: self.config.effective_label_align(),
            grid_config: self.config.grid_config.clone(),
        }
    }

    /// Compute layout for a single tick
    fn compute_tick_layout(&self, tick: &Tick, axis_position: f64, is_minor: bool) -> AxisTick {
        // Apply band offset for discrete scales
        let pos = tick.position + self.config.band_offset;
        let tick_size = self.config.tick_size_inner;
        let padding = self.config.tick_padding;
        let label_offset = self.config.label_offset;
        let grid_length = self.config.grid_length;

        let (tick_start, tick_end, label_position, grid_end) = match self.config.orientation {
            AxisOrientation::Bottom => {
                let start = (pos, axis_position);
                let end = (pos, axis_position + tick_size);
                let label = (pos, axis_position + tick_size + padding + label_offset);
                let grid = if self.config.show_grid {
                    Some((pos, axis_position - grid_length))
                } else {
                    None
                };
                (start, end, label, grid)
            }
            AxisOrientation::Top => {
                let start = (pos, axis_position);
                let end = (pos, axis_position - tick_size);
                let label = (pos, axis_position - tick_size - padding - label_offset);
                let grid = if self.config.show_grid {
                    Some((pos, axis_position + grid_length))
                } else {
                    None
                };
                (start, end, label, grid)
            }
            AxisOrientation::Left => {
                let start = (axis_position, pos);
                let end = (axis_position - tick_size, pos);
                let label = (axis_position - tick_size - padding - label_offset, pos);
                let grid = if self.config.show_grid {
                    Some((axis_position + grid_length, pos))
                } else {
                    None
                };
                (start, end, label, grid)
            }
            AxisOrientation::Right => {
                let start = (axis_position, pos);
                let end = (axis_position + tick_size, pos);
                let label = (axis_position + tick_size + padding + label_offset, pos);
                let grid = if self.config.show_grid {
                    Some((axis_position - grid_length, pos))
                } else {
                    None
                };
                (start, end, label, grid)
            }
        };

        // Format the label
        // Use axis format when explicitly set (not Auto), otherwise use tick's label if available
        let label = match &self.config.format {
            NumberFormat::Auto => {
                if tick.label.is_empty() {
                    self.config.format.format(tick.value)
                } else {
                    tick.label.clone()
                }
            }
            _ => self.config.format.format(tick.value),
        };

        AxisTick {
            tick: tick.clone(),
            label,
            position: pos,
            tick_start,
            tick_end,
            label_position,
            grid_end,
            label_rotation: self.config.label_rotation.angle,
            text_anchor: self.config.effective_text_anchor(),
            is_minor,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scale::{LinearScale, ScaleExt};

    #[test]
    fn test_axis_orientation() {
        assert!(AxisOrientation::Bottom.is_horizontal());
        assert!(AxisOrientation::Top.is_horizontal());
        assert!(AxisOrientation::Left.is_vertical());
        assert!(AxisOrientation::Right.is_vertical());
    }

    #[test]
    fn test_axis_config_builder() {
        let config = AxisConfig::bottom()
            .with_tick_size(10.0)
            .with_tick_padding(5.0)
            .with_grid(200.0);

        assert_eq!(config.orientation, AxisOrientation::Bottom);
        assert_eq!(config.tick_size, 10.0);
        assert_eq!(config.tick_padding, 5.0);
        assert!(config.show_grid);
        assert_eq!(config.grid_length, 200.0);
    }

    #[test]
    fn test_axis_with_linear_scale() {
        let scale = LinearScale::new()
            .with_domain(0.0, 100.0)
            .with_range(0.0, 500.0);

        let mut axis = Axis::with_config(AxisConfig::bottom());
        axis.set_scale(&scale);

        assert!(!axis.ticks().is_empty());
        assert_eq!(axis.range, (0.0, 500.0));
    }

    #[test]
    fn test_axis_layout_bottom() {
        let scale = LinearScale::new()
            .with_domain(0.0, 100.0)
            .with_range(0.0, 500.0);

        let mut axis = Axis::with_config(AxisConfig::bottom().with_tick_size(6.0));
        axis.set_scale(&scale);

        let layout = axis.compute_layout(300.0);

        assert_eq!(layout.orientation, AxisOrientation::Bottom);
        assert_eq!(layout.domain_start, (0.0, 300.0));
        assert_eq!(layout.domain_end, (500.0, 300.0));
        assert!(!layout.ticks.is_empty());

        // Check first tick layout
        let first_tick = &layout.ticks[0];
        assert_eq!(first_tick.tick_start.1, 300.0);
        assert_eq!(first_tick.tick_end.1, 306.0); // 300 + tick_size
    }

    #[test]
    fn test_axis_layout_left() {
        let scale = LinearScale::new()
            .with_domain(0.0, 100.0)
            .with_range(500.0, 0.0); // Inverted for y-axis

        let mut axis = Axis::with_config(AxisConfig::left().with_tick_size(6.0));
        axis.set_scale(&scale);

        let layout = axis.compute_layout(50.0);

        assert_eq!(layout.orientation, AxisOrientation::Left);
        assert_eq!(layout.domain_start, (50.0, 500.0));
        assert_eq!(layout.domain_end, (50.0, 0.0));
    }

    #[test]
    fn test_axis_with_grid() {
        let scale = LinearScale::new()
            .with_domain(0.0, 100.0)
            .with_range(0.0, 500.0);

        let mut axis = Axis::with_config(AxisConfig::bottom().with_grid(200.0));
        axis.set_scale(&scale);

        let layout = axis.compute_layout(300.0);

        // All ticks should have grid end points
        for tick in &layout.ticks {
            assert!(tick.grid_end.is_some());
            let (_, y) = tick.grid_end.unwrap();
            assert_eq!(y, 100.0); // 300 - 200 (grid extends upward)
        }
    }

    #[test]
    fn test_axis_custom_format() {
        let scale = LinearScale::new()
            .with_domain(0.0, 1.0)
            .with_range(0.0, 100.0);

        let mut axis = Axis::with_config(AxisConfig::bottom().with_format(NumberFormat::Percent));
        axis.set_scale(&scale);

        let layout = axis.compute_layout(0.0);

        // Labels should be formatted as percentages
        let has_percent = layout.ticks.iter().any(|t| t.label.contains('%'));
        assert!(has_percent);
    }

    #[test]
    fn test_set_ticks_directly() {
        let mut axis = Axis::new();
        axis.set_range((0.0, 100.0));
        axis.set_ticks(vec![
            Tick::new(0.0, "0").with_position(0.0),
            Tick::new(50.0, "50").with_position(50.0),
            Tick::new(100.0, "100").with_position(100.0),
        ]);

        assert_eq!(axis.ticks().len(), 3);
    }

    // New tests for enhanced features

    #[test]
    fn test_label_rotation() {
        let rotation = LabelRotation::diagonal();
        assert!(rotation.is_rotated());
        assert_eq!(rotation.angle, -45.0);
        assert_eq!(rotation.anchor, Some(TextAnchor::End));
    }

    #[test]
    fn test_label_rotation_vertical() {
        let rotation = LabelRotation::vertical();
        assert_eq!(rotation.angle, -90.0);
        assert!((rotation.radians() - (-std::f64::consts::PI / 2.0)).abs() < 0.0001);
    }

    #[test]
    fn test_axis_config_with_rotation() {
        let config = AxisConfig::bottom()
            .with_diagonal_labels()
            .with_tick_size(6.0);

        assert_eq!(config.label_rotation.angle, -45.0);
        assert_eq!(config.effective_text_anchor(), TextAnchor::End);
    }

    #[test]
    fn test_axis_layout_includes_rotation() {
        let scale = LinearScale::new()
            .with_domain(0.0, 100.0)
            .with_range(0.0, 500.0);

        let mut axis = Axis::with_config(
            AxisConfig::bottom().with_label_rotation(LabelRotation::degrees(-30.0)),
        );
        axis.set_scale(&scale);

        let layout = axis.compute_layout(300.0);
        assert_eq!(layout.label_rotation, -30.0);
    }

    #[test]
    fn test_text_anchor_defaults() {
        assert_eq!(
            AxisOrientation::Bottom.default_text_anchor(),
            TextAnchor::Middle
        );
        assert_eq!(AxisOrientation::Left.default_text_anchor(), TextAnchor::End);
        assert_eq!(
            AxisOrientation::Right.default_text_anchor(),
            TextAnchor::Start
        );
    }

    #[test]
    fn test_band_offset() {
        let mut axis = Axis::with_config(AxisConfig::bottom().with_band_offset(25.0));
        axis.set_range((0.0, 200.0));
        axis.set_ticks(vec![
            Tick::new(0.0, "A").with_position(0.0),
            Tick::new(1.0, "B").with_position(50.0),
        ]);

        let layout = axis.compute_layout(100.0);

        // Tick positions should be offset by 25
        assert_eq!(layout.ticks[0].position, 25.0);
        assert_eq!(layout.ticks[1].position, 75.0);
    }

    #[test]
    fn test_axis_with_band_scale() {
        let scale = BandScale::new()
            .domain(vec!["A", "B", "C"])
            .range(0.0, 300.0);

        let mut axis = Axis::with_config(AxisConfig::bottom());
        axis.set_band_scale(&scale);

        assert!(axis.bandwidth() > 0.0);
        assert_eq!(axis.range, (0.0, 300.0));
    }

    #[test]
    fn test_axis_tick_labels_for_discrete_scale() {
        let mut axis = Axis::new();
        axis.set_range((0.0, 300.0));
        // Discrete scale ticks have labels set
        axis.set_ticks(vec![
            Tick::new(0.0, "January").with_position(50.0),
            Tick::new(1.0, "February").with_position(150.0),
            Tick::new(2.0, "March").with_position(250.0),
        ]);

        let layout = axis.compute_layout(100.0);

        // Labels should come from tick.label, not formatted value
        assert_eq!(layout.ticks[0].label, "January");
        assert_eq!(layout.ticks[1].label, "February");
        assert_eq!(layout.ticks[2].label, "March");
    }

    #[test]
    fn test_grid_config_integration() {
        let grid_config = GridConfig::light_dashed();

        let config = AxisConfig::bottom().with_grid_config(grid_config);

        assert!(config.show_grid);
        assert!(config.grid_config.is_enabled());
    }
}

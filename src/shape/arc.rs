//! Arc generator for pie and donut charts
//!
//! Creates arc paths for radial visualizations including pie charts,
//! donut charts, and gauge visualizations.

use super::path::{PathSegment, Point};
use std::f64::consts::{PI, TAU};

/// Arc generator for radial visualizations
///
/// Generates arc paths defined by inner/outer radius and start/end angles.
///
/// # Example
/// ```
/// use makepad_d3::shape::ArcGenerator;
/// use std::f64::consts::PI;
///
/// let arc = ArcGenerator::new()
///     .inner_radius(50.0)
///     .outer_radius(100.0)
///     .start_angle(0.0)
///     .end_angle(PI / 2.0);
///
/// let path = arc.generate();
/// ```
#[derive(Clone, Debug)]
pub struct ArcGenerator {
    /// Inner radius (0 for pie, >0 for donut)
    inner_radius: f64,
    /// Outer radius
    outer_radius: f64,
    /// Start angle in radians (0 = 12 o'clock, clockwise)
    start_angle: f64,
    /// End angle in radians
    end_angle: f64,
    /// Corner radius for rounded corners
    corner_radius: f64,
    /// Padding angle between adjacent arcs
    pad_angle: f64,
}

impl Default for ArcGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl ArcGenerator {
    /// Create a new arc generator with default settings
    pub fn new() -> Self {
        Self {
            inner_radius: 0.0,
            outer_radius: 100.0,
            start_angle: 0.0,
            end_angle: TAU,
            corner_radius: 0.0,
            pad_angle: 0.0,
        }
    }

    /// Set the inner radius
    pub fn inner_radius(mut self, radius: f64) -> Self {
        self.inner_radius = radius.max(0.0);
        self
    }

    /// Set the outer radius
    pub fn outer_radius(mut self, radius: f64) -> Self {
        self.outer_radius = radius.max(0.0);
        self
    }

    /// Set the start angle in radians
    pub fn start_angle(mut self, angle: f64) -> Self {
        self.start_angle = angle;
        self
    }

    /// Set the end angle in radians
    pub fn end_angle(mut self, angle: f64) -> Self {
        self.end_angle = angle;
        self
    }

    /// Set the corner radius
    pub fn corner_radius(mut self, radius: f64) -> Self {
        self.corner_radius = radius.max(0.0);
        self
    }

    /// Set the pad angle (gap between adjacent arcs)
    pub fn pad_angle(mut self, angle: f64) -> Self {
        self.pad_angle = angle.max(0.0);
        self
    }

    /// Get the inner radius
    pub fn get_inner_radius(&self) -> f64 {
        self.inner_radius
    }

    /// Get the outer radius
    pub fn get_outer_radius(&self) -> f64 {
        self.outer_radius
    }

    /// Calculate the centroid of the arc
    ///
    /// Returns the center point of the arc, useful for label placement.
    pub fn centroid(&self) -> Point {
        let r = (self.inner_radius + self.outer_radius) / 2.0;
        let a = (self.start_angle + self.end_angle) / 2.0 - PI / 2.0;
        Point::new(r * a.cos(), r * a.sin())
    }

    /// Get a point on the arc at a given angle and radius
    fn point_at(&self, angle: f64, radius: f64) -> Point {
        // Adjust angle so 0 is at 12 o'clock and increases clockwise
        let adjusted = angle - PI / 2.0;
        Point::new(radius * adjusted.cos(), radius * adjusted.sin())
    }

    /// Generate path segments for the arc
    pub fn generate(&self) -> Vec<PathSegment> {
        let mut path = Vec::new();

        let inner = self.inner_radius;
        let outer = self.outer_radius;
        let mut start = self.start_angle;
        let mut end = self.end_angle;

        // Apply padding
        if self.pad_angle > 0.0 {
            let pad = self.pad_angle / 2.0;
            start += pad;
            end -= pad;
        }

        // Ensure end > start
        let sweep = end - start;
        if sweep.abs() < 1e-10 {
            return path;
        }

        // Check if it's a full circle
        let is_full = sweep.abs() >= TAU - 1e-10;

        if is_full {
            // Full circle - need two arcs
            self.generate_full_circle(&mut path, inner, outer);
        } else if inner < 1e-10 {
            // Pie slice (no inner radius)
            self.generate_pie_slice(&mut path, outer, start, end);
        } else {
            // Donut slice
            self.generate_donut_slice(&mut path, inner, outer, start, end);
        }

        path
    }

    /// Generate a full circle (or annulus)
    fn generate_full_circle(&self, path: &mut Vec<PathSegment>, inner: f64, outer: f64) {
        // Outer circle (two semicircles)
        let start_outer = self.point_at(0.0, outer);
        path.push(PathSegment::MoveTo(start_outer));

        // First semicircle
        path.push(PathSegment::ArcTo {
            center: Point::zero(),
            radius: outer,
            start_angle: -PI / 2.0,
            end_angle: PI / 2.0,
            counterclockwise: false,
        });

        // Second semicircle
        path.push(PathSegment::ArcTo {
            center: Point::zero(),
            radius: outer,
            start_angle: PI / 2.0,
            end_angle: -PI / 2.0,
            counterclockwise: false,
        });

        if inner > 1e-10 {
            // Inner circle (for annulus)
            let start_inner = self.point_at(0.0, inner);
            path.push(PathSegment::MoveTo(start_inner));

            // Draw inner circle counterclockwise
            path.push(PathSegment::ArcTo {
                center: Point::zero(),
                radius: inner,
                start_angle: -PI / 2.0,
                end_angle: PI / 2.0,
                counterclockwise: true,
            });

            path.push(PathSegment::ArcTo {
                center: Point::zero(),
                radius: inner,
                start_angle: PI / 2.0,
                end_angle: -PI / 2.0,
                counterclockwise: true,
            });
        }

        path.push(PathSegment::ClosePath);
    }

    /// Generate a pie slice (no inner radius)
    fn generate_pie_slice(&self, path: &mut Vec<PathSegment>, outer: f64, start: f64, end: f64) {
        // Start at center
        path.push(PathSegment::MoveTo(Point::zero()));

        // Line to outer arc start
        let outer_start = self.point_at(start, outer);
        path.push(PathSegment::LineTo(outer_start));

        // Arc along outer edge
        let adjusted_start = start - PI / 2.0;
        let adjusted_end = end - PI / 2.0;

        path.push(PathSegment::ArcTo {
            center: Point::zero(),
            radius: outer,
            start_angle: adjusted_start,
            end_angle: adjusted_end,
            counterclockwise: false,
        });

        // Close back to center
        path.push(PathSegment::ClosePath);
    }

    /// Generate a donut slice
    fn generate_donut_slice(
        &self,
        path: &mut Vec<PathSegment>,
        inner: f64,
        outer: f64,
        start: f64,
        end: f64,
    ) {
        // Start at outer arc start
        let outer_start = self.point_at(start, outer);
        path.push(PathSegment::MoveTo(outer_start));

        // Arc along outer edge
        let adjusted_start = start - PI / 2.0;
        let adjusted_end = end - PI / 2.0;

        path.push(PathSegment::ArcTo {
            center: Point::zero(),
            radius: outer,
            start_angle: adjusted_start,
            end_angle: adjusted_end,
            counterclockwise: false,
        });

        // Line to inner arc end
        let inner_end = self.point_at(end, inner);
        path.push(PathSegment::LineTo(inner_end));

        // Arc along inner edge (reverse direction)
        path.push(PathSegment::ArcTo {
            center: Point::zero(),
            radius: inner,
            start_angle: adjusted_end,
            end_angle: adjusted_start,
            counterclockwise: true,
        });

        // Close back to start
        path.push(PathSegment::ClosePath);
    }
}

/// Builder for creating arcs from pie slice data
#[derive(Clone, Debug)]
pub struct ArcDatum {
    /// Inner radius
    pub inner_radius: f64,
    /// Outer radius
    pub outer_radius: f64,
    /// Start angle
    pub start_angle: f64,
    /// End angle
    pub end_angle: f64,
    /// Corner radius
    pub corner_radius: f64,
    /// Pad angle
    pub pad_angle: f64,
}

impl ArcDatum {
    /// Create a new arc datum
    pub fn new(start_angle: f64, end_angle: f64) -> Self {
        Self {
            inner_radius: 0.0,
            outer_radius: 100.0,
            start_angle,
            end_angle,
            corner_radius: 0.0,
            pad_angle: 0.0,
        }
    }

    /// Convert to an ArcGenerator
    pub fn to_generator(&self) -> ArcGenerator {
        ArcGenerator::new()
            .inner_radius(self.inner_radius)
            .outer_radius(self.outer_radius)
            .start_angle(self.start_angle)
            .end_angle(self.end_angle)
            .corner_radius(self.corner_radius)
            .pad_angle(self.pad_angle)
    }

    /// Generate path segments
    pub fn generate(&self) -> Vec<PathSegment> {
        self.to_generator().generate()
    }

    /// Calculate centroid
    pub fn centroid(&self) -> Point {
        self.to_generator().centroid()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arc_generator_basic() {
        let arc = ArcGenerator::new()
            .outer_radius(100.0)
            .start_angle(0.0)
            .end_angle(PI / 2.0);

        let path = arc.generate();
        assert!(!path.is_empty());
    }

    #[test]
    fn test_arc_generator_donut() {
        let arc = ArcGenerator::new()
            .inner_radius(50.0)
            .outer_radius(100.0)
            .start_angle(0.0)
            .end_angle(PI);

        let path = arc.generate();
        assert!(!path.is_empty());
    }

    #[test]
    fn test_arc_generator_full_circle() {
        let arc = ArcGenerator::new()
            .outer_radius(100.0)
            .start_angle(0.0)
            .end_angle(TAU);

        let path = arc.generate();
        assert!(!path.is_empty());
    }

    #[test]
    fn test_arc_centroid() {
        let arc = ArcGenerator::new()
            .outer_radius(100.0)
            .start_angle(0.0)
            .end_angle(PI);

        let centroid = arc.centroid();
        // Mid radius = 50, mid angle = PI/2 (adjusted for 12 o'clock start)
        // Centroid should be at 3 o'clock position
        assert!(centroid.x > 0.0); // Should be to the right
        assert!(centroid.y.abs() < 1e-10); // Should be on horizontal axis
    }

    #[test]
    fn test_arc_with_padding() {
        let arc = ArcGenerator::new()
            .outer_radius(100.0)
            .start_angle(0.0)
            .end_angle(PI / 2.0)
            .pad_angle(0.1);

        let path = arc.generate();
        assert!(!path.is_empty());
    }

    #[test]
    fn test_arc_datum() {
        let datum = ArcDatum::new(0.0, PI);
        let path = datum.generate();
        assert!(!path.is_empty());
    }
}

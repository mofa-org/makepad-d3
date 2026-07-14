//! Curve interpolation algorithms
//!
//! This module provides various curve interpolation strategies for connecting
//! data points in line and area charts.
//!
//! # Available Curves
//!
//! - [`LinearCurve`]: Straight line segments between points
//! - [`StepCurve`]: Step function with configurable position
//! - [`BasisCurve`]: B-spline interpolation (smooth, doesn't pass through points)
//! - [`CardinalCurve`]: Cardinal spline with tension parameter
//! - [`CatmullRomCurve`]: Catmull-Rom spline (passes through all points)
//! - [`MonotoneCurve`]: Monotone cubic interpolation (preserves monotonicity)
//! - [`NaturalCurve`]: Natural cubic spline (C2 continuous)

mod basis;
mod cardinal;
mod catmull_rom;
mod linear;
mod monotone;
mod natural;
mod step;

pub use basis::BasisCurve;
pub use cardinal::CardinalCurve;
pub use catmull_rom::CatmullRomCurve;
pub use linear::LinearCurve;
pub use monotone::MonotoneCurve;
pub use natural::NaturalCurve;
pub use step::{StepCurve, StepPosition};

use super::path::{PathSegment, Point};

/// Trait for curve interpolation algorithms
pub trait Curve: Send + Sync {
    /// Generate path segments from a sequence of points
    fn generate(&self, points: &[Point]) -> Vec<PathSegment>;

    /// Get the curve type name
    fn curve_type(&self) -> &'static str;
}

/// Box a curve for dynamic dispatch
pub fn box_curve(curve: impl Curve + 'static) -> Box<dyn Curve> {
    Box::new(curve)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_curve() {
        let curve = LinearCurve;
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(50.0, 50.0),
            Point::new(100.0, 0.0),
        ];

        let path = curve.generate(&points);
        assert_eq!(path.len(), 3); // MoveTo + 2 LineTo
    }

    #[test]
    fn test_step_curve() {
        let curve = StepCurve::after();
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(50.0, 50.0),
            Point::new(100.0, 0.0),
        ];

        let path = curve.generate(&points);
        assert!(!path.is_empty());
    }

    #[test]
    fn test_empty_points() {
        let curve = LinearCurve;
        let path = curve.generate(&[]);
        assert!(path.is_empty());
    }

    #[test]
    fn test_single_point() {
        let curve = LinearCurve;
        let points = vec![Point::new(50.0, 50.0)];
        let path = curve.generate(&points);
        assert_eq!(path.len(), 1); // Just MoveTo
    }
}

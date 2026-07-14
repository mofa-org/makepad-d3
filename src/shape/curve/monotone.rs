//! Monotone cubic curve interpolation

use super::{Curve, PathSegment, Point};

/// Monotone cubic interpolation curve
///
/// Creates a smooth curve that preserves the monotonicity of the data.
/// If the data is monotonically increasing or decreasing in Y, the curve
/// will not have overshoots or oscillations.
///
/// Uses the Fritsch-Carlson method for computing tangents.
///
/// # Example
/// ```
/// use makepad_d3::shape::curve::{Curve, MonotoneCurve};
/// use makepad_d3::shape::Point;
///
/// let curve = MonotoneCurve::new();
/// let points = vec![
///     Point::new(0.0, 0.0),
///     Point::new(50.0, 100.0),
///     Point::new(100.0, 200.0),
///     Point::new(150.0, 180.0),
/// ];
/// let path = curve.generate(&points);
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct MonotoneCurve;

impl MonotoneCurve {
    /// Create a new monotone curve
    pub fn new() -> Self {
        Self
    }

    /// Compute tangents using Fritsch-Carlson method
    fn compute_tangents(points: &[Point]) -> Vec<f64> {
        let n = points.len();
        if n < 2 {
            return vec![0.0; n];
        }

        // Calculate secants (slopes between consecutive points)
        let mut secants: Vec<f64> = Vec::with_capacity(n - 1);
        for i in 0..n - 1 {
            let dx = points[i + 1].x - points[i].x;
            if dx.abs() < 1e-10 {
                secants.push(0.0);
            } else {
                secants.push((points[i + 1].y - points[i].y) / dx);
            }
        }

        // Calculate tangents
        let mut tangents = vec![0.0; n];

        // First point
        tangents[0] = secants[0];

        // Interior points
        for i in 1..n - 1 {
            // If secants have different signs or either is zero, tangent is zero
            if secants[i - 1] * secants[i] <= 0.0 {
                tangents[i] = 0.0;
            } else {
                // Use harmonic mean for tangent
                tangents[i] = (secants[i - 1] + secants[i]) / 2.0;

                // Apply Fritsch-Carlson modification to ensure monotonicity
                let alpha = tangents[i] / secants[i - 1];
                let beta = tangents[i] / secants[i];

                // Check if we need to adjust
                if alpha * alpha + beta * beta > 9.0 {
                    let tau = 3.0 / (alpha * alpha + beta * beta).sqrt();
                    tangents[i] = tau * tangents[i];
                }
            }
        }

        // Last point
        tangents[n - 1] = secants[n - 2];

        tangents
    }
}

impl Curve for MonotoneCurve {
    fn generate(&self, points: &[Point]) -> Vec<PathSegment> {
        if points.is_empty() {
            return vec![];
        }

        if points.len() == 1 {
            return vec![PathSegment::MoveTo(points[0])];
        }

        if points.len() == 2 {
            return vec![
                PathSegment::MoveTo(points[0]),
                PathSegment::LineTo(points[1]),
            ];
        }

        let tangents = Self::compute_tangents(points);
        let mut path = Vec::with_capacity(points.len());
        path.push(PathSegment::MoveTo(points[0]));

        for i in 0..points.len() - 1 {
            let p0 = points[i];
            let p1 = points[i + 1];
            let dx = p1.x - p0.x;
            let m0 = tangents[i];
            let m1 = tangents[i + 1];

            // Convert to cubic Bezier control points
            let cp1 = Point::new(p0.x + dx / 3.0, p0.y + m0 * dx / 3.0);

            let cp2 = Point::new(p1.x - dx / 3.0, p1.y - m1 * dx / 3.0);

            path.push(PathSegment::CurveTo { cp1, cp2, end: p1 });
        }

        path
    }

    fn curve_type(&self) -> &'static str {
        "monotone"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monotone_basic() {
        let curve = MonotoneCurve::new();
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(50.0, 100.0),
            Point::new(100.0, 150.0),
            Point::new(150.0, 200.0),
        ];

        let path = curve.generate(&points);
        assert_eq!(path.len(), 4); // MoveTo + 3 curves
    }

    #[test]
    fn test_monotone_preserves_monotonicity() {
        let curve = MonotoneCurve::new();
        // Monotonically increasing data
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(50.0, 50.0),
            Point::new(100.0, 100.0),
            Point::new(150.0, 150.0),
        ];

        let path = curve.generate(&points);

        // Verify the curve doesn't overshoot
        for segment in &path[1..] {
            if let PathSegment::CurveTo { cp1, cp2, end } = segment {
                // Control points should be within bounds
                assert!(cp1.y >= 0.0 && cp1.y <= 150.0);
                assert!(cp2.y >= 0.0 && cp2.y <= 150.0);
                assert!(end.y >= 0.0 && end.y <= 150.0);
            }
        }
    }

    #[test]
    fn test_monotone_two_points() {
        let curve = MonotoneCurve::new();
        let points = vec![Point::new(0.0, 0.0), Point::new(100.0, 100.0)];

        let path = curve.generate(&points);
        assert_eq!(path.len(), 2); // Falls back to linear
    }

    #[test]
    fn test_monotone_flat_segment() {
        let curve = MonotoneCurve::new();
        let points = vec![
            Point::new(0.0, 50.0),
            Point::new(50.0, 50.0),
            Point::new(100.0, 100.0),
        ];

        let path = curve.generate(&points);
        assert!(!path.is_empty());
    }
}

//! Basis spline (B-spline) curve interpolation

use super::{Curve, PathSegment, Point};

/// B-spline curve
///
/// Creates a smooth curve using B-spline interpolation. The curve does not
/// pass through the control points but creates a smooth approximation.
///
/// # Example
/// ```
/// use makepad_d3::shape::curve::{Curve, BasisCurve};
/// use makepad_d3::shape::Point;
///
/// let curve = BasisCurve::new();
/// let points = vec![
///     Point::new(0.0, 0.0),
///     Point::new(50.0, 100.0),
///     Point::new(100.0, 50.0),
///     Point::new(150.0, 100.0),
/// ];
/// let path = curve.generate(&points);
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct BasisCurve;

impl BasisCurve {
    /// Create a new basis curve
    pub fn new() -> Self {
        Self
    }
}

impl Curve for BasisCurve {
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

        let mut path = Vec::new();

        // B-spline basis functions for uniform cubic B-spline
        // D3 behavior: moveTo first point, then lineTo the blend point where B-spline starts
        let p0 = points[0];
        let p1 = points[1];

        // D3 first moves to the actual first control point
        path.push(PathSegment::MoveTo(p0));

        // Then draws a line to where the B-spline curve actually starts
        // This is the blend point: (5*p0 + p1) / 6
        let blend_start = Point::new((5.0 * p0.x + p1.x) / 6.0, (5.0 * p0.y + p1.y) / 6.0);
        path.push(PathSegment::LineTo(blend_start));

        // Generate curve segments
        for i in 1..points.len() - 1 {
            let p0 = points[i - 1];
            let p1 = points[i];
            let p2 = points[i + 1];

            // B-spline control points for this segment
            let cp1 = Point::new((2.0 * p0.x + p1.x) / 3.0, (2.0 * p0.y + p1.y) / 3.0);
            let cp2 = Point::new((p0.x + 2.0 * p1.x) / 3.0, (p0.y + 2.0 * p1.y) / 3.0);
            let end = Point::new(
                (p0.x + 4.0 * p1.x + p2.x) / 6.0,
                (p0.y + 4.0 * p1.y + p2.y) / 6.0,
            );

            path.push(PathSegment::CurveTo { cp1, cp2, end });
        }

        // End segment: final bezier curve
        let n = points.len();
        let p0 = points[n - 2];
        let p1 = points[n - 1];

        let cp1 = Point::new((2.0 * p0.x + p1.x) / 3.0, (2.0 * p0.y + p1.y) / 3.0);
        let cp2 = Point::new((p0.x + 2.0 * p1.x) / 3.0, (p0.y + 2.0 * p1.y) / 3.0);
        // End blend point: (p0 + 5*p1) / 6
        let end = Point::new((p0.x + 5.0 * p1.x) / 6.0, (p0.y + 5.0 * p1.y) / 6.0);

        path.push(PathSegment::CurveTo { cp1, cp2, end });

        // D3 adds a final lineTo to connect to the last control point
        path.push(PathSegment::LineTo(p1));

        path
    }

    fn curve_type(&self) -> &'static str {
        "basis"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-10
    }

    #[test]
    fn test_basis_basic() {
        let curve = BasisCurve::new();
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(50.0, 100.0),
            Point::new(100.0, 50.0),
            Point::new(150.0, 100.0),
        ];

        let path = curve.generate(&points);
        assert!(!path.is_empty());

        // First segment should be MoveTo to first control point (D3 behavior)
        match &path[0] {
            PathSegment::MoveTo(p) => {
                assert!(approx_eq(p.x, 0.0));
                assert!(approx_eq(p.y, 0.0));
            }
            _ => panic!("Expected MoveTo"),
        }

        // Second segment should be LineTo blend point
        match &path[1] {
            PathSegment::LineTo(p) => {
                // (5*0 + 50) / 6 = 8.333...
                assert!(approx_eq(p.x, 50.0 / 6.0));
                // (5*0 + 100) / 6 = 16.666...
                assert!(approx_eq(p.y, 100.0 / 6.0));
            }
            _ => panic!("Expected LineTo blend point"),
        }
    }

    #[test]
    fn test_basis_two_points() {
        let curve = BasisCurve::new();
        let points = vec![Point::new(0.0, 0.0), Point::new(100.0, 100.0)];

        let path = curve.generate(&points);
        assert_eq!(path.len(), 2); // Falls back to linear
    }

    #[test]
    fn test_basis_d3_formula_compliance() {
        // Test that the B-spline formulas match D3.js
        let curve = BasisCurve::new();
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(200.0, 0.0),
        ];

        let path = curve.generate(&points);

        // Structure: MoveTo, LineTo (blend start), CurveTo (loop i=1), CurveTo (end), LineTo (final)
        assert_eq!(path.len(), 5);

        // MoveTo first point
        match &path[0] {
            PathSegment::MoveTo(p) => {
                assert!(approx_eq(p.x, 0.0));
                assert!(approx_eq(p.y, 0.0));
            }
            _ => panic!("Expected MoveTo"),
        }

        // LineTo blend start: (5*0 + 100)/6 = 100/6
        match &path[1] {
            PathSegment::LineTo(p) => {
                assert!(approx_eq(p.x, 100.0 / 6.0));
                assert!(approx_eq(p.y, 100.0 / 6.0));
            }
            _ => panic!("Expected LineTo"),
        }

        // First CurveTo (loop iteration i=1)
        // p0=points[0]=(0,0), p1=points[1]=(100,100), p2=points[2]=(200,0)
        match &path[2] {
            PathSegment::CurveTo { cp1, cp2, end } => {
                // cp1 = (2*0 + 100) / 3 = 100/3
                assert!(approx_eq(cp1.x, 100.0 / 3.0));
                assert!(approx_eq(cp1.y, 100.0 / 3.0));
                // cp2 = (0 + 2*100) / 3 = 200/3
                assert!(approx_eq(cp2.x, 200.0 / 3.0));
                assert!(approx_eq(cp2.y, 200.0 / 3.0));
                // end = (0 + 4*100 + 200) / 6 = 600/6 = 100
                assert!(approx_eq(end.x, 100.0));
                assert!(approx_eq(end.y, (0.0 + 400.0 + 0.0) / 6.0)); // = 400/6
            }
            _ => panic!("Expected first CurveTo"),
        }

        // End CurveTo
        // p0=points[1]=(100,100), p1=points[2]=(200,0)
        match &path[3] {
            PathSegment::CurveTo { cp1, cp2, end } => {
                // cp1 = (2*100 + 200) / 3 = 400/3
                assert!(approx_eq(cp1.x, 400.0 / 3.0));
                // cp2 = (100 + 2*200) / 3 = 500/3
                assert!(approx_eq(cp2.x, 500.0 / 3.0));
                // end = (100 + 5*200) / 6 = 1100/6
                assert!(approx_eq(end.x, 1100.0 / 6.0));
                assert!(approx_eq(end.y, (100.0 + 0.0) / 6.0)); // = 100/6
            }
            _ => panic!("Expected end CurveTo"),
        }

        // Final LineTo last point
        match &path[4] {
            PathSegment::LineTo(p) => {
                assert!(approx_eq(p.x, 200.0));
                assert!(approx_eq(p.y, 0.0));
            }
            _ => panic!("Expected final LineTo"),
        }
    }

    #[test]
    fn test_basis_empty() {
        let curve = BasisCurve::new();
        let path = curve.generate(&[]);
        assert!(path.is_empty());
    }

    #[test]
    fn test_basis_single_point() {
        let curve = BasisCurve::new();
        let points = vec![Point::new(50.0, 50.0)];
        let path = curve.generate(&points);
        assert_eq!(path.len(), 1);
        match &path[0] {
            PathSegment::MoveTo(p) => {
                assert!(approx_eq(p.x, 50.0));
                assert!(approx_eq(p.y, 50.0));
            }
            _ => panic!("Expected MoveTo"),
        }
    }

    #[test]
    fn test_basis_curve_type() {
        let curve = BasisCurve::new();
        assert_eq!(curve.curve_type(), "basis");
    }
}

//! Linear curve interpolation

use super::{Curve, PathSegment, Point};

/// Linear interpolation - straight line segments between points
///
/// This is the simplest curve type, connecting points with straight lines.
///
/// # Example
/// ```
/// use makepad_d3::shape::curve::{Curve, LinearCurve};
/// use makepad_d3::shape::Point;
///
/// let curve = LinearCurve;
/// let points = vec![
///     Point::new(0.0, 0.0),
///     Point::new(50.0, 100.0),
///     Point::new(100.0, 50.0),
/// ];
/// let path = curve.generate(&points);
/// assert_eq!(path.len(), 3); // MoveTo + 2 LineTo
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct LinearCurve;

impl LinearCurve {
    /// Create a new linear curve
    pub fn new() -> Self {
        Self
    }
}

impl Curve for LinearCurve {
    fn generate(&self, points: &[Point]) -> Vec<PathSegment> {
        if points.is_empty() {
            return vec![];
        }

        let mut path = Vec::with_capacity(points.len());
        path.push(PathSegment::MoveTo(points[0]));

        for point in &points[1..] {
            path.push(PathSegment::LineTo(*point));
        }

        path
    }

    fn curve_type(&self) -> &'static str {
        "linear"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_basic() {
        let curve = LinearCurve::new();
        let points = vec![Point::new(0.0, 0.0), Point::new(100.0, 100.0)];

        let path = curve.generate(&points);
        assert_eq!(path.len(), 2);

        match &path[0] {
            PathSegment::MoveTo(p) => assert_eq!(*p, Point::new(0.0, 0.0)),
            _ => panic!("Expected MoveTo"),
        }

        match &path[1] {
            PathSegment::LineTo(p) => assert_eq!(*p, Point::new(100.0, 100.0)),
            _ => panic!("Expected LineTo"),
        }
    }

    #[test]
    fn test_linear_multiple_points() {
        let curve = LinearCurve;
        let points: Vec<Point> = (0..5)
            .map(|i| Point::new(i as f64 * 25.0, (i as f64 * 25.0).sin() * 50.0 + 50.0))
            .collect();

        let path = curve.generate(&points);
        assert_eq!(path.len(), 5);
    }
}

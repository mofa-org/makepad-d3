//! Cardinal spline curve interpolation

use super::{Curve, PathSegment, Point};

/// Cardinal spline curve
///
/// Creates a smooth curve that passes through all points. The tension
/// parameter controls how tight the curve is around the points.
///
/// - Tension 0: Catmull-Rom spline (passes through points smoothly)
/// - Tension 1: Straight line segments
///
/// # Example
/// ```
/// use makepad_d3::shape::curve::{Curve, CardinalCurve};
/// use makepad_d3::shape::Point;
///
/// let curve = CardinalCurve::new(0.5);
/// let points = vec![
///     Point::new(0.0, 0.0),
///     Point::new(50.0, 100.0),
///     Point::new(100.0, 50.0),
///     Point::new(150.0, 100.0),
/// ];
/// let path = curve.generate(&points);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct CardinalCurve {
    /// Tension parameter (0 to 1)
    pub tension: f64,
}

impl Default for CardinalCurve {
    fn default() -> Self {
        Self { tension: 0.0 }
    }
}

impl CardinalCurve {
    /// Create a new cardinal curve with given tension
    pub fn new(tension: f64) -> Self {
        Self {
            tension: tension.clamp(0.0, 1.0),
        }
    }

    /// Create a cardinal curve with zero tension (Catmull-Rom)
    pub fn catmull_rom() -> Self {
        Self::new(0.0)
    }
}

impl Curve for CardinalCurve {
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

        let mut path = Vec::with_capacity(points.len());
        path.push(PathSegment::MoveTo(points[0]));

        // Scale factor based on tension
        let k = (1.0 - self.tension) / 6.0;

        for i in 1..points.len() {
            let p0 = if i >= 2 { points[i - 2] } else { points[0] };
            let p1 = points[i - 1];
            let p2 = points[i];
            let p3 = if i + 1 < points.len() {
                points[i + 1]
            } else {
                points[points.len() - 1]
            };

            // Calculate control points using cardinal spline formula
            let cp1 = Point::new(p1.x + k * (p2.x - p0.x), p1.y + k * (p2.y - p0.y));

            let cp2 = Point::new(p2.x - k * (p3.x - p1.x), p2.y - k * (p3.y - p1.y));

            path.push(PathSegment::CurveTo { cp1, cp2, end: p2 });
        }

        path
    }

    fn curve_type(&self) -> &'static str {
        "cardinal"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cardinal_basic() {
        let curve = CardinalCurve::new(0.5);
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(50.0, 100.0),
            Point::new(100.0, 50.0),
            Point::new(150.0, 100.0),
        ];

        let path = curve.generate(&points);
        assert_eq!(path.len(), 4); // MoveTo + 3 curves

        // Curve passes through all points
        match &path[0] {
            PathSegment::MoveTo(p) => assert_eq!(*p, points[0]),
            _ => panic!("Expected MoveTo"),
        }
    }

    #[test]
    fn test_cardinal_tension_1() {
        // With tension 1, should be more like straight lines
        let curve = CardinalCurve::new(1.0);
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(50.0, 100.0),
            Point::new(100.0, 50.0),
        ];

        let path = curve.generate(&points);
        assert!(!path.is_empty());
    }

    #[test]
    fn test_cardinal_two_points() {
        let curve = CardinalCurve::new(0.5);
        let points = vec![Point::new(0.0, 0.0), Point::new(100.0, 100.0)];

        let path = curve.generate(&points);
        assert_eq!(path.len(), 2); // Falls back to linear
    }
}

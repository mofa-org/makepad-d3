//! Catmull-Rom spline curve interpolation

use super::{Curve, PathSegment, Point};

/// Catmull-Rom spline curve
///
/// Creates a smooth curve that passes through all control points.
/// The alpha parameter controls the type of Catmull-Rom spline:
///
/// - Alpha 0.0: Uniform Catmull-Rom
/// - Alpha 0.5: Centripetal Catmull-Rom (recommended, avoids cusps)
/// - Alpha 1.0: Chordal Catmull-Rom
///
/// # Example
/// ```
/// use makepad_d3::shape::curve::{Curve, CatmullRomCurve};
/// use makepad_d3::shape::Point;
///
/// // Centripetal Catmull-Rom (recommended)
/// let curve = CatmullRomCurve::centripetal();
/// let points = vec![
///     Point::new(0.0, 0.0),
///     Point::new(50.0, 100.0),
///     Point::new(100.0, 50.0),
///     Point::new(150.0, 100.0),
/// ];
/// let path = curve.generate(&points);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct CatmullRomCurve {
    /// Alpha parameter (0.0 = uniform, 0.5 = centripetal, 1.0 = chordal)
    pub alpha: f64,
}

impl Default for CatmullRomCurve {
    fn default() -> Self {
        Self { alpha: 0.5 } // Centripetal by default
    }
}

impl CatmullRomCurve {
    /// Create a new Catmull-Rom curve with given alpha
    pub fn new(alpha: f64) -> Self {
        Self {
            alpha: alpha.clamp(0.0, 1.0),
        }
    }

    /// Create a uniform Catmull-Rom curve (alpha = 0)
    pub fn uniform() -> Self {
        Self::new(0.0)
    }

    /// Create a centripetal Catmull-Rom curve (alpha = 0.5)
    pub fn centripetal() -> Self {
        Self::new(0.5)
    }

    /// Create a chordal Catmull-Rom curve (alpha = 1)
    pub fn chordal() -> Self {
        Self::new(1.0)
    }

    /// Calculate the parameter t for Catmull-Rom
    fn get_t(&self, t: f64, p0: Point, p1: Point) -> f64 {
        let dx = p1.x - p0.x;
        let dy = p1.y - p0.y;
        let dist = (dx * dx + dy * dy).powf(self.alpha * 0.5);
        t + dist
    }

    /// Interpolate a point on the Catmull-Rom curve
    fn interpolate(&self, p0: Point, p1: Point, p2: Point, p3: Point, t: f64) -> Point {
        // Calculate t values
        let t0 = 0.0;
        let t1 = self.get_t(t0, p0, p1);
        let t2 = self.get_t(t1, p1, p2);
        let t3 = self.get_t(t2, p2, p3);

        // Map t from [0, 1] to [t1, t2]
        let t = t1 + t * (t2 - t1);

        // Avoid division by zero
        let epsilon = 1e-10;

        // Calculate intermediate points
        let a1 = if (t1 - t0).abs() > epsilon {
            Point::new(
                (t1 - t) / (t1 - t0) * p0.x + (t - t0) / (t1 - t0) * p1.x,
                (t1 - t) / (t1 - t0) * p0.y + (t - t0) / (t1 - t0) * p1.y,
            )
        } else {
            p0.lerp(&p1, 0.5)
        };

        let a2 = if (t2 - t1).abs() > epsilon {
            Point::new(
                (t2 - t) / (t2 - t1) * p1.x + (t - t1) / (t2 - t1) * p2.x,
                (t2 - t) / (t2 - t1) * p1.y + (t - t1) / (t2 - t1) * p2.y,
            )
        } else {
            p1.lerp(&p2, 0.5)
        };

        let a3 = if (t3 - t2).abs() > epsilon {
            Point::new(
                (t3 - t) / (t3 - t2) * p2.x + (t - t2) / (t3 - t2) * p3.x,
                (t3 - t) / (t3 - t2) * p2.y + (t - t2) / (t3 - t2) * p3.y,
            )
        } else {
            p2.lerp(&p3, 0.5)
        };

        let b1 = if (t2 - t0).abs() > epsilon {
            Point::new(
                (t2 - t) / (t2 - t0) * a1.x + (t - t0) / (t2 - t0) * a2.x,
                (t2 - t) / (t2 - t0) * a1.y + (t - t0) / (t2 - t0) * a2.y,
            )
        } else {
            a1.lerp(&a2, 0.5)
        };

        let b2 = if (t3 - t1).abs() > epsilon {
            Point::new(
                (t3 - t) / (t3 - t1) * a2.x + (t - t1) / (t3 - t1) * a3.x,
                (t3 - t) / (t3 - t1) * a2.y + (t - t1) / (t3 - t1) * a3.y,
            )
        } else {
            a2.lerp(&a3, 0.5)
        };

        if (t2 - t1).abs() > epsilon {
            Point::new(
                (t2 - t) / (t2 - t1) * b1.x + (t - t1) / (t2 - t1) * b2.x,
                (t2 - t) / (t2 - t1) * b1.y + (t - t1) / (t2 - t1) * b2.y,
            )
        } else {
            b1.lerp(&b2, 0.5)
        }
    }

    /// Convert Catmull-Rom segment to cubic Bezier
    fn to_bezier(&self, p0: Point, p1: Point, p2: Point, p3: Point) -> (Point, Point) {
        // Use differentiation to find the control points
        // For uniform Catmull-Rom, this is straightforward
        if self.alpha == 0.0 {
            let cp1 = Point::new(p1.x + (p2.x - p0.x) / 6.0, p1.y + (p2.y - p0.y) / 6.0);
            let cp2 = Point::new(p2.x - (p3.x - p1.x) / 6.0, p2.y - (p3.y - p1.y) / 6.0);
            return (cp1, cp2);
        }

        // For non-uniform, sample the curve and fit Bezier
        let q1 = self.interpolate(p0, p1, p2, p3, 1.0 / 3.0);
        let q2 = self.interpolate(p0, p1, p2, p3, 2.0 / 3.0);

        // Derive control points from sampled points
        let cp1 = Point::new(
            (-5.0 * p1.x + 18.0 * q1.x - 9.0 * q2.x + 2.0 * p2.x) / 6.0,
            (-5.0 * p1.y + 18.0 * q1.y - 9.0 * q2.y + 2.0 * p2.y) / 6.0,
        );
        let cp2 = Point::new(
            (2.0 * p1.x - 9.0 * q1.x + 18.0 * q2.x - 5.0 * p2.x) / 6.0,
            (2.0 * p1.y - 9.0 * q1.y + 18.0 * q2.y - 5.0 * p2.y) / 6.0,
        );

        (cp1, cp2)
    }
}

impl Curve for CatmullRomCurve {
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

        for i in 0..points.len() - 1 {
            // Get the four control points for this segment
            let p0 = if i > 0 { points[i - 1] } else { points[0] };
            let p1 = points[i];
            let p2 = points[i + 1];
            let p3 = if i + 2 < points.len() {
                points[i + 2]
            } else {
                points[points.len() - 1]
            };

            let (cp1, cp2) = self.to_bezier(p0, p1, p2, p3);

            path.push(PathSegment::CurveTo { cp1, cp2, end: p2 });
        }

        path
    }

    fn curve_type(&self) -> &'static str {
        "catmull-rom"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catmull_rom_basic() {
        let curve = CatmullRomCurve::centripetal();
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

        // Last curve ends at last point
        match &path[3] {
            PathSegment::CurveTo { end, .. } => assert_eq!(*end, points[3]),
            _ => panic!("Expected CurveTo"),
        }
    }

    #[test]
    fn test_catmull_rom_uniform() {
        let curve = CatmullRomCurve::uniform();
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(50.0, 100.0),
            Point::new(100.0, 50.0),
        ];

        let path = curve.generate(&points);
        assert!(!path.is_empty());
    }

    #[test]
    fn test_catmull_rom_two_points() {
        let curve = CatmullRomCurve::centripetal();
        let points = vec![Point::new(0.0, 0.0), Point::new(100.0, 100.0)];

        let path = curve.generate(&points);
        assert_eq!(path.len(), 2); // Falls back to linear
    }
}

//! Natural cubic spline curve interpolation

use super::{Curve, PathSegment, Point};

/// Natural cubic spline curve
///
/// Creates a C2 continuous curve (continuous second derivative) that passes
/// through all points. Uses natural boundary conditions (second derivative
/// is zero at endpoints).
///
/// # Example
/// ```
/// use makepad_d3::shape::curve::{Curve, NaturalCurve};
/// use makepad_d3::shape::Point;
///
/// let curve = NaturalCurve::new();
/// let points = vec![
///     Point::new(0.0, 0.0),
///     Point::new(50.0, 100.0),
///     Point::new(100.0, 50.0),
///     Point::new(150.0, 100.0),
/// ];
/// let path = curve.generate(&points);
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct NaturalCurve;

impl NaturalCurve {
    /// Create a new natural curve
    pub fn new() -> Self {
        Self
    }

    /// Solve tridiagonal system using Thomas algorithm
    fn solve_tridiagonal(n: usize, a: &[f64], b: &[f64], c: &[f64], d: &[f64]) -> Vec<f64> {
        if n == 0 {
            return vec![];
        }

        let mut c_prime = vec![0.0; n];
        let mut d_prime = vec![0.0; n];
        let mut x = vec![0.0; n];

        // Forward sweep
        c_prime[0] = c[0] / b[0];
        d_prime[0] = d[0] / b[0];

        for i in 1..n {
            let denom = b[i] - a[i] * c_prime[i - 1];
            if denom.abs() < 1e-10 {
                c_prime[i] = 0.0;
                d_prime[i] = 0.0;
            } else {
                c_prime[i] = c[i] / denom;
                d_prime[i] = (d[i] - a[i] * d_prime[i - 1]) / denom;
            }
        }

        // Back substitution
        x[n - 1] = d_prime[n - 1];
        for i in (0..n - 1).rev() {
            x[i] = d_prime[i] - c_prime[i] * x[i + 1];
        }

        x
    }

    /// Compute natural cubic spline coefficients
    fn compute_spline_coefficients(points: &[Point]) -> Vec<(f64, f64)> {
        let n = points.len();
        if n < 3 {
            return vec![(0.0, 0.0); n.saturating_sub(1)];
        }

        // Compute h values (distances between x coordinates)
        let h: Vec<f64> = (0..n - 1).map(|i| points[i + 1].x - points[i].x).collect();

        // Set up tridiagonal system for second derivatives
        let mut a = vec![0.0; n - 2];
        let mut b = vec![0.0; n - 2];
        let mut c = vec![0.0; n - 2];
        let mut d = vec![0.0; n - 2];

        for i in 0..n - 2 {
            a[i] = if i > 0 { h[i] } else { 0.0 };
            b[i] = 2.0 * (h[i] + h[i + 1]);
            c[i] = if i < n - 3 { h[i + 1] } else { 0.0 };

            let y0 = points[i].y;
            let y1 = points[i + 1].y;
            let y2 = points[i + 2].y;

            d[i] = 6.0 * ((y2 - y1) / h[i + 1] - (y1 - y0) / h[i]);
        }

        // Solve for interior second derivatives
        let m_interior = Self::solve_tridiagonal(n - 2, &a, &b, &c, &d);

        // Full second derivatives with natural boundary conditions (M[0] = M[n-1] = 0)
        let mut m = vec![0.0; n];
        for (i, &val) in m_interior.iter().enumerate() {
            m[i + 1] = val;
        }

        // Compute first derivatives at each point
        let mut tangents = Vec::with_capacity(n - 1);
        for i in 0..n - 1 {
            let dy = points[i + 1].y - points[i].y;
            let dx = h[i];
            let tangent = dy / dx - dx * (2.0 * m[i] + m[i + 1]) / 6.0;
            tangents.push((tangent, m[i]));
        }

        tangents
    }
}

impl Curve for NaturalCurve {
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

        let coeffs = Self::compute_spline_coefficients(points);
        let mut path = Vec::with_capacity(points.len());
        path.push(PathSegment::MoveTo(points[0]));

        for i in 0..points.len() - 1 {
            let p0 = points[i];
            let p1 = points[i + 1];
            let dx = p1.x - p0.x;

            let (m0, _second_deriv) = coeffs[i];

            // For the end tangent, use the next coefficient or compute from boundary
            let m1 = if i + 1 < coeffs.len() {
                let next_dy = points[i + 2].y - points[i + 1].y;
                let next_dx = points[i + 2].x - points[i + 1].x;
                next_dy / next_dx - next_dx * coeffs[i + 1].1 / 3.0
            } else {
                // Last segment: use derivative at end
                (p1.y - p0.y) / dx
            };

            // Convert to cubic Bezier control points
            let cp1 = Point::new(p0.x + dx / 3.0, p0.y + m0 * dx / 3.0);

            let cp2 = Point::new(p1.x - dx / 3.0, p1.y - m1 * dx / 3.0);

            path.push(PathSegment::CurveTo { cp1, cp2, end: p1 });
        }

        path
    }

    fn curve_type(&self) -> &'static str {
        "natural"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_natural_basic() {
        let curve = NaturalCurve::new();
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
    fn test_natural_two_points() {
        let curve = NaturalCurve::new();
        let points = vec![Point::new(0.0, 0.0), Point::new(100.0, 100.0)];

        let path = curve.generate(&points);
        assert_eq!(path.len(), 2); // Falls back to linear
    }

    #[test]
    fn test_natural_three_points() {
        let curve = NaturalCurve::new();
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(50.0, 100.0),
            Point::new(100.0, 0.0),
        ];

        let path = curve.generate(&points);
        assert_eq!(path.len(), 3); // MoveTo + 2 curves
    }
}

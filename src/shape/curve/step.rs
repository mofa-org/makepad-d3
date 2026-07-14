//! Step curve interpolation

use super::{Curve, PathSegment, Point};

/// Position of the step change
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StepPosition {
    /// Step occurs before the point (step-before)
    Before,
    /// Step occurs at the midpoint (step)
    #[default]
    Middle,
    /// Step occurs after the point (step-after)
    After,
}

/// Step function curve
///
/// Creates a step function between points. The step position determines
/// where the vertical transition occurs.
///
/// # Example
/// ```
/// use makepad_d3::shape::curve::{Curve, StepCurve, StepPosition};
/// use makepad_d3::shape::Point;
///
/// let curve = StepCurve::new(StepPosition::After);
/// let points = vec![
///     Point::new(0.0, 0.0),
///     Point::new(50.0, 100.0),
///     Point::new(100.0, 50.0),
/// ];
/// let path = curve.generate(&points);
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct StepCurve {
    /// Where the step transition occurs
    pub position: StepPosition,
}

impl StepCurve {
    /// Create a new step curve with given position
    pub fn new(position: StepPosition) -> Self {
        Self { position }
    }

    /// Create a step-before curve
    pub fn before() -> Self {
        Self::new(StepPosition::Before)
    }

    /// Create a step curve (middle)
    pub fn middle() -> Self {
        Self::new(StepPosition::Middle)
    }

    /// Create a step-after curve
    pub fn after() -> Self {
        Self::new(StepPosition::After)
    }
}

impl Curve for StepCurve {
    fn generate(&self, points: &[Point]) -> Vec<PathSegment> {
        if points.is_empty() {
            return vec![];
        }

        if points.len() == 1 {
            return vec![PathSegment::MoveTo(points[0])];
        }

        let mut path = Vec::with_capacity(points.len() * 2);
        path.push(PathSegment::MoveTo(points[0]));

        for i in 1..points.len() {
            let prev = points[i - 1];
            let curr = points[i];

            match self.position {
                StepPosition::Before => {
                    // Vertical first, then horizontal
                    path.push(PathSegment::LineTo(Point::new(prev.x, curr.y)));
                    path.push(PathSegment::LineTo(curr));
                }
                StepPosition::Middle => {
                    // Horizontal to midpoint, vertical, horizontal to end
                    let mid_x = (prev.x + curr.x) / 2.0;
                    path.push(PathSegment::LineTo(Point::new(mid_x, prev.y)));
                    path.push(PathSegment::LineTo(Point::new(mid_x, curr.y)));
                    path.push(PathSegment::LineTo(curr));
                }
                StepPosition::After => {
                    // Horizontal first, then vertical
                    path.push(PathSegment::LineTo(Point::new(curr.x, prev.y)));
                    path.push(PathSegment::LineTo(curr));
                }
            }
        }

        path
    }

    fn curve_type(&self) -> &'static str {
        match self.position {
            StepPosition::Before => "step-before",
            StepPosition::Middle => "step",
            StepPosition::After => "step-after",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_after() {
        let curve = StepCurve::after();
        let points = vec![Point::new(0.0, 0.0), Point::new(100.0, 100.0)];

        let path = curve.generate(&points);
        assert_eq!(path.len(), 3); // MoveTo, LineTo (horizontal), LineTo (vertical)

        // Check intermediate point
        match &path[1] {
            PathSegment::LineTo(p) => {
                assert_eq!(p.x, 100.0); // Same x as end
                assert_eq!(p.y, 0.0); // Same y as start
            }
            _ => panic!("Expected LineTo"),
        }
    }

    #[test]
    fn test_step_before() {
        let curve = StepCurve::before();
        let points = vec![Point::new(0.0, 0.0), Point::new(100.0, 100.0)];

        let path = curve.generate(&points);
        assert_eq!(path.len(), 3);

        // Check intermediate point
        match &path[1] {
            PathSegment::LineTo(p) => {
                assert_eq!(p.x, 0.0); // Same x as start
                assert_eq!(p.y, 100.0); // Same y as end
            }
            _ => panic!("Expected LineTo"),
        }
    }

    #[test]
    fn test_step_middle() {
        let curve = StepCurve::middle();
        let points = vec![Point::new(0.0, 0.0), Point::new(100.0, 100.0)];

        let path = curve.generate(&points);
        assert_eq!(path.len(), 4); // MoveTo + 3 LineTo for middle step

        // Check midpoint
        match &path[1] {
            PathSegment::LineTo(p) => {
                assert_eq!(p.x, 50.0); // Midpoint x
                assert_eq!(p.y, 0.0); // Start y
            }
            _ => panic!("Expected LineTo"),
        }
    }
}

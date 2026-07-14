//! Line generator for creating line paths from data
//!
//! Converts data points to drawable path segments using configurable
//! curve interpolation.

use super::curve::{Curve, LinearCurve};
use super::path::{PathSegment, Point};
use crate::data::DataPoint;

/// Line generator that converts data points to path segments
///
/// # Example
/// ```
/// use makepad_d3::data::DataPoint;
/// use makepad_d3::shape::{LineGenerator, Point};
/// use makepad_d3::shape::curve::LinearCurve;
///
/// let data = vec![
///     DataPoint::from((0.0, 100.0)),
///     DataPoint::from((50.0, 150.0)),
///     DataPoint::from((100.0, 120.0)),
/// ];
///
/// let line = LineGenerator::new();
/// let path = line.generate(&data);
/// assert_eq!(path.len(), 3);
/// ```
pub struct LineGenerator {
    /// Function to extract x coordinate
    x_fn: Box<dyn Fn(&DataPoint, usize) -> f64 + Send + Sync>,
    /// Function to extract y coordinate
    y_fn: Box<dyn Fn(&DataPoint, usize) -> f64 + Send + Sync>,
    /// Function to determine if a point is defined (included)
    defined_fn: Box<dyn Fn(&DataPoint, usize) -> bool + Send + Sync>,
    /// Curve interpolation
    curve: Box<dyn Curve>,
}

impl Default for LineGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl LineGenerator {
    /// Create a new line generator with default settings
    pub fn new() -> Self {
        Self {
            x_fn: Box::new(|d, i| d.x_or(i)),
            y_fn: Box::new(|d, _| d.y),
            defined_fn: Box::new(|d, _| d.y.is_finite()),
            curve: Box::new(LinearCurve),
        }
    }

    /// Set the x accessor function
    pub fn x<F>(mut self, f: F) -> Self
    where
        F: Fn(&DataPoint, usize) -> f64 + Send + Sync + 'static,
    {
        self.x_fn = Box::new(f);
        self
    }

    /// Set the y accessor function
    pub fn y<F>(mut self, f: F) -> Self
    where
        F: Fn(&DataPoint, usize) -> f64 + Send + Sync + 'static,
    {
        self.y_fn = Box::new(f);
        self
    }

    /// Set the defined predicate
    ///
    /// Points for which this function returns false will be excluded,
    /// creating gaps in the line.
    pub fn defined<F>(mut self, f: F) -> Self
    where
        F: Fn(&DataPoint, usize) -> bool + Send + Sync + 'static,
    {
        self.defined_fn = Box::new(f);
        self
    }

    /// Set the curve interpolation
    pub fn curve(mut self, curve: impl Curve + 'static) -> Self {
        self.curve = Box::new(curve);
        self
    }

    /// Generate path segments from data points
    pub fn generate(&self, data: &[DataPoint]) -> Vec<PathSegment> {
        // Collect defined points into segments
        let mut segments: Vec<Vec<Point>> = Vec::new();
        let mut current_segment: Vec<Point> = Vec::new();

        for (i, d) in data.iter().enumerate() {
            if (self.defined_fn)(d, i) {
                let x = (self.x_fn)(d, i);
                let y = (self.y_fn)(d, i);
                current_segment.push(Point::new(x, y));
            } else if !current_segment.is_empty() {
                // End current segment
                segments.push(std::mem::take(&mut current_segment));
            }
        }

        // Don't forget the last segment
        if !current_segment.is_empty() {
            segments.push(current_segment);
        }

        // Generate path for each segment
        let mut path = Vec::new();
        for segment in segments {
            path.extend(self.curve.generate(&segment));
        }

        path
    }

    /// Generate path segments from raw points
    pub fn generate_from_points(&self, points: &[Point]) -> Vec<PathSegment> {
        self.curve.generate(points)
    }
}

/// Convenience functions for creating line generators with specific curves
impl LineGenerator {
    /// Create a line generator with linear interpolation
    pub fn linear() -> Self {
        Self::new().curve(LinearCurve)
    }

    /// Create a line generator with step interpolation
    pub fn step() -> Self {
        use super::curve::StepCurve;
        Self::new().curve(StepCurve::middle())
    }

    /// Create a line generator with step-before interpolation
    pub fn step_before() -> Self {
        use super::curve::StepCurve;
        Self::new().curve(StepCurve::before())
    }

    /// Create a line generator with step-after interpolation
    pub fn step_after() -> Self {
        use super::curve::StepCurve;
        Self::new().curve(StepCurve::after())
    }

    /// Create a line generator with basis interpolation
    pub fn basis() -> Self {
        use super::curve::BasisCurve;
        Self::new().curve(BasisCurve::new())
    }

    /// Create a line generator with cardinal interpolation
    pub fn cardinal(tension: f64) -> Self {
        use super::curve::CardinalCurve;
        Self::new().curve(CardinalCurve::new(tension))
    }

    /// Create a line generator with Catmull-Rom interpolation
    pub fn catmull_rom() -> Self {
        use super::curve::CatmullRomCurve;
        Self::new().curve(CatmullRomCurve::centripetal())
    }

    /// Create a line generator with monotone interpolation
    pub fn monotone() -> Self {
        use super::curve::MonotoneCurve;
        Self::new().curve(MonotoneCurve::new())
    }

    /// Create a line generator with natural cubic spline
    pub fn natural() -> Self {
        use super::curve::NaturalCurve;
        Self::new().curve(NaturalCurve::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_data() -> Vec<DataPoint> {
        vec![
            DataPoint::from((0.0, 100.0)),
            DataPoint::from((50.0, 150.0)),
            DataPoint::from((100.0, 120.0)),
            DataPoint::from((150.0, 180.0)),
        ]
    }

    #[test]
    fn test_line_generator_basic() {
        let data = sample_data();
        let line = LineGenerator::new();
        let path = line.generate(&data);

        assert_eq!(path.len(), 4); // MoveTo + 3 LineTo
    }

    #[test]
    fn test_line_generator_with_gap() {
        let mut data = sample_data();
        data[1].y = f64::NAN; // Create a gap

        let line = LineGenerator::new();
        let path = line.generate(&data);

        // Should have two segments (before and after the gap)
        // Segment 1: MoveTo + nothing (single point)
        // Segment 2: MoveTo + 2 LineTo
        assert!(!path.is_empty());
    }

    #[test]
    fn test_line_generator_custom_x() {
        let data = sample_data();
        let line = LineGenerator::new().x(|_, i| i as f64 * 100.0);
        let path = line.generate(&data);

        // Check first point uses custom x
        match &path[0] {
            PathSegment::MoveTo(p) => assert_eq!(p.x, 0.0),
            _ => panic!("Expected MoveTo"),
        }
    }

    #[test]
    fn test_line_generator_curves() {
        let data = sample_data();

        // Test different curve types compile and work
        let _ = LineGenerator::linear().generate(&data);
        let _ = LineGenerator::step().generate(&data);
        let _ = LineGenerator::step_before().generate(&data);
        let _ = LineGenerator::step_after().generate(&data);
        let _ = LineGenerator::basis().generate(&data);
        let _ = LineGenerator::cardinal(0.5).generate(&data);
        let _ = LineGenerator::catmull_rom().generate(&data);
        let _ = LineGenerator::monotone().generate(&data);
        let _ = LineGenerator::natural().generate(&data);
    }

    #[test]
    fn test_line_generator_empty() {
        let line = LineGenerator::new();
        let path = line.generate(&[]);
        assert!(path.is_empty());
    }

    #[test]
    fn test_line_generator_single_point() {
        let data = vec![DataPoint::from((50.0, 100.0))];
        let line = LineGenerator::new();
        let path = line.generate(&data);
        assert_eq!(path.len(), 1); // Just MoveTo
    }
}

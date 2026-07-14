//! Area generator for creating filled regions between curves
//!
//! Creates closed path segments representing the area between a baseline
//! and a top line.

use super::curve::{Curve, LinearCurve};
use super::path::{PathSegment, Point};
use crate::data::DataPoint;

/// Area generator for filled regions
///
/// Generates a closed path representing the area between a baseline (y0)
/// and a top line (y1).
///
/// # Example
/// ```
/// use makepad_d3::data::DataPoint;
/// use makepad_d3::shape::AreaGenerator;
///
/// let data = vec![
///     DataPoint::from((0.0, 100.0)),
///     DataPoint::from((50.0, 150.0)),
///     DataPoint::from((100.0, 120.0)),
/// ];
///
/// let area = AreaGenerator::new();
/// let path = area.generate(&data);
/// ```
pub struct AreaGenerator {
    /// Function to extract x coordinate
    x_fn: Box<dyn Fn(&DataPoint, usize) -> f64 + Send + Sync>,
    /// Function to extract baseline y coordinate (y0)
    y0_fn: Box<dyn Fn(&DataPoint, usize) -> f64 + Send + Sync>,
    /// Function to extract top y coordinate (y1)
    y1_fn: Box<dyn Fn(&DataPoint, usize) -> f64 + Send + Sync>,
    /// Function to determine if a point is defined
    defined_fn: Box<dyn Fn(&DataPoint, usize) -> bool + Send + Sync>,
    /// Curve interpolation
    curve: Box<dyn Curve>,
}

impl Default for AreaGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl AreaGenerator {
    /// Create a new area generator with default settings
    ///
    /// Default baseline is y=0, top is the data point's y value.
    pub fn new() -> Self {
        Self {
            x_fn: Box::new(|d, i| d.x_or(i)),
            y0_fn: Box::new(|_, _| 0.0),
            y1_fn: Box::new(|d, _| d.y),
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

    /// Set the baseline y accessor function (y0)
    pub fn y0<F>(mut self, f: F) -> Self
    where
        F: Fn(&DataPoint, usize) -> f64 + Send + Sync + 'static,
    {
        self.y0_fn = Box::new(f);
        self
    }

    /// Set the top y accessor function (y1)
    pub fn y1<F>(mut self, f: F) -> Self
    where
        F: Fn(&DataPoint, usize) -> f64 + Send + Sync + 'static,
    {
        self.y1_fn = Box::new(f);
        self
    }

    /// Set both y0 and y1 to the same value (for range areas like error bands)
    pub fn y<F>(self, f: F) -> Self
    where
        F: Fn(&DataPoint, usize) -> f64 + Send + Sync + Clone + 'static,
    {
        let f_clone = f.clone();
        self.y0(f).y1(f_clone)
    }

    /// Set the defined predicate
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
        // Collect defined points
        let mut top_points: Vec<Point> = Vec::new();
        let mut bottom_points: Vec<Point> = Vec::new();

        for (i, d) in data.iter().enumerate() {
            if (self.defined_fn)(d, i) {
                let x = (self.x_fn)(d, i);
                let y0 = (self.y0_fn)(d, i);
                let y1 = (self.y1_fn)(d, i);

                top_points.push(Point::new(x, y1));
                bottom_points.push(Point::new(x, y0));
            }
        }

        if top_points.is_empty() {
            return vec![];
        }

        // Generate top curve
        let mut path = self.curve.generate(&top_points);

        // Reverse bottom points and generate bottom curve
        bottom_points.reverse();
        let bottom_path = self.curve.generate(&bottom_points);

        // Connect to bottom curve (skip the MoveTo)
        if bottom_path.len() > 1 {
            // Add line to first bottom point
            if let Some(PathSegment::MoveTo(p)) = bottom_path.first() {
                path.push(PathSegment::LineTo(*p));
            }
            // Add rest of bottom curve
            path.extend(bottom_path.into_iter().skip(1));
        }

        // Close the path
        path.push(PathSegment::ClosePath);

        path
    }

    /// Generate an area between two y values for each data point
    ///
    /// Useful for error bands or range areas.
    pub fn generate_range(&self, data: &[DataPoint]) -> Vec<PathSegment> {
        self.generate(data)
    }
}

/// Convenience functions for creating area generators with specific curves
impl AreaGenerator {
    /// Create an area generator with linear interpolation
    pub fn linear() -> Self {
        Self::new().curve(LinearCurve)
    }

    /// Create an area generator with step interpolation
    pub fn step() -> Self {
        use super::curve::StepCurve;
        Self::new().curve(StepCurve::middle())
    }

    /// Create an area generator with Catmull-Rom interpolation
    pub fn catmull_rom() -> Self {
        use super::curve::CatmullRomCurve;
        Self::new().curve(CatmullRomCurve::centripetal())
    }

    /// Create an area generator with monotone interpolation
    pub fn monotone() -> Self {
        use super::curve::MonotoneCurve;
        Self::new().curve(MonotoneCurve::new())
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
    fn test_area_generator_basic() {
        let data = sample_data();
        let area = AreaGenerator::new();
        let path = area.generate(&data);

        // Should have: MoveTo + LineTo's for top + LineTo to bottom + LineTo's for bottom + ClosePath
        assert!(!path.is_empty());

        // Should be closed
        assert!(matches!(path.last(), Some(PathSegment::ClosePath)));
    }

    #[test]
    fn test_area_generator_custom_baseline() {
        let data = sample_data();
        let area = AreaGenerator::new().y0(|_, _| 50.0);
        let path = area.generate(&data);

        assert!(!path.is_empty());
    }

    #[test]
    fn test_area_generator_range() {
        // Create data with y_min for range areas using range constructor
        let mut p1 = DataPoint::new(0.0, 100.0);
        p1.y_min = Some(50.0);

        let mut p2 = DataPoint::new(50.0, 150.0);
        p2.y_min = Some(100.0);

        let mut p3 = DataPoint::new(100.0, 120.0);
        p3.y_min = Some(80.0);

        let data = vec![p1, p2, p3];

        let area = AreaGenerator::new()
            .y0(|d, _| d.y_min.unwrap_or(0.0))
            .y1(|d, _| d.y);

        let path = area.generate(&data);
        assert!(!path.is_empty());
    }

    #[test]
    fn test_area_generator_empty() {
        let area = AreaGenerator::new();
        let path = area.generate(&[]);
        assert!(path.is_empty());
    }

    #[test]
    fn test_area_generator_curves() {
        let data = sample_data();

        let _ = AreaGenerator::linear().generate(&data);
        let _ = AreaGenerator::step().generate(&data);
        let _ = AreaGenerator::catmull_rom().generate(&data);
        let _ = AreaGenerator::monotone().generate(&data);
    }
}

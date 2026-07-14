//! Geographic path generator
//!
//! Generates SVG-like path segments from geographic data.

use super::geojson::{Feature, FeatureCollection, GeoJson, Geometry, Position};
use super::projection::Projection;

/// A segment of a geographic path
#[derive(Clone, Debug, PartialEq)]
pub enum GeoPathSegment {
    /// Move to a point (M x y)
    MoveTo(f64, f64),
    /// Line to a point (L x y)
    LineTo(f64, f64),
    /// Close the path (Z)
    ClosePath,
}

impl GeoPathSegment {
    /// Convert segment to SVG path command string
    pub fn to_svg(&self) -> String {
        match self {
            GeoPathSegment::MoveTo(x, y) => format!("M{:.6},{:.6}", x, y),
            GeoPathSegment::LineTo(x, y) => format!("L{:.6},{:.6}", x, y),
            GeoPathSegment::ClosePath => "Z".to_string(),
        }
    }
}

/// Geographic path generator
///
/// Converts geographic coordinates to screen coordinates using a projection
/// and generates path segments for rendering.
///
/// # Example
///
/// ```
/// use makepad_d3::geo::{GeoPath, MercatorProjection, Projection, ProjectionBuilder, GeoJson};
///
/// let projection = MercatorProjection::new()
///     .scale(100.0)
///     .translate(400.0, 300.0);
///
/// let path_gen = GeoPath::new(&projection);
///
/// // Parse GeoJSON and generate path
/// let geojson_str = r#"{
///     "type": "Feature",
///     "geometry": {
///         "type": "LineString",
///         "coordinates": [[-122.4, 37.8], [-73.9, 40.7]]
///     }
/// }"#;
///
/// let geojson = GeoJson::parse(geojson_str).unwrap();
/// let segments = path_gen.generate(&geojson);
/// let svg_path = path_gen.to_svg(&geojson);
/// ```
pub struct GeoPath<'a, P: Projection> {
    projection: &'a P,
    /// Point radius for rendering point geometries
    point_radius: f64,
}

impl<'a, P: Projection> GeoPath<'a, P> {
    /// Create a new path generator with the given projection
    pub fn new(projection: &'a P) -> Self {
        Self {
            projection,
            point_radius: 4.5,
        }
    }

    /// Set the radius for rendering point geometries
    pub fn point_radius(mut self, radius: f64) -> Self {
        self.point_radius = radius.max(0.0);
        self
    }

    /// Generate path segments from GeoJSON
    pub fn generate(&self, geojson: &GeoJson) -> Vec<GeoPathSegment> {
        let mut segments = Vec::new();

        match geojson {
            GeoJson::Geometry(geometry) => {
                self.geometry_to_segments(geometry, &mut segments);
            }
            GeoJson::Feature(feature) => {
                if let Some(ref geometry) = feature.geometry {
                    self.geometry_to_segments(geometry, &mut segments);
                }
            }
            GeoJson::FeatureCollection(collection) => {
                for feature in &collection.features {
                    if let Some(ref geometry) = feature.geometry {
                        self.geometry_to_segments(geometry, &mut segments);
                    }
                }
            }
        }

        segments
    }

    /// Generate SVG path string from GeoJSON
    pub fn to_svg(&self, geojson: &GeoJson) -> String {
        let segments = self.generate(geojson);
        segments
            .iter()
            .map(|s| s.to_svg())
            .collect::<Vec<_>>()
            .join("")
    }

    /// Generate path segments for a geometry
    fn geometry_to_segments(&self, geometry: &Geometry, segments: &mut Vec<GeoPathSegment>) {
        match geometry {
            Geometry::Point { coordinates } => {
                self.point_to_segments(coordinates[0], coordinates[1], segments);
            }
            Geometry::MultiPoint { coordinates } => {
                for coord in coordinates {
                    self.point_to_segments(coord[0], coord[1], segments);
                }
            }
            Geometry::LineString { coordinates } => {
                self.line_to_segments(coordinates, segments);
            }
            Geometry::MultiLineString { coordinates } => {
                for line in coordinates {
                    self.line_to_segments(line, segments);
                }
            }
            Geometry::Polygon { coordinates } => {
                self.polygon_to_segments(coordinates, segments);
            }
            Geometry::MultiPolygon { coordinates } => {
                for polygon in coordinates {
                    self.polygon_to_segments(polygon, segments);
                }
            }
            Geometry::GeometryCollection { geometries } => {
                for geom in geometries {
                    self.geometry_to_segments(geom, segments);
                }
            }
        }
    }

    /// Generate path segments for a point (as a small circle)
    fn point_to_segments(&self, lon: f64, lat: f64, segments: &mut Vec<GeoPathSegment>) {
        if !self.projection.is_visible(lon, lat) {
            return;
        }

        let (x, y) = self.projection.project(lon, lat);

        // Generate a circle approximation using 8 line segments
        let n = 8;
        let r = self.point_radius;

        for i in 0..=n {
            let angle = 2.0 * std::f64::consts::PI * (i as f64) / (n as f64);
            let px = x + r * angle.cos();
            let py = y + r * angle.sin();

            if i == 0 {
                segments.push(GeoPathSegment::MoveTo(px, py));
            } else {
                segments.push(GeoPathSegment::LineTo(px, py));
            }
        }

        segments.push(GeoPathSegment::ClosePath);
    }

    /// Generate path segments for a line string
    fn line_to_segments(&self, coordinates: &[Position], segments: &mut Vec<GeoPathSegment>) {
        let mut started = false;

        for coord in coordinates {
            let lon = coord[0];
            let lat = coord[1];

            if !self.projection.is_visible(lon, lat) {
                // End current line segment if we hit invisible point
                started = false;
                continue;
            }

            let (x, y) = self.projection.project(lon, lat);

            if !started {
                segments.push(GeoPathSegment::MoveTo(x, y));
                started = true;
            } else {
                segments.push(GeoPathSegment::LineTo(x, y));
            }
        }
    }

    /// Generate path segments for a polygon
    fn polygon_to_segments(&self, rings: &[Vec<Position>], segments: &mut Vec<GeoPathSegment>) {
        for ring in rings {
            self.ring_to_segments(ring, segments);
        }
    }

    /// Generate path segments for a ring (closed line string)
    fn ring_to_segments(&self, coordinates: &[Position], segments: &mut Vec<GeoPathSegment>) {
        if coordinates.is_empty() {
            return;
        }

        let mut started = false;
        let mut visible_count = 0;

        for coord in coordinates {
            let lon = coord[0];
            let lat = coord[1];

            if !self.projection.is_visible(lon, lat) {
                continue;
            }

            visible_count += 1;
            let (x, y) = self.projection.project(lon, lat);

            if !started {
                segments.push(GeoPathSegment::MoveTo(x, y));
                started = true;
            } else {
                segments.push(GeoPathSegment::LineTo(x, y));
            }
        }

        // Close the path if we drew at least 2 visible points
        if visible_count >= 2 {
            segments.push(GeoPathSegment::ClosePath);
        }
    }

    /// Compute the centroid of a geometry
    pub fn centroid(&self, geometry: &Geometry) -> Option<(f64, f64)> {
        let coords = self.collect_coordinates(geometry);

        if coords.is_empty() {
            return None;
        }

        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut count = 0.0;

        for coord in &coords {
            if self.projection.is_visible(coord[0], coord[1]) {
                let (x, y) = self.projection.project(coord[0], coord[1]);
                sum_x += x;
                sum_y += y;
                count += 1.0;
            }
        }

        if count > 0.0 {
            Some((sum_x / count, sum_y / count))
        } else {
            None
        }
    }

    /// Collect all coordinates from a geometry
    fn collect_coordinates(&self, geometry: &Geometry) -> Vec<Position> {
        let mut coords = Vec::new();

        match geometry {
            Geometry::Point { coordinates } => {
                coords.push(*coordinates);
            }
            Geometry::MultiPoint { coordinates } => {
                coords.extend(coordinates.iter().copied());
            }
            Geometry::LineString { coordinates } => {
                coords.extend(coordinates.iter().copied());
            }
            Geometry::MultiLineString { coordinates } => {
                for line in coordinates {
                    coords.extend(line.iter().copied());
                }
            }
            Geometry::Polygon { coordinates } => {
                for ring in coordinates {
                    coords.extend(ring.iter().copied());
                }
            }
            Geometry::MultiPolygon { coordinates } => {
                for polygon in coordinates {
                    for ring in polygon {
                        coords.extend(ring.iter().copied());
                    }
                }
            }
            Geometry::GeometryCollection { geometries } => {
                for geom in geometries {
                    coords.extend(self.collect_coordinates(geom));
                }
            }
        }

        coords
    }

    /// Compute the bounding box of a geometry in projected coordinates
    pub fn bounds(&self, geometry: &Geometry) -> Option<[[f64; 2]; 2]> {
        let coords = self.collect_coordinates(geometry);

        if coords.is_empty() {
            return None;
        }

        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        for coord in &coords {
            if self.projection.is_visible(coord[0], coord[1]) {
                let (x, y) = self.projection.project(coord[0], coord[1]);
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }

        if min_x.is_finite() && min_y.is_finite() && max_x.is_finite() && max_y.is_finite() {
            Some([[min_x, min_y], [max_x, max_y]])
        } else {
            None
        }
    }

    /// Calculate the approximate area of a geometry in square pixels
    pub fn area(&self, geometry: &Geometry) -> f64 {
        match geometry {
            Geometry::Polygon { coordinates } => self.polygon_area(coordinates),
            Geometry::MultiPolygon { coordinates } => {
                coordinates.iter().map(|p| self.polygon_area(p)).sum()
            }
            _ => 0.0,
        }
    }

    /// Calculate the area of a polygon using the shoelace formula
    fn polygon_area(&self, rings: &[Vec<Position>]) -> f64 {
        if rings.is_empty() {
            return 0.0;
        }

        // Outer ring area
        let outer_area = self.ring_area(&rings[0]);

        // Subtract hole areas
        let hole_area: f64 = rings.iter().skip(1).map(|ring| self.ring_area(ring)).sum();

        (outer_area - hole_area).abs()
    }

    /// Calculate the area of a ring using the shoelace formula
    fn ring_area(&self, ring: &[Position]) -> f64 {
        if ring.len() < 3 {
            return 0.0;
        }

        // Project all coordinates
        let projected: Vec<(f64, f64)> = ring
            .iter()
            .filter(|c| self.projection.is_visible(c[0], c[1]))
            .map(|c| self.projection.project(c[0], c[1]))
            .collect();

        if projected.len() < 3 {
            return 0.0;
        }

        // Shoelace formula
        let mut area = 0.0;
        let n = projected.len();

        for i in 0..n {
            let j = (i + 1) % n;
            area += projected[i].0 * projected[j].1;
            area -= projected[j].0 * projected[i].1;
        }

        area.abs() / 2.0
    }

    /// Calculate the approximate length of a geometry in pixels
    pub fn measure(&self, geometry: &Geometry) -> f64 {
        match geometry {
            Geometry::LineString { coordinates } => self.line_length(coordinates),
            Geometry::MultiLineString { coordinates } => {
                coordinates.iter().map(|l| self.line_length(l)).sum()
            }
            Geometry::Polygon { coordinates } => {
                coordinates.iter().map(|r| self.line_length(r)).sum()
            }
            Geometry::MultiPolygon { coordinates } => coordinates
                .iter()
                .flat_map(|p| p.iter())
                .map(|r| self.line_length(r))
                .sum(),
            _ => 0.0,
        }
    }

    /// Calculate the length of a line string in projected coordinates
    fn line_length(&self, coordinates: &[Position]) -> f64 {
        if coordinates.len() < 2 {
            return 0.0;
        }

        let mut length = 0.0;
        let mut prev: Option<(f64, f64)> = None;

        for coord in coordinates {
            if !self.projection.is_visible(coord[0], coord[1]) {
                prev = None;
                continue;
            }

            let (x, y) = self.projection.project(coord[0], coord[1]);

            if let Some((px, py)) = prev {
                let dx = x - px;
                let dy = y - py;
                length += (dx * dx + dy * dy).sqrt();
            }

            prev = Some((x, y));
        }

        length
    }
}

/// Builder for creating geographic paths with custom settings
pub struct GeoPathBuilder<'a, P: Projection> {
    projection: &'a P,
    point_radius: f64,
}

impl<'a, P: Projection> GeoPathBuilder<'a, P> {
    /// Create a new builder with a projection
    pub fn new(projection: &'a P) -> Self {
        Self {
            projection,
            point_radius: 4.5,
        }
    }

    /// Set the point radius
    pub fn point_radius(mut self, radius: f64) -> Self {
        self.point_radius = radius;
        self
    }

    /// Build the GeoPath
    pub fn build(self) -> GeoPath<'a, P> {
        GeoPath {
            projection: self.projection,
            point_radius: self.point_radius,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geo::projection::{
        EquirectangularProjection, MercatorProjection, ProjectionBuilder,
    };

    #[test]
    fn test_geo_path_segment_svg() {
        assert_eq!(
            GeoPathSegment::MoveTo(10.0, 20.0).to_svg(),
            "M10.000000,20.000000"
        );
        assert_eq!(
            GeoPathSegment::LineTo(30.0, 40.0).to_svg(),
            "L30.000000,40.000000"
        );
        assert_eq!(GeoPathSegment::ClosePath.to_svg(), "Z");
    }

    #[test]
    fn test_geo_path_new() {
        let projection = MercatorProjection::new();
        let path = GeoPath::new(&projection);
        assert_eq!(path.point_radius, 4.5);
    }

    #[test]
    fn test_geo_path_point_radius() {
        let projection = MercatorProjection::new();
        let path = GeoPath::new(&projection).point_radius(10.0);
        assert_eq!(path.point_radius, 10.0);
    }

    #[test]
    fn test_geo_path_point() {
        let projection = EquirectangularProjection::new()
            .scale(1.0)
            .translate(0.0, 0.0);
        let path = GeoPath::new(&projection).point_radius(5.0);

        let geometry = Geometry::Point {
            coordinates: [0.0, 0.0],
        };
        let geojson = GeoJson::Geometry(geometry);
        let segments = path.generate(&geojson);

        // Should have MoveTo, several LineTo, and ClosePath
        assert!(segments.len() > 2);
        assert!(matches!(segments[0], GeoPathSegment::MoveTo(_, _)));
        assert!(matches!(segments.last(), Some(GeoPathSegment::ClosePath)));
    }

    #[test]
    fn test_geo_path_linestring() {
        let projection = EquirectangularProjection::new()
            .scale(1.0)
            .translate(0.0, 0.0);
        let path = GeoPath::new(&projection);

        let geometry = Geometry::LineString {
            coordinates: vec![[0.0, 0.0], [10.0, 10.0], [20.0, 0.0]],
        };
        let geojson = GeoJson::Geometry(geometry);
        let segments = path.generate(&geojson);

        assert_eq!(segments.len(), 3);
        assert!(matches!(segments[0], GeoPathSegment::MoveTo(_, _)));
        assert!(matches!(segments[1], GeoPathSegment::LineTo(_, _)));
        assert!(matches!(segments[2], GeoPathSegment::LineTo(_, _)));
    }

    #[test]
    fn test_geo_path_polygon() {
        let projection = EquirectangularProjection::new()
            .scale(1.0)
            .translate(0.0, 0.0);
        let path = GeoPath::new(&projection);

        let geometry = Geometry::Polygon {
            coordinates: vec![vec![
                [0.0, 0.0],
                [10.0, 0.0],
                [10.0, 10.0],
                [0.0, 10.0],
                [0.0, 0.0],
            ]],
        };
        let geojson = GeoJson::Geometry(geometry);
        let segments = path.generate(&geojson);

        // Should have MoveTo, LineTo's, and ClosePath
        assert!(segments.len() >= 3);
        assert!(matches!(segments[0], GeoPathSegment::MoveTo(_, _)));
        assert!(matches!(segments.last(), Some(GeoPathSegment::ClosePath)));
    }

    #[test]
    fn test_geo_path_to_svg() {
        let projection = EquirectangularProjection::new()
            .scale(1.0)
            .translate(0.0, 0.0);
        let path = GeoPath::new(&projection);

        let geometry = Geometry::LineString {
            coordinates: vec![[0.0, 0.0], [10.0, 0.0]],
        };
        let geojson = GeoJson::Geometry(geometry);
        let svg = path.to_svg(&geojson);

        assert!(svg.starts_with('M'));
        assert!(svg.contains('L'));
    }

    #[test]
    fn test_geo_path_feature() {
        let projection = EquirectangularProjection::new()
            .scale(1.0)
            .translate(0.0, 0.0);
        let path = GeoPath::new(&projection);

        let feature = Feature {
            geometry: Some(Geometry::Point {
                coordinates: [0.0, 0.0],
            }),
            properties: None,
            id: None,
            bbox: None,
        };
        let geojson = GeoJson::Feature(Box::new(feature));
        let segments = path.generate(&geojson);

        assert!(!segments.is_empty());
    }

    #[test]
    fn test_geo_path_feature_collection() {
        let projection = EquirectangularProjection::new()
            .scale(1.0)
            .translate(0.0, 0.0);
        let path = GeoPath::new(&projection);

        let collection = FeatureCollection {
            features: vec![
                Feature {
                    geometry: Some(Geometry::Point {
                        coordinates: [0.0, 0.0],
                    }),
                    properties: None,
                    id: None,
                    bbox: None,
                },
                Feature {
                    geometry: Some(Geometry::Point {
                        coordinates: [10.0, 10.0],
                    }),
                    properties: None,
                    id: None,
                    bbox: None,
                },
            ],
            bbox: None,
        };
        let geojson = GeoJson::FeatureCollection(collection);
        let segments = path.generate(&geojson);

        // Should have segments for both points
        assert!(segments.len() > 10);
    }

    #[test]
    fn test_geo_path_centroid() {
        let projection = EquirectangularProjection::new()
            .scale(1.0)
            .translate(0.0, 0.0);
        let path = GeoPath::new(&projection);

        let geometry = Geometry::Polygon {
            coordinates: vec![vec![
                [0.0, 0.0],
                [10.0, 0.0],
                [10.0, 10.0],
                [0.0, 10.0],
                [0.0, 0.0],
            ]],
        };

        let centroid = path.centroid(&geometry);
        assert!(centroid.is_some());

        let (cx, cy) = centroid.unwrap();
        // Centroid should be roughly in the middle
        assert!((cx - 5.0 * std::f64::consts::PI / 180.0).abs() < 1.0);
    }

    #[test]
    fn test_geo_path_bounds() {
        let projection = EquirectangularProjection::new()
            .scale(1.0)
            .translate(0.0, 0.0);
        let path = GeoPath::new(&projection);

        let geometry = Geometry::LineString {
            coordinates: vec![[0.0, 0.0], [10.0, 10.0], [20.0, 5.0]],
        };

        let bounds = path.bounds(&geometry);
        assert!(bounds.is_some());

        let [[min_x, min_y], [max_x, max_y]] = bounds.unwrap();
        assert!(min_x < max_x);
        assert!(min_y < max_y);
    }

    #[test]
    fn test_geo_path_area() {
        let projection = EquirectangularProjection::new()
            .scale(100.0)
            .translate(0.0, 0.0);
        let path = GeoPath::new(&projection);

        let geometry = Geometry::Polygon {
            coordinates: vec![vec![
                [0.0, 0.0],
                [1.0, 0.0],
                [1.0, 1.0],
                [0.0, 1.0],
                [0.0, 0.0],
            ]],
        };

        let area = path.area(&geometry);
        assert!(area > 0.0);
    }

    #[test]
    fn test_geo_path_measure() {
        let projection = EquirectangularProjection::new()
            .scale(100.0)
            .translate(0.0, 0.0);
        let path = GeoPath::new(&projection);

        let geometry = Geometry::LineString {
            coordinates: vec![[0.0, 0.0], [1.0, 0.0]],
        };

        let length = path.measure(&geometry);
        assert!(length > 0.0);
    }

    #[test]
    fn test_geo_path_multipoint() {
        let projection = EquirectangularProjection::new()
            .scale(1.0)
            .translate(0.0, 0.0);
        let path = GeoPath::new(&projection);

        let geometry = Geometry::MultiPoint {
            coordinates: vec![[0.0, 0.0], [10.0, 10.0]],
        };
        let geojson = GeoJson::Geometry(geometry);
        let segments = path.generate(&geojson);

        // Should have segments for both points
        assert!(segments.len() > 10);
    }

    #[test]
    fn test_geo_path_multilinestring() {
        let projection = EquirectangularProjection::new()
            .scale(1.0)
            .translate(0.0, 0.0);
        let path = GeoPath::new(&projection);

        let geometry = Geometry::MultiLineString {
            coordinates: vec![
                vec![[0.0, 0.0], [10.0, 0.0]],
                vec![[0.0, 10.0], [10.0, 10.0]],
            ],
        };
        let geojson = GeoJson::Geometry(geometry);
        let segments = path.generate(&geojson);

        // Should have 2 MoveTo's (one for each line)
        let move_count = segments
            .iter()
            .filter(|s| matches!(s, GeoPathSegment::MoveTo(_, _)))
            .count();
        assert_eq!(move_count, 2);
    }

    #[test]
    fn test_geo_path_multipolygon() {
        let projection = EquirectangularProjection::new()
            .scale(1.0)
            .translate(0.0, 0.0);
        let path = GeoPath::new(&projection);

        let geometry = Geometry::MultiPolygon {
            coordinates: vec![
                vec![vec![
                    [0.0, 0.0],
                    [5.0, 0.0],
                    [5.0, 5.0],
                    [0.0, 5.0],
                    [0.0, 0.0],
                ]],
                vec![vec![
                    [10.0, 10.0],
                    [15.0, 10.0],
                    [15.0, 15.0],
                    [10.0, 15.0],
                    [10.0, 10.0],
                ]],
            ],
        };
        let geojson = GeoJson::Geometry(geometry);
        let segments = path.generate(&geojson);

        // Should have segments for both polygons
        assert!(!segments.is_empty());
    }

    #[test]
    fn test_geo_path_geometry_collection() {
        let projection = EquirectangularProjection::new()
            .scale(1.0)
            .translate(0.0, 0.0);
        let path = GeoPath::new(&projection);

        let geometry = Geometry::GeometryCollection {
            geometries: vec![
                Geometry::Point {
                    coordinates: [0.0, 0.0],
                },
                Geometry::LineString {
                    coordinates: vec![[10.0, 10.0], [20.0, 20.0]],
                },
            ],
        };
        let geojson = GeoJson::Geometry(geometry);
        let segments = path.generate(&geojson);

        // Should have segments for both geometries
        assert!(!segments.is_empty());
    }

    #[test]
    fn test_geo_path_builder() {
        let projection = MercatorProjection::new();
        let path = GeoPathBuilder::new(&projection).point_radius(8.0).build();

        assert_eq!(path.point_radius, 8.0);
    }

    #[test]
    fn test_geo_path_empty_geometry() {
        let projection = EquirectangularProjection::new();
        let path = GeoPath::new(&projection);

        let geometry = Geometry::LineString {
            coordinates: vec![],
        };
        let geojson = GeoJson::Geometry(geometry);
        let segments = path.generate(&geojson);

        assert!(segments.is_empty());
    }

    #[test]
    fn test_geo_path_polygon_with_hole() {
        let projection = EquirectangularProjection::new()
            .scale(100.0)
            .translate(0.0, 0.0);
        let path = GeoPath::new(&projection);

        let geometry = Geometry::Polygon {
            coordinates: vec![
                // Outer ring
                vec![
                    [0.0, 0.0],
                    [10.0, 0.0],
                    [10.0, 10.0],
                    [0.0, 10.0],
                    [0.0, 0.0],
                ],
                // Hole
                vec![[2.0, 2.0], [8.0, 2.0], [8.0, 8.0], [2.0, 8.0], [2.0, 2.0]],
            ],
        };

        let area = path.area(&geometry);
        // Area should be positive but less than full square
        assert!(area > 0.0);
    }
}

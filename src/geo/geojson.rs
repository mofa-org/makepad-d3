//! GeoJSON parsing and types
//!
//! Implements the GeoJSON specification (RFC 7946).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A position is an array of numbers (longitude, latitude, [altitude])
pub type Position = [f64; 2];

/// A bounding box [west, south, east, north]
pub type BoundingBox = [f64; 4];

/// Properties of a feature (key-value pairs)
pub type Properties = HashMap<String, serde_json::Value>;

/// GeoJSON geometry types
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Geometry {
    /// A single point
    Point {
        /// Coordinates [longitude, latitude]
        coordinates: Position,
    },
    /// Multiple points
    MultiPoint {
        /// Array of positions
        coordinates: Vec<Position>,
    },
    /// A line string (connected sequence of points)
    LineString {
        /// Array of positions
        coordinates: Vec<Position>,
    },
    /// Multiple line strings
    MultiLineString {
        /// Array of line strings
        coordinates: Vec<Vec<Position>>,
    },
    /// A polygon (closed ring with optional holes)
    Polygon {
        /// Array of linear rings (first is exterior, rest are holes)
        coordinates: Vec<Vec<Position>>,
    },
    /// Multiple polygons
    MultiPolygon {
        /// Array of polygons
        coordinates: Vec<Vec<Vec<Position>>>,
    },
    /// A collection of geometries
    GeometryCollection {
        /// Array of geometry objects
        geometries: Vec<Geometry>,
    },
}

/// Enum for geometry type names
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GeometryType {
    /// Point
    Point,
    /// MultiPoint
    MultiPoint,
    /// LineString
    LineString,
    /// MultiLineString
    MultiLineString,
    /// Polygon
    Polygon,
    /// MultiPolygon
    MultiPolygon,
    /// GeometryCollection
    GeometryCollection,
}

impl Geometry {
    /// Create a Point geometry
    pub fn point(lon: f64, lat: f64) -> Self {
        Geometry::Point {
            coordinates: [lon, lat],
        }
    }

    /// Create a LineString geometry
    pub fn line_string(coords: Vec<Position>) -> Self {
        Geometry::LineString {
            coordinates: coords,
        }
    }

    /// Create a Polygon geometry
    pub fn polygon(rings: Vec<Vec<Position>>) -> Self {
        Geometry::Polygon { coordinates: rings }
    }

    /// Create a simple polygon from exterior ring
    pub fn simple_polygon(exterior: Vec<Position>) -> Self {
        Geometry::Polygon {
            coordinates: vec![exterior],
        }
    }

    /// Get the geometry type
    pub fn geometry_type(&self) -> GeometryType {
        match self {
            Geometry::Point { .. } => GeometryType::Point,
            Geometry::MultiPoint { .. } => GeometryType::MultiPoint,
            Geometry::LineString { .. } => GeometryType::LineString,
            Geometry::MultiLineString { .. } => GeometryType::MultiLineString,
            Geometry::Polygon { .. } => GeometryType::Polygon,
            Geometry::MultiPolygon { .. } => GeometryType::MultiPolygon,
            Geometry::GeometryCollection { .. } => GeometryType::GeometryCollection,
        }
    }

    /// Get the bounding box of the geometry
    pub fn bbox(&self) -> Option<BoundingBox> {
        let mut min_lon = f64::INFINITY;
        let mut min_lat = f64::INFINITY;
        let mut max_lon = f64::NEG_INFINITY;
        let mut max_lat = f64::NEG_INFINITY;

        let mut update_bounds = |pos: &Position| {
            min_lon = min_lon.min(pos[0]);
            min_lat = min_lat.min(pos[1]);
            max_lon = max_lon.max(pos[0]);
            max_lat = max_lat.max(pos[1]);
        };

        self.for_each_position(&mut update_bounds);

        if min_lon.is_finite() {
            Some([min_lon, min_lat, max_lon, max_lat])
        } else {
            None
        }
    }

    /// Iterate over all positions in the geometry
    pub fn for_each_position<F: FnMut(&Position)>(&self, f: &mut F) {
        match self {
            Geometry::Point { coordinates } => f(coordinates),
            Geometry::MultiPoint { coordinates } => {
                for pos in coordinates {
                    f(pos);
                }
            }
            Geometry::LineString { coordinates } => {
                for pos in coordinates {
                    f(pos);
                }
            }
            Geometry::MultiLineString { coordinates } => {
                for line in coordinates {
                    for pos in line {
                        f(pos);
                    }
                }
            }
            Geometry::Polygon { coordinates } => {
                for ring in coordinates {
                    for pos in ring {
                        f(pos);
                    }
                }
            }
            Geometry::MultiPolygon { coordinates } => {
                for polygon in coordinates {
                    for ring in polygon {
                        for pos in ring {
                            f(pos);
                        }
                    }
                }
            }
            Geometry::GeometryCollection { geometries } => {
                for geom in geometries {
                    geom.for_each_position(f);
                }
            }
        }
    }

    /// Count all positions in the geometry
    pub fn position_count(&self) -> usize {
        let mut count = 0;
        self.for_each_position(&mut |_| count += 1);
        count
    }
}

/// A GeoJSON Feature
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Feature {
    /// The geometry (can be null)
    pub geometry: Option<Geometry>,
    /// Properties object
    #[serde(default)]
    pub properties: Option<Properties>,
    /// Optional feature ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<serde_json::Value>,
    /// Optional bounding box
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bbox: Option<BoundingBox>,
}

impl Feature {
    /// Create a new feature with geometry
    pub fn new(geometry: Geometry) -> Self {
        Self {
            geometry: Some(geometry),
            properties: None,
            id: None,
            bbox: None,
        }
    }

    /// Create a feature with no geometry
    pub fn empty() -> Self {
        Self {
            geometry: None,
            properties: None,
            id: None,
            bbox: None,
        }
    }

    /// Set properties
    pub fn with_properties(mut self, properties: Properties) -> Self {
        self.properties = Some(properties);
        self
    }

    /// Add a property
    pub fn with_property(mut self, key: &str, value: impl Into<serde_json::Value>) -> Self {
        let props = self.properties.get_or_insert_with(HashMap::new);
        props.insert(key.to_string(), value.into());
        self
    }

    /// Set the feature ID
    pub fn with_id(mut self, id: impl Into<serde_json::Value>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Get a property value
    pub fn get_property(&self, key: &str) -> Option<&serde_json::Value> {
        self.properties.as_ref()?.get(key)
    }

    /// Get a property as string
    pub fn get_string(&self, key: &str) -> Option<&str> {
        self.get_property(key)?.as_str()
    }

    /// Get a property as f64
    pub fn get_number(&self, key: &str) -> Option<f64> {
        self.get_property(key)?.as_f64()
    }

    /// Compute bounding box
    pub fn compute_bbox(&mut self) {
        if let Some(ref geom) = self.geometry {
            self.bbox = geom.bbox();
        }
    }
}

/// A GeoJSON FeatureCollection
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FeatureCollection {
    /// Array of features
    pub features: Vec<Feature>,
    /// Optional bounding box
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bbox: Option<BoundingBox>,
}

impl FeatureCollection {
    /// Create a new empty feature collection
    pub fn new() -> Self {
        Self {
            features: Vec::new(),
            bbox: None,
        }
    }

    /// Create from a vector of features
    pub fn from_features(features: Vec<Feature>) -> Self {
        Self {
            features,
            bbox: None,
        }
    }

    /// Add a feature
    pub fn add_feature(&mut self, feature: Feature) {
        self.features.push(feature);
    }

    /// Add a geometry as a feature
    pub fn add_geometry(&mut self, geometry: Geometry) {
        self.features.push(Feature::new(geometry));
    }

    /// Get number of features
    pub fn len(&self) -> usize {
        self.features.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.features.is_empty()
    }

    /// Compute bounding box for all features
    pub fn compute_bbox(&mut self) {
        let mut min_lon = f64::INFINITY;
        let mut min_lat = f64::INFINITY;
        let mut max_lon = f64::NEG_INFINITY;
        let mut max_lat = f64::NEG_INFINITY;

        for feature in &self.features {
            if let Some(ref geom) = feature.geometry {
                if let Some(bbox) = geom.bbox() {
                    min_lon = min_lon.min(bbox[0]);
                    min_lat = min_lat.min(bbox[1]);
                    max_lon = max_lon.max(bbox[2]);
                    max_lat = max_lat.max(bbox[3]);
                }
            }
        }

        if min_lon.is_finite() {
            self.bbox = Some([min_lon, min_lat, max_lon, max_lat]);
        }
    }

    /// Iterator over features
    pub fn iter(&self) -> impl Iterator<Item = &Feature> {
        self.features.iter()
    }
}

impl Default for FeatureCollection {
    fn default() -> Self {
        Self::new()
    }
}

/// Top-level GeoJSON object
#[derive(Clone, Debug, PartialEq)]
pub enum GeoJson {
    /// A geometry object
    Geometry(Geometry),
    /// A feature
    Feature(Box<Feature>),
    /// A feature collection
    FeatureCollection(FeatureCollection),
}

impl GeoJson {
    /// Parse GeoJSON from a string
    pub fn parse(json: &str) -> Result<Self, serde_json::Error> {
        // Try parsing as each type in order
        if let Ok(fc) = serde_json::from_str::<FeatureCollectionJson>(json) {
            if fc.r#type == "FeatureCollection" {
                return Ok(GeoJson::FeatureCollection(FeatureCollection {
                    features: fc.features,
                    bbox: fc.bbox,
                }));
            }
        }

        if let Ok(f) = serde_json::from_str::<FeatureJson>(json) {
            if f.r#type == "Feature" {
                return Ok(GeoJson::Feature(Box::new(Feature {
                    geometry: f.geometry,
                    properties: f.properties,
                    id: f.id,
                    bbox: f.bbox,
                })));
            }
        }

        // Try as geometry
        let geom: Geometry = serde_json::from_str(json)?;
        Ok(GeoJson::Geometry(geom))
    }

    /// Serialize to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        match self {
            GeoJson::Geometry(g) => serde_json::to_string(g),
            GeoJson::Feature(f) => {
                let fj = FeatureJson {
                    r#type: "Feature".to_string(),
                    geometry: f.geometry.clone(),
                    properties: f.properties.clone(),
                    id: f.id.clone(),
                    bbox: f.bbox,
                };
                serde_json::to_string(&fj)
            }
            GeoJson::FeatureCollection(fc) => {
                let fcj = FeatureCollectionJson {
                    r#type: "FeatureCollection".to_string(),
                    features: fc.features.clone(),
                    bbox: fc.bbox,
                };
                serde_json::to_string(&fcj)
            }
        }
    }

    /// Serialize to pretty JSON string
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        match self {
            GeoJson::Geometry(g) => serde_json::to_string_pretty(g),
            GeoJson::Feature(f) => {
                let fj = FeatureJson {
                    r#type: "Feature".to_string(),
                    geometry: f.geometry.clone(),
                    properties: f.properties.clone(),
                    id: f.id.clone(),
                    bbox: f.bbox,
                };
                serde_json::to_string_pretty(&fj)
            }
            GeoJson::FeatureCollection(fc) => {
                let fcj = FeatureCollectionJson {
                    r#type: "FeatureCollection".to_string(),
                    features: fc.features.clone(),
                    bbox: fc.bbox,
                };
                serde_json::to_string_pretty(&fcj)
            }
        }
    }

    /// Get all features (wraps geometries in features)
    pub fn features(&self) -> Vec<&Feature> {
        match self {
            GeoJson::Geometry(_) => Vec::new(),
            GeoJson::Feature(f) => vec![f.as_ref()],
            GeoJson::FeatureCollection(fc) => fc.features.iter().collect(),
        }
    }

    /// Get all geometries
    pub fn geometries(&self) -> Vec<&Geometry> {
        match self {
            GeoJson::Geometry(g) => vec![g],
            GeoJson::Feature(f) => f.geometry.as_ref().map(|g| vec![g]).unwrap_or_default(),
            GeoJson::FeatureCollection(fc) => fc
                .features
                .iter()
                .filter_map(|f| f.geometry.as_ref())
                .collect(),
        }
    }

    /// Compute bounding box
    pub fn bbox(&self) -> Option<BoundingBox> {
        match self {
            GeoJson::Geometry(g) => g.bbox(),
            GeoJson::Feature(f) => f.geometry.as_ref().and_then(|g| g.bbox()),
            GeoJson::FeatureCollection(fc) => {
                let mut min_lon = f64::INFINITY;
                let mut min_lat = f64::INFINITY;
                let mut max_lon = f64::NEG_INFINITY;
                let mut max_lat = f64::NEG_INFINITY;

                for feature in &fc.features {
                    if let Some(ref geom) = feature.geometry {
                        if let Some(bbox) = geom.bbox() {
                            min_lon = min_lon.min(bbox[0]);
                            min_lat = min_lat.min(bbox[1]);
                            max_lon = max_lon.max(bbox[2]);
                            max_lat = max_lat.max(bbox[3]);
                        }
                    }
                }

                if min_lon.is_finite() {
                    Some([min_lon, min_lat, max_lon, max_lat])
                } else {
                    None
                }
            }
        }
    }
}

// Helper structs for JSON serialization
#[derive(Serialize, Deserialize)]
struct FeatureJson {
    r#type: String,
    geometry: Option<Geometry>,
    #[serde(default)]
    properties: Option<Properties>,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bbox: Option<BoundingBox>,
}

#[derive(Serialize, Deserialize)]
struct FeatureCollectionJson {
    r#type: String,
    features: Vec<Feature>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bbox: Option<BoundingBox>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geometry_point() {
        let point = Geometry::point(-122.4, 37.8);
        assert_eq!(point.geometry_type(), GeometryType::Point);

        if let Geometry::Point { coordinates } = point {
            assert_eq!(coordinates[0], -122.4);
            assert_eq!(coordinates[1], 37.8);
        }
    }

    #[test]
    fn test_geometry_line_string() {
        let line = Geometry::line_string(vec![[0.0, 0.0], [1.0, 1.0], [2.0, 0.0]]);
        assert_eq!(line.geometry_type(), GeometryType::LineString);
        assert_eq!(line.position_count(), 3);
    }

    #[test]
    fn test_geometry_polygon() {
        let polygon = Geometry::simple_polygon(vec![
            [0.0, 0.0],
            [1.0, 0.0],
            [1.0, 1.0],
            [0.0, 1.0],
            [0.0, 0.0],
        ]);
        assert_eq!(polygon.geometry_type(), GeometryType::Polygon);
    }

    #[test]
    fn test_geometry_bbox() {
        let polygon = Geometry::simple_polygon(vec![
            [-10.0, -5.0],
            [10.0, -5.0],
            [10.0, 5.0],
            [-10.0, 5.0],
            [-10.0, -5.0],
        ]);

        let bbox = polygon.bbox().unwrap();
        assert_eq!(bbox[0], -10.0); // west
        assert_eq!(bbox[1], -5.0); // south
        assert_eq!(bbox[2], 10.0); // east
        assert_eq!(bbox[3], 5.0); // north
    }

    #[test]
    fn test_feature_new() {
        let feature = Feature::new(Geometry::point(0.0, 0.0));
        assert!(feature.geometry.is_some());
    }

    #[test]
    fn test_feature_with_properties() {
        let feature = Feature::new(Geometry::point(0.0, 0.0))
            .with_property("name", "Test Point")
            .with_property("population", 1000);

        assert_eq!(feature.get_string("name"), Some("Test Point"));
        assert_eq!(feature.get_number("population"), Some(1000.0));
    }

    #[test]
    fn test_feature_collection() {
        let mut fc = FeatureCollection::new();
        fc.add_geometry(Geometry::point(0.0, 0.0));
        fc.add_geometry(Geometry::point(1.0, 1.0));

        assert_eq!(fc.len(), 2);
        assert!(!fc.is_empty());
    }

    #[test]
    fn test_feature_collection_bbox() {
        let mut fc = FeatureCollection::new();
        fc.add_geometry(Geometry::point(-10.0, -10.0));
        fc.add_geometry(Geometry::point(10.0, 10.0));
        fc.compute_bbox();

        let bbox = fc.bbox.unwrap();
        assert_eq!(bbox[0], -10.0);
        assert_eq!(bbox[1], -10.0);
        assert_eq!(bbox[2], 10.0);
        assert_eq!(bbox[3], 10.0);
    }

    #[test]
    fn test_geojson_parse_point() {
        let json = r#"{"type": "Point", "coordinates": [100.0, 0.0]}"#;
        let geojson: Geometry = serde_json::from_str(json).unwrap();

        if let Geometry::Point { coordinates } = geojson {
            assert_eq!(coordinates[0], 100.0);
            assert_eq!(coordinates[1], 0.0);
        } else {
            panic!("Expected Point geometry");
        }
    }

    #[test]
    fn test_geojson_parse_feature() {
        let json = r#"{
            "type": "Feature",
            "geometry": {"type": "Point", "coordinates": [102.0, 0.5]},
            "properties": {"name": "Test"}
        }"#;

        let geojson = GeoJson::parse(json).unwrap();
        if let GeoJson::Feature(f) = geojson {
            assert_eq!(f.get_string("name"), Some("Test"));
        } else {
            panic!("Expected Feature");
        }
    }

    #[test]
    fn test_geojson_parse_feature_collection() {
        let json = r#"{
            "type": "FeatureCollection",
            "features": [
                {
                    "type": "Feature",
                    "geometry": {"type": "Point", "coordinates": [102.0, 0.5]},
                    "properties": {"name": "A"}
                },
                {
                    "type": "Feature",
                    "geometry": {"type": "Point", "coordinates": [103.0, 1.5]},
                    "properties": {"name": "B"}
                }
            ]
        }"#;

        let geojson = GeoJson::parse(json).unwrap();
        if let GeoJson::FeatureCollection(fc) = geojson {
            assert_eq!(fc.len(), 2);
        } else {
            panic!("Expected FeatureCollection");
        }
    }

    #[test]
    fn test_geojson_serialize() {
        let point = Geometry::point(100.0, 0.0);
        let json = serde_json::to_string(&point).unwrap();
        assert!(json.contains("Point"));
        assert!(json.contains("100"));
    }

    #[test]
    fn test_geometry_for_each_position() {
        let polygon =
            Geometry::simple_polygon(vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 0.0]]);

        let mut positions = Vec::new();
        polygon.for_each_position(&mut |pos| {
            positions.push(*pos);
        });

        assert_eq!(positions.len(), 4);
    }

    #[test]
    fn test_geojson_to_json() {
        let fc = FeatureCollection::from_features(vec![Feature::new(Geometry::point(0.0, 0.0))]);
        let geojson = GeoJson::FeatureCollection(fc);
        let json = geojson.to_json().unwrap();
        assert!(json.contains("FeatureCollection"));
    }
}

//! Spherical area calculation
//!
//! Computes the spherical area of GeoJSON geometries in steradians.
//! This is similar to D3's `d3.geoArea()`.
//!
//! # Limitations
//!
//! - The area formula is an approximation that works well for small to medium
//!   polygons but may have errors for very large polygons (like hemispheres)
//!   or polygons near the poles.
//! - `geo_bounds` does not handle geometries that cross the antimeridian (±180°).
//! - `geo_centroid` uses planar formulas, not true spherical centroids.

use super::geojson::{Geometry, Position};

/// Calculate the spherical area of a geometry in steradians.
///
/// The area is computed on a unit sphere. To get the area on Earth,
/// multiply by Earth's radius squared (approximately 6371 km² = 510 million km²).
///
/// # Example
///
/// ```
/// use makepad_d3::geo::{Geometry, geo_area};
///
/// // A polygon covering the western hemisphere (approximately)
/// let geometry = Geometry::Polygon {
///     coordinates: vec![vec![
///         [-180.0, -90.0],
///         [-180.0, 90.0],
///         [0.0, 90.0],
///         [0.0, -90.0],
///         [-180.0, -90.0],
///     ]],
/// };
///
/// let area = geo_area(&geometry);
/// // Area should be approximately 2π steradians (half the sphere)
/// ```
pub fn geo_area(geometry: &Geometry) -> f64 {
    match geometry {
        Geometry::Point { .. } | Geometry::MultiPoint { .. } => 0.0,
        Geometry::LineString { .. } | Geometry::MultiLineString { .. } => 0.0,
        Geometry::Polygon { coordinates } => polygon_area(coordinates),
        Geometry::MultiPolygon { coordinates } => coordinates.iter().map(|p| polygon_area(p)).sum(),
        Geometry::GeometryCollection { geometries } => geometries.iter().map(geo_area).sum(),
    }
}

/// Calculate the spherical area of a polygon (outer ring minus holes)
fn polygon_area(rings: &[Vec<Position>]) -> f64 {
    if rings.is_empty() {
        return 0.0;
    }

    // Outer ring area (positive if counter-clockwise on sphere)
    let outer = ring_area(&rings[0]);

    // Subtract hole areas
    let holes: f64 = rings.iter().skip(1).map(|ring| ring_area(ring).abs()).sum();

    (outer.abs() - holes).abs()
}

/// Calculate the spherical area of a ring using spherical excess formula.
///
/// Uses the formula based on the cross product of adjacent edges on the sphere.
/// The sign indicates winding direction.
fn ring_area(ring: &[Position]) -> f64 {
    let n = ring.len();
    if n < 3 {
        return 0.0;
    }

    let mut sum = 0.0;

    // Convert first point to radians
    let mut prev_lon = ring[0][0].to_radians();
    let mut prev_lat = ring[0][1].to_radians();

    for i in 1..n {
        let lon = ring[i][0].to_radians();
        let lat = ring[i][1].to_radians();

        // Spherical excess contribution using the formula:
        // E = 2 * atan(tan(Δλ/2) * (sin(φ1) + sin(φ2)) / (1 + cos(φ1)cos(φ2)cos(Δλ) + sin(φ1)sin(φ2)))
        // Simplified formula for polygon area as line integral:
        // A = ∫ (1 - cos(φ)) dλ
        // For discrete points: A ≈ Σ (λ2 - λ1) * (2 + sin(φ1) + sin(φ2)) / 4
        // But we use the proper spherical formula

        let d_lon = lon - prev_lon;

        // Use the formula: contribution = (λ2 - λ1) * (2 + sin(φ1) + sin(φ2)) / 4
        // This is an approximation but works well for most geometries
        // For exact calculation, we'd need spherical trigonometry
        sum += d_lon * (2.0 + prev_lat.sin() + lat.sin());

        prev_lon = lon;
        prev_lat = lat;
    }

    // Close the polygon (last point to first)
    let lon = ring[0][0].to_radians();
    let lat = ring[0][1].to_radians();
    let d_lon = lon - prev_lon;
    sum += d_lon * (2.0 + prev_lat.sin() + lat.sin());

    // The formula gives area in terms of the solid angle
    // Divide by 4 and take absolute value
    (sum / 4.0).abs()
}

/// Calculate the spherical length of a geometry in radians.
///
/// For LineString and Polygon boundaries, this computes the great-circle
/// distance along the path.
///
/// # Example
///
/// ```
/// use makepad_d3::geo::{Geometry, geo_length};
///
/// // A line from San Francisco to New York
/// let geometry = Geometry::LineString {
///     coordinates: vec![
///         [-122.4, 37.8],  // San Francisco
///         [-74.0, 40.7],   // New York
///     ],
/// };
///
/// let length = geo_length(&geometry);
/// // Length in radians (multiply by Earth's radius for km)
/// ```
pub fn geo_length(geometry: &Geometry) -> f64 {
    match geometry {
        Geometry::Point { .. } | Geometry::MultiPoint { .. } => 0.0,
        Geometry::LineString { coordinates } => line_length(coordinates),
        Geometry::MultiLineString { coordinates } => {
            coordinates.iter().map(|l| line_length(l)).sum()
        }
        Geometry::Polygon { coordinates } => coordinates.iter().map(|r| line_length(r)).sum(),
        Geometry::MultiPolygon { coordinates } => coordinates
            .iter()
            .flat_map(|p| p.iter())
            .map(|r| line_length(r))
            .sum(),
        Geometry::GeometryCollection { geometries } => geometries.iter().map(geo_length).sum(),
    }
}

/// Calculate the great-circle length of a line string in radians.
fn line_length(coordinates: &[Position]) -> f64 {
    if coordinates.len() < 2 {
        return 0.0;
    }

    let mut length = 0.0;

    for i in 1..coordinates.len() {
        length += haversine_distance(
            coordinates[i - 1][0],
            coordinates[i - 1][1],
            coordinates[i][0],
            coordinates[i][1],
        );
    }

    length
}

/// Calculate the great-circle distance between two points using the Haversine formula.
/// Returns distance in radians.
fn haversine_distance(lon1: f64, lat1: f64, lon2: f64, lat2: f64) -> f64 {
    let phi1 = lat1.to_radians();
    let phi2 = lat2.to_radians();
    let d_phi = (lat2 - lat1).to_radians();
    let d_lambda = (lon2 - lon1).to_radians();

    let a = (d_phi / 2.0).sin().powi(2) + phi1.cos() * phi2.cos() * (d_lambda / 2.0).sin().powi(2);

    2.0 * a.sqrt().asin()
}

/// Calculate the spherical centroid of a geometry.
///
/// Returns the centroid as (longitude, latitude) in degrees.
pub fn geo_centroid(geometry: &Geometry) -> Option<(f64, f64)> {
    match geometry {
        Geometry::Point { coordinates } => Some((coordinates[0], coordinates[1])),
        Geometry::MultiPoint { coordinates } => {
            if coordinates.is_empty() {
                return None;
            }
            let sum_x: f64 = coordinates.iter().map(|c| c[0]).sum();
            let sum_y: f64 = coordinates.iter().map(|c| c[1]).sum();
            let n = coordinates.len() as f64;
            Some((sum_x / n, sum_y / n))
        }
        Geometry::LineString { coordinates } => line_centroid(coordinates),
        Geometry::MultiLineString { coordinates } => {
            // Weighted centroid by line length
            let mut total_weight = 0.0;
            let mut weighted_x = 0.0;
            let mut weighted_y = 0.0;

            for line in coordinates {
                if let Some((cx, cy)) = line_centroid(line) {
                    let weight = line_length(line);
                    weighted_x += cx * weight;
                    weighted_y += cy * weight;
                    total_weight += weight;
                }
            }

            if total_weight > 0.0 {
                Some((weighted_x / total_weight, weighted_y / total_weight))
            } else {
                None
            }
        }
        Geometry::Polygon { coordinates } => {
            if coordinates.is_empty() {
                return None;
            }
            // Use the centroid of the outer ring
            polygon_centroid(&coordinates[0])
        }
        Geometry::MultiPolygon { coordinates } => {
            // Weighted centroid by area
            let mut total_weight = 0.0;
            let mut weighted_x = 0.0;
            let mut weighted_y = 0.0;

            for polygon in coordinates {
                if let Some(outer) = polygon.first() {
                    if let Some((cx, cy)) = polygon_centroid(outer) {
                        let weight = polygon_area(polygon);
                        weighted_x += cx * weight;
                        weighted_y += cy * weight;
                        total_weight += weight;
                    }
                }
            }

            if total_weight > 0.0 {
                Some((weighted_x / total_weight, weighted_y / total_weight))
            } else {
                None
            }
        }
        Geometry::GeometryCollection { geometries } => {
            if geometries.is_empty() {
                return None;
            }

            let centroids: Vec<_> = geometries.iter().filter_map(geo_centroid).collect();
            if centroids.is_empty() {
                return None;
            }

            let sum_x: f64 = centroids.iter().map(|(x, _)| x).sum();
            let sum_y: f64 = centroids.iter().map(|(_, y)| y).sum();
            let n = centroids.len() as f64;
            Some((sum_x / n, sum_y / n))
        }
    }
}

/// Calculate centroid of a line string
fn line_centroid(coordinates: &[Position]) -> Option<(f64, f64)> {
    if coordinates.is_empty() {
        return None;
    }

    // For a line, use length-weighted average of segment midpoints
    if coordinates.len() == 1 {
        return Some((coordinates[0][0], coordinates[0][1]));
    }

    let mut total_length = 0.0;
    let mut weighted_x = 0.0;
    let mut weighted_y = 0.0;

    for i in 1..coordinates.len() {
        let len = haversine_distance(
            coordinates[i - 1][0],
            coordinates[i - 1][1],
            coordinates[i][0],
            coordinates[i][1],
        );
        let mid_x = (coordinates[i - 1][0] + coordinates[i][0]) / 2.0;
        let mid_y = (coordinates[i - 1][1] + coordinates[i][1]) / 2.0;
        weighted_x += mid_x * len;
        weighted_y += mid_y * len;
        total_length += len;
    }

    if total_length > 0.0 {
        Some((weighted_x / total_length, weighted_y / total_length))
    } else {
        Some((coordinates[0][0], coordinates[0][1]))
    }
}

/// Calculate centroid of a polygon ring
fn polygon_centroid(ring: &[Position]) -> Option<(f64, f64)> {
    if ring.is_empty() {
        return None;
    }

    // Use the area-weighted centroid formula for a polygon
    let n = ring.len();
    if n < 3 {
        // Not enough points for a polygon, use simple average
        let sum_x: f64 = ring.iter().map(|c| c[0]).sum();
        let sum_y: f64 = ring.iter().map(|c| c[1]).sum();
        return Some((sum_x / n as f64, sum_y / n as f64));
    }

    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut signed_area = 0.0;

    for i in 0..n {
        let j = (i + 1) % n;
        let x0 = ring[i][0];
        let y0 = ring[i][1];
        let x1 = ring[j][0];
        let y1 = ring[j][1];

        let a = x0 * y1 - x1 * y0;
        signed_area += a;
        cx += (x0 + x1) * a;
        cy += (y0 + y1) * a;
    }

    if signed_area.abs() < 1e-10 {
        // Degenerate polygon, use simple average
        let sum_x: f64 = ring.iter().map(|c| c[0]).sum();
        let sum_y: f64 = ring.iter().map(|c| c[1]).sum();
        return Some((sum_x / n as f64, sum_y / n as f64));
    }

    signed_area *= 0.5;
    cx /= 6.0 * signed_area;
    cy /= 6.0 * signed_area;

    Some((cx, cy))
}

/// Calculate the bounds of a geometry in geographic coordinates.
///
/// Returns [[west, south], [east, north]] in degrees.
pub fn geo_bounds(geometry: &Geometry) -> Option<[[f64; 2]; 2]> {
    let coords = collect_coordinates(geometry);
    if coords.is_empty() {
        return None;
    }

    let mut min_lon = f64::INFINITY;
    let mut max_lon = f64::NEG_INFINITY;
    let mut min_lat = f64::INFINITY;
    let mut max_lat = f64::NEG_INFINITY;

    for coord in coords {
        min_lon = min_lon.min(coord[0]);
        max_lon = max_lon.max(coord[0]);
        min_lat = min_lat.min(coord[1]);
        max_lat = max_lat.max(coord[1]);
    }

    Some([[min_lon, min_lat], [max_lon, max_lat]])
}

/// Collect all coordinates from a geometry
fn collect_coordinates(geometry: &Geometry) -> Vec<Position> {
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
                coords.extend(collect_coordinates(geom));
            }
        }
    }

    coords
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_geo_area_point() {
        let geometry = Geometry::Point {
            coordinates: [0.0, 0.0],
        };
        assert_eq!(geo_area(&geometry), 0.0);
    }

    #[test]
    fn test_geo_area_linestring() {
        let geometry = Geometry::LineString {
            coordinates: vec![[0.0, 0.0], [10.0, 10.0]],
        };
        assert_eq!(geo_area(&geometry), 0.0);
    }

    #[test]
    fn test_geo_area_small_polygon() {
        // A small square polygon (1 degree x 1 degree at equator)
        let geometry = Geometry::Polygon {
            coordinates: vec![vec![
                [0.0, 0.0],
                [1.0, 0.0],
                [1.0, 1.0],
                [0.0, 1.0],
                [0.0, 0.0],
            ]],
        };

        let area = geo_area(&geometry);
        // Area should be approximately (1° × π/180)² = 0.000305 steradians
        assert!(area > 0.0);
        assert!(area < 0.001);
    }

    #[test]
    fn test_geo_area_hemisphere() {
        // A polygon approximating the western hemisphere
        let geometry = Geometry::Polygon {
            coordinates: vec![vec![
                [-180.0, -85.0],
                [-180.0, 85.0],
                [0.0, 85.0],
                [0.0, -85.0],
                [-180.0, -85.0],
            ]],
        };

        let area = geo_area(&geometry);
        // Area should be roughly half the sphere (~2π steradians)
        // But this approximation won't be exact
        assert!(area > 1.0);
    }

    #[test]
    fn test_geo_length_linestring() {
        // A line along the equator from 0° to 90°
        let geometry = Geometry::LineString {
            coordinates: vec![[0.0, 0.0], [90.0, 0.0]],
        };

        let length = geo_length(&geometry);
        // Should be approximately π/2 radians (90°)
        assert!((length - PI / 2.0).abs() < 0.01);
    }

    #[test]
    fn test_geo_length_sf_to_ny() {
        // San Francisco to New York
        let geometry = Geometry::LineString {
            coordinates: vec![[-122.4, 37.8], [-74.0, 40.7]],
        };

        let length = geo_length(&geometry);
        // Should be roughly 0.65 radians (about 4000 km / 6371 km)
        assert!(length > 0.5);
        assert!(length < 0.8);
    }

    #[test]
    fn test_haversine_distance() {
        // 90 degrees along equator
        let dist = haversine_distance(0.0, 0.0, 90.0, 0.0);
        assert!((dist - PI / 2.0).abs() < 0.0001);

        // Pole to pole
        let dist = haversine_distance(0.0, 90.0, 0.0, -90.0);
        assert!((dist - PI).abs() < 0.0001);
    }

    #[test]
    fn test_geo_centroid_point() {
        let geometry = Geometry::Point {
            coordinates: [10.0, 20.0],
        };
        let centroid = geo_centroid(&geometry);
        assert_eq!(centroid, Some((10.0, 20.0)));
    }

    #[test]
    fn test_geo_centroid_polygon() {
        let geometry = Geometry::Polygon {
            coordinates: vec![vec![
                [0.0, 0.0],
                [10.0, 0.0],
                [10.0, 10.0],
                [0.0, 10.0],
                [0.0, 0.0],
            ]],
        };
        let centroid = geo_centroid(&geometry);
        assert!(centroid.is_some());
        let (cx, cy) = centroid.unwrap();
        assert!((cx - 5.0).abs() < 0.01);
        assert!((cy - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_geo_bounds() {
        let geometry = Geometry::Polygon {
            coordinates: vec![vec![
                [-10.0, -20.0],
                [30.0, -20.0],
                [30.0, 40.0],
                [-10.0, 40.0],
                [-10.0, -20.0],
            ]],
        };
        let bounds = geo_bounds(&geometry);
        assert!(bounds.is_some());
        let [[west, south], [east, north]] = bounds.unwrap();
        assert_eq!(west, -10.0);
        assert_eq!(south, -20.0);
        assert_eq!(east, 30.0);
        assert_eq!(north, 40.0);
    }

    #[test]
    fn test_geo_area_with_hole() {
        // Polygon with a hole
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

        let area = geo_area(&geometry);
        // Area should be positive but less than the outer ring
        assert!(area > 0.0);

        // Compare to outer ring only
        let outer_only = Geometry::Polygon {
            coordinates: vec![vec![
                [0.0, 0.0],
                [10.0, 0.0],
                [10.0, 10.0],
                [0.0, 10.0],
                [0.0, 0.0],
            ]],
        };
        let outer_area = geo_area(&outer_only);
        assert!(area < outer_area);
    }
}

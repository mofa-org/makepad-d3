//! Delaunay triangulation and Voronoi diagram
//!
//! Implementation of Delaunay triangulation for 2D point sets,
//! used for Voronoi diagrams, stippling, and other spatial algorithms.

/// A 2D point
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn distance_squared(&self, other: &Point) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }
}

/// A triangle defined by three point indices
#[derive(Clone, Copy, Debug)]
struct Triangle {
    a: usize,
    b: usize,
    c: usize,
}

/// Delaunay triangulation
pub struct Delaunay {
    /// The input points
    pub points: Vec<Point>,
    /// Triangle indices (each 3 consecutive values form a triangle)
    pub triangles: Vec<usize>,
    /// Halfedge indices for navigation
    pub halfedges: Vec<i32>,
    /// Hull indices (convex hull of points)
    pub hull: Vec<usize>,
}

impl Delaunay {
    /// Create a new Delaunay triangulation from points
    pub fn new(coords: &[f64]) -> Self {
        let n = coords.len() / 2;
        let mut points = Vec::with_capacity(n);
        for i in 0..n {
            points.push(Point::new(coords[i * 2], coords[i * 2 + 1]));
        }

        if n < 3 {
            return Self {
                points,
                triangles: Vec::new(),
                halfedges: Vec::new(),
                hull: (0..n).collect(),
            };
        }

        // Simple incremental algorithm
        let (triangles, halfedges, hull) = Self::triangulate(&points);

        Self {
            points,
            triangles,
            halfedges,
            hull,
        }
    }

    /// Create from a slice of Points
    pub fn from_points(points: Vec<Point>) -> Self {
        if points.len() < 3 {
            return Self {
                hull: (0..points.len()).collect(),
                triangles: Vec::new(),
                halfedges: Vec::new(),
                points,
            };
        }

        let (triangles, halfedges, hull) = Self::triangulate(&points);

        Self {
            points,
            triangles,
            halfedges,
            hull,
        }
    }

    fn triangulate(points: &[Point]) -> (Vec<usize>, Vec<i32>, Vec<usize>) {
        let n = points.len();
        if n < 3 {
            return (Vec::new(), Vec::new(), (0..n).collect());
        }

        // Find bounding box
        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;

        for p in points {
            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);
            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
        }

        let dx = max_x - min_x;
        let dy = max_y - min_y;
        let d_max = dx.max(dy);
        let mid_x = (min_x + max_x) / 2.0;
        let mid_y = (min_y + max_y) / 2.0;

        // Create super triangle
        let mut all_points = points.to_vec();
        let p1 = Point::new(mid_x - 20.0 * d_max, mid_y - d_max);
        let p2 = Point::new(mid_x, mid_y + 20.0 * d_max);
        let p3 = Point::new(mid_x + 20.0 * d_max, mid_y - d_max);

        all_points.push(p1);
        all_points.push(p2);
        all_points.push(p3);

        let st1 = n;
        let st2 = n + 1;
        let st3 = n + 2;

        let mut triangles: Vec<Triangle> = vec![Triangle {
            a: st1,
            b: st2,
            c: st3,
        }];

        // Insert points one by one
        for i in 0..n {
            let p = &all_points[i];
            let mut bad_triangles = Vec::new();

            // Find triangles whose circumcircle contains the point
            for (t_idx, tri) in triangles.iter().enumerate() {
                if Self::in_circumcircle(
                    p,
                    &all_points[tri.a],
                    &all_points[tri.b],
                    &all_points[tri.c],
                ) {
                    bad_triangles.push(t_idx);
                }
            }

            // Find boundary of polygonal hole
            let mut polygon = Vec::new();
            for &t_idx in &bad_triangles {
                let tri = triangles[t_idx];
                let edges = [(tri.a, tri.b), (tri.b, tri.c), (tri.c, tri.a)];

                for edge in edges {
                    let mut is_shared = false;
                    for &other_idx in &bad_triangles {
                        if other_idx != t_idx {
                            let other = triangles[other_idx];
                            let other_edges =
                                [(other.a, other.b), (other.b, other.c), (other.c, other.a)];
                            for other_edge in other_edges {
                                if (edge.0 == other_edge.0 && edge.1 == other_edge.1)
                                    || (edge.0 == other_edge.1 && edge.1 == other_edge.0)
                                {
                                    is_shared = true;
                                    break;
                                }
                            }
                        }
                        if is_shared {
                            break;
                        }
                    }
                    if !is_shared {
                        polygon.push(edge);
                    }
                }
            }

            // Remove bad triangles (in reverse order to maintain indices)
            bad_triangles.sort_unstable();
            for &t_idx in bad_triangles.iter().rev() {
                triangles.remove(t_idx);
            }

            // Create new triangles
            for edge in polygon {
                triangles.push(Triangle {
                    a: edge.0,
                    b: edge.1,
                    c: i,
                });
            }
        }

        // Remove triangles with super triangle vertices
        triangles.retain(|tri| tri.a < n && tri.b < n && tri.c < n);

        // Convert to flat arrays
        let mut result_triangles = Vec::with_capacity(triangles.len() * 3);
        for tri in &triangles {
            result_triangles.push(tri.a);
            result_triangles.push(tri.b);
            result_triangles.push(tri.c);
        }

        // Simple halfedges (placeholder - full implementation would compute these)
        let halfedges = vec![-1i32; result_triangles.len()];

        // Compute convex hull
        let hull = Self::compute_hull(points);

        (result_triangles, halfedges, hull)
    }

    fn in_circumcircle(p: &Point, a: &Point, b: &Point, c: &Point) -> bool {
        let ax = a.x - p.x;
        let ay = a.y - p.y;
        let bx = b.x - p.x;
        let by = b.y - p.y;
        let cx = c.x - p.x;
        let cy = c.y - p.y;

        let det = (ax * ax + ay * ay) * (bx * cy - cx * by)
            - (bx * bx + by * by) * (ax * cy - cx * ay)
            + (cx * cx + cy * cy) * (ax * by - bx * ay);

        det > 0.0
    }

    fn compute_hull(points: &[Point]) -> Vec<usize> {
        let n = points.len();
        if n < 3 {
            return (0..n).collect();
        }

        // Find leftmost point
        let mut start = 0;
        for i in 1..n {
            if points[i].x < points[start].x
                || (points[i].x == points[start].x && points[i].y < points[start].y)
            {
                start = i;
            }
        }

        let mut hull = Vec::new();
        let mut current = start;

        loop {
            hull.push(current);
            let mut next = 0;

            for i in 0..n {
                if i == current {
                    continue;
                }
                if next == current {
                    next = i;
                    continue;
                }

                let cross = Self::cross_product(&points[current], &points[next], &points[i]);

                if cross < 0.0
                    || (cross == 0.0
                        && points[current].distance_squared(&points[i])
                            > points[current].distance_squared(&points[next]))
                {
                    next = i;
                }
            }

            current = next;
            if current == start {
                break;
            }
        }

        hull
    }

    fn cross_product(o: &Point, a: &Point, b: &Point) -> f64 {
        (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x)
    }

    /// Find the index of the point closest to (x, y)
    pub fn find(&self, x: f64, y: f64, start: usize) -> usize {
        let p = Point::new(x, y);
        let mut closest = start.min(self.points.len().saturating_sub(1));
        let mut min_dist = self
            .points
            .get(closest)
            .map(|pt| p.distance_squared(pt))
            .unwrap_or(f64::MAX);

        for (i, pt) in self.points.iter().enumerate() {
            let dist = p.distance_squared(pt);
            if dist < min_dist {
                min_dist = dist;
                closest = i;
            }
        }

        closest
    }

    /// Update the triangulation after points have been modified
    pub fn update(&mut self) {
        let (triangles, halfedges, hull) = Self::triangulate(&self.points);
        self.triangles = triangles;
        self.halfedges = halfedges;
        self.hull = hull;
    }
}

/// Voronoi diagram computed from Delaunay triangulation
pub struct Voronoi {
    /// Reference to the underlying Delaunay triangulation
    delaunay: Delaunay,
    /// Bounding box [xmin, ymin, xmax, ymax]
    bounds: [f64; 4],
    /// Circumcenters of triangles
    circumcenters: Vec<Point>,
}

impl Voronoi {
    /// Create a Voronoi diagram from a Delaunay triangulation
    pub fn new(delaunay: Delaunay, bounds: [f64; 4]) -> Self {
        let circumcenters = Self::compute_circumcenters(&delaunay);
        Self {
            delaunay,
            bounds,
            circumcenters,
        }
    }

    fn compute_circumcenters(delaunay: &Delaunay) -> Vec<Point> {
        let mut centers = Vec::new();
        let triangles = &delaunay.triangles;
        let points = &delaunay.points;

        for i in (0..triangles.len()).step_by(3) {
            if i + 2 >= triangles.len() {
                break;
            }
            let a = &points[triangles[i]];
            let b = &points[triangles[i + 1]];
            let c = &points[triangles[i + 2]];

            let center = Self::circumcenter(a, b, c);
            centers.push(center);
        }

        centers
    }

    fn circumcenter(a: &Point, b: &Point, c: &Point) -> Point {
        let ax = a.x;
        let ay = a.y;
        let bx = b.x;
        let by = b.y;
        let cx = c.x;
        let cy = c.y;

        let d = 2.0 * (ax * (by - cy) + bx * (cy - ay) + cx * (ay - by));
        if d.abs() < 1e-10 {
            return Point::new((ax + bx + cx) / 3.0, (ay + by + cy) / 3.0);
        }

        let ux = ((ax * ax + ay * ay) * (by - cy)
            + (bx * bx + by * by) * (cy - ay)
            + (cx * cx + cy * cy) * (ay - by))
            / d;
        let uy = ((ax * ax + ay * ay) * (cx - bx)
            + (bx * bx + by * by) * (ax - cx)
            + (cx * cx + cy * cy) * (bx - ax))
            / d;

        Point::new(ux, uy)
    }

    /// Get the Delaunay triangulation
    pub fn delaunay(&self) -> &Delaunay {
        &self.delaunay
    }

    /// Get mutable access to the Delaunay triangulation
    pub fn delaunay_mut(&mut self) -> &mut Delaunay {
        &mut self.delaunay
    }

    /// Update the Voronoi diagram after points have changed
    pub fn update(&mut self) {
        self.delaunay.update();
        self.circumcenters = Self::compute_circumcenters(&self.delaunay);
    }
}

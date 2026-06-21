//! Plane geometry in the planner's coordinate system (units = feet).
//!
//! Ports of the spike helpers `area` (shoelace), `pin` (point-in-polygon ray
//! cast), and `plen` (polyline length), but typed and unit-tested.

use serde::{Deserialize, Serialize};

/// A 2D point in feet. The yard's south-west corner is the origin; `+x` runs
/// east, `+y` runs north (matching the spikes' convention).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    #[must_use]
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

impl From<(f64, f64)> for Point {
    fn from((x, y): (f64, f64)) -> Self {
        Self { x, y }
    }
}

/// Area of a simple polygon (ft²) via the shoelace formula. The result is the
/// absolute area, so winding order does not matter. Fewer than three vertices
/// encloses no area and returns `0.0`.
#[must_use]
pub fn area(pts: &[Point]) -> f64 {
    if pts.len() < 3 {
        return 0.0;
    }
    let n = pts.len();
    let mut sum = 0.0;
    for i in 0..n {
        let a = pts[i];
        let b = pts[(i + 1) % n];
        sum += a.x * b.y - b.x * a.y;
    }
    sum.abs() / 2.0
}

/// Total length of an open polyline (ft) — the sum of its segment lengths.
#[must_use]
pub fn polyline_length(pts: &[Point]) -> f64 {
    pts.windows(2).map(|w| w[0].dist(w[1])).sum()
}

impl Point {
    #[must_use]
    pub fn dist(self, other: Self) -> f64 {
        (self.x - other.x).hypot(self.y - other.y)
    }
}

/// Whether `p` lies inside the polygon `pts`, by the even-odd ray-cast rule
/// (port of the spike's `pin`). Behavior exactly on an edge or vertex is
/// unspecified, as with the original.
#[must_use]
pub fn point_in_polygon(p: Point, pts: &[Point]) -> bool {
    let n = pts.len();
    if n < 3 {
        return false;
    }
    let mut inside = false;
    let mut j = n - 1;
    for i in 0..n {
        let (xi, yi) = (pts[i].x, pts[i].y);
        let (xj, yj) = (pts[j].x, pts[j].y);
        if ((yi > p.y) != (yj > p.y)) && (p.x < (xj - xi) * (p.y - yi) / (yj - yi) + xi) {
            inside = !inside;
        }
        j = i;
    }
    inside
}

#[cfg(test)]
mod tests {
    use super::*;

    fn poly(coords: &[(f64, f64)]) -> Vec<Point> {
        coords.iter().map(|&c| c.into()).collect()
    }

    #[test]
    fn area_of_unit_square_is_one() {
        let sq = poly(&[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]);
        assert!((area(&sq) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn area_is_orientation_independent() {
        let cw = poly(&[(0.0, 0.0), (0.0, 2.0), (3.0, 2.0), (3.0, 0.0)]);
        let ccw = poly(&[(0.0, 0.0), (3.0, 0.0), (3.0, 2.0), (0.0, 2.0)]);
        assert!((area(&cw) - 6.0).abs() < 1e-9);
        assert!((area(&ccw) - 6.0).abs() < 1e-9);
    }

    #[test]
    fn degenerate_polygons_have_no_area() {
        assert!(area(&[]).abs() < 1e-12);
        assert!(area(&poly(&[(0.0, 0.0), (1.0, 1.0)])).abs() < 1e-12);
    }

    #[test]
    fn polyline_length_sums_segments() {
        let path = poly(&[(0.0, 0.0), (3.0, 0.0), (3.0, 4.0)]);
        assert!((polyline_length(&path) - 7.0).abs() < 1e-9);
    }

    #[test]
    fn point_in_polygon_inside_and_outside() {
        let sq = poly(&[(0.0, 0.0), (4.0, 0.0), (4.0, 4.0), (0.0, 4.0)]);
        assert!(point_in_polygon(Point::new(2.0, 2.0), &sq));
        assert!(!point_in_polygon(Point::new(5.0, 2.0), &sq));
        assert!(!point_in_polygon(Point::new(-1.0, 2.0), &sq));
    }

    #[test]
    fn point_in_concave_polygon() {
        // An L-shape: the notch at (3,3) must read as outside.
        let l = poly(&[
            (0.0, 0.0),
            (4.0, 0.0),
            (4.0, 2.0),
            (2.0, 2.0),
            (2.0, 4.0),
            (0.0, 4.0),
        ]);
        assert!(point_in_polygon(Point::new(1.0, 3.0), &l));
        assert!(!point_in_polygon(Point::new(3.0, 3.0), &l));
    }
}

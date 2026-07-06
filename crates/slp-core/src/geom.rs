//! Plane geometry in the planner's coordinate system (units = feet):
//! polygon area (shoelace), point-in-polygon (ray cast), and polyline length —
//! typed and unit-tested.

use serde::{Deserialize, Serialize};

/// A 2D point in feet. The yard's south-west corner is the origin; `+x` runs
/// east, `+y` runs north.
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

/// Area of a circle (ft²) of the given radius — `πr²`. A non-positive radius
/// encloses no area and returns `0.0`.
#[must_use]
pub fn circle_area(radius_ft: f64) -> f64 {
    if radius_ft <= 0.0 {
        return 0.0;
    }
    std::f64::consts::PI * radius_ft * radius_ft
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

/// Whether `p` lies inside the polygon `pts`, by the even-odd ray-cast rule.
/// Behavior exactly on an edge or vertex is unspecified.
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

/// The four corners (world feet) of a `w` × `d` footprint centered at `(cx, cy)`
/// and rotated `rot_deg` **clockwise from north** — the same sense the 2D render
/// uses, so containment matches what the user sees. Corners are returned
/// counter-clockwise starting from the local south-west.
#[must_use]
pub fn footprint_corners(cx: f64, cy: f64, w: f64, d: f64, rot_deg: f64) -> [Point; 4] {
    let (hw, hd) = (w / 2.0, d / 2.0);
    let (s, c) = rot_deg.to_radians().sin_cos();
    // Clockwise-from-north rotation (north toward east): a local offset
    // (lx east, ly north) maps to (lx·cos + ly·sin, -lx·sin + ly·cos).
    let corner = |lx: f64, ly: f64| {
        Point::new(
            lx.mul_add(c, ly.mul_add(s, cx)),
            (-lx).mul_add(s, ly.mul_add(c, cy)),
        )
    };
    [
        corner(-hw, -hd),
        corner(hw, -hd),
        corner(hw, hd),
        corner(-hw, hd),
    ]
}

/// The heading from `center` to `to`, in degrees **clockwise from north** —
/// north is 0°, east 90°, south 180°, west 270°. Matches the rotation sense of
/// [`footprint_corners`], so dragging a handle to a point and setting `rot` to
/// this heading turns the object's north edge toward that point. `center == to`
/// yields 0°.
#[must_use]
pub fn heading(center: Point, to: Point) -> f64 {
    let east = to.x - center.x;
    let north = to.y - center.y;
    east.atan2(north).to_degrees().rem_euclid(360.0)
}

/// Whether every point of `pts` lies inside one and the same polygon among
/// `surfaces` — i.e. the footprint is fully contained in a *single* area.
/// A footprint that touches/crosses a boundary, sits off every surface, or
/// spans two surfaces is **not** contained.
#[must_use]
pub fn within_a_single(pts: &[Point], surfaces: &[Vec<Point>]) -> bool {
    surfaces
        .iter()
        .any(|s| pts.iter().all(|p| point_in_polygon(*p, s)))
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
    fn circle_area_is_pi_r_squared() {
        assert!((circle_area(1.0) - std::f64::consts::PI).abs() < 1e-9);
        assert!((circle_area(2.0) - 4.0 * std::f64::consts::PI).abs() < 1e-9);
    }

    #[test]
    fn a_nonpositive_radius_has_no_area() {
        assert!(circle_area(0.0).abs() < 1e-12);
        assert!(circle_area(-3.0).abs() < 1e-12);
    }

    #[test]
    fn area_of_a_general_triangle() {
        // A slanted, off-origin triangle (no zero cross-terms) pins the shoelace
        // sum: 0.5·|1(2−6) + 5(6−1) + 2(1−2)| = 0.5·19 = 9.5. Three vertices, so
        // it also exercises the `len < 3` guard at the boundary.
        let tri = poly(&[(1.0, 1.0), (5.0, 2.0), (2.0, 6.0)]);
        assert!((area(&tri) - 9.5).abs() < 1e-9);
    }

    #[test]
    fn polyline_length_sums_segments() {
        let path = poly(&[(0.0, 0.0), (3.0, 0.0), (3.0, 4.0)]);
        assert!((polyline_length(&path) - 7.0).abs() < 1e-9);
    }

    #[test]
    fn dist_between_offset_points() {
        // Off-origin and asymmetric in both axes so each `-` matters:
        // hypot(1−4, 2−6) = hypot(-3, -4) = 5.
        assert!((Point::new(1.0, 2.0).dist(Point::new(4.0, 6.0)) - 5.0).abs() < 1e-9);
    }

    #[test]
    fn point_in_polygon_inside_and_outside() {
        let sq = poly(&[(0.0, 0.0), (4.0, 0.0), (4.0, 4.0), (0.0, 4.0)]);
        assert!(point_in_polygon(Point::new(2.0, 2.0), &sq));
        assert!(!point_in_polygon(Point::new(5.0, 2.0), &sq));
        assert!(!point_in_polygon(Point::new(-1.0, 2.0), &sq));
    }

    #[test]
    fn point_in_polygon_with_slanted_edges() {
        // Triangle with two slanted edges so the x-intercept arithmetic
        // (xj−xi)·(p.y−yi)/(yj−yi)+xi actually matters. At y=2 the left edge is
        // at x=1 and the right edge at x=3; at y=3 they are at x=1.5 and x=2.5.
        let tri = poly(&[(0.0, 0.0), (4.0, 0.0), (2.0, 4.0)]);
        for inside in [(2.0, 1.0), (2.5, 2.0), (1.5, 2.0), (2.0, 3.0)] {
            assert!(
                point_in_polygon(Point::new(inside.0, inside.1), &tri),
                "{inside:?}"
            );
        }
        for outside in [
            (3.5, 2.0),
            (0.5, 2.0),
            (3.0, 3.0),
            (5.0, 2.0),
            (2.0, -1.0),
            (2.0, 5.0),
        ] {
            assert!(
                !point_in_polygon(Point::new(outside.0, outside.1), &tri),
                "{outside:?}"
            );
        }
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

    #[test]
    fn footprint_corners_at_zero_rotation() {
        // A 4 (E-W) × 2 (N-S) box centered at (10,10): SW, SE, NE, NW.
        let c = footprint_corners(10.0, 10.0, 4.0, 2.0, 0.0);
        let expect = [(8.0, 9.0), (12.0, 9.0), (12.0, 11.0), (8.0, 11.0)];
        for (got, (ex, ey)) in c.iter().zip(expect) {
            assert!(
                (got.x - ex).abs() < 1e-9 && (got.y - ey).abs() < 1e-9,
                "{got:?}"
            );
        }
    }

    #[test]
    fn footprint_corners_rotate_clockwise() {
        // A unit square at the origin rotated 45° clockwise becomes a diamond:
        // the south-west corner swings to due west at distance √2. This pins both
        // the sin and cos terms of the rotation.
        let c = footprint_corners(0.0, 0.0, 2.0, 2.0, 45.0);
        let root2 = std::f64::consts::SQRT_2;
        assert!(
            (c[0].x + root2).abs() < 1e-9 && c[0].y.abs() < 1e-9,
            "SW → due west: {:?}",
            c[0]
        );
    }

    #[test]
    fn heading_is_clockwise_from_north() {
        let c = Point::new(2.0, 3.0);
        // N, E, S, W of the center → 0, 90, 180, 270.
        assert!(
            (heading(c, Point::new(2.0, 9.0)) - 0.0).abs() < 1e-9,
            "north"
        );
        assert!(
            (heading(c, Point::new(9.0, 3.0)) - 90.0).abs() < 1e-9,
            "east"
        );
        assert!(
            (heading(c, Point::new(2.0, -1.0)) - 180.0).abs() < 1e-9,
            "south"
        );
        assert!(
            (heading(c, Point::new(-1.0, 3.0)) - 270.0).abs() < 1e-9,
            "west"
        );
        // A degenerate (center == to) is 0, not NaN.
        assert!((heading(c, c) - 0.0).abs() < 1e-9, "self → 0");
    }

    #[test]
    fn footprint_within_a_single_surface() {
        let deck = poly(&[(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)]);
        let surfaces = std::slice::from_ref(&deck);
        // A 2×2 footprint well inside the deck is contained.
        let inside = footprint_corners(5.0, 5.0, 2.0, 2.0, 0.0);
        assert!(within_a_single(&inside, surfaces));
        // Shoved into the corner so it overhangs (a corner lands outside) — not.
        let over = footprint_corners(0.5, 0.5, 2.0, 2.0, 0.0);
        assert!(!within_a_single(&over, surfaces));
        // No surfaces → never contained.
        assert!(!within_a_single(&inside, &[]));
    }

    #[test]
    fn footprint_spanning_two_surfaces_is_not_within_a_single() {
        // Two abutting decks; a footprint straddling the shared edge sits fully
        // in neither one alone, so it is not contained in a single surface.
        let left = poly(&[(0.0, 0.0), (5.0, 0.0), (5.0, 10.0), (0.0, 10.0)]);
        let right = poly(&[(5.0, 0.0), (10.0, 0.0), (10.0, 10.0), (5.0, 10.0)]);
        let straddle = footprint_corners(5.0, 5.0, 2.0, 2.0, 0.0);
        assert!(!within_a_single(&straddle, &[left, right]));
    }
}

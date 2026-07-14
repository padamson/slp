//! Distance geometry for keep-clear zones: how far is a point / a segment from
//! a polygon (a rectangle, in practice)? These answer "is X within `clearance`
//! of this footprint?" for a rectangular grill's shape-following clearance zone,
//! the way [`crate::clearance`]'s circle overlaps answer it for a round
//! footprint. Pure and headless; `slp-ui` resolves footprints and calls these
//! per pair.

use crate::geom::{Point, point_in_polygon};

/// Distance from `p` to the closest point on segment `a`–`b`. A degenerate
/// segment (`a == b`) reduces to the point-to-point distance.
#[must_use]
pub fn dist_point_to_segment(p: Point, a: Point, b: Point) -> f64 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let len2 = dx.mul_add(dx, dy * dy);
    let closest = if len2 <= 0.0 {
        a
    } else {
        let t = ((p.x - a.x).mul_add(dx, (p.y - a.y) * dy) / len2).clamp(0.0, 1.0);
        Point::new(dx.mul_add(t, a.x), dy.mul_add(t, a.y))
    };
    p.dist(closest)
}

/// The signed area of triangle `a`,`b`,`c` × 2 — positive when `c` is left of
/// the directed line `a`→`b`, zero when the three are colinear.
fn orient(a: Point, b: Point, c: Point) -> f64 {
    (b.x - a.x).mul_add(c.y - a.y, -((b.y - a.y) * (c.x - a.x)))
}

/// Whether segments `p1`–`p2` and `q1`–`q2` cross at an interior point (each
/// segment straddles the other's line). A touch at an endpoint or a colinear
/// overlap is deliberately *not* reported here: in that case an endpoint of one
/// segment lies on the other, so [`dist_segment_to_segment`]'s endpoint-distance
/// fallback already yields `0` without a separate colinear test — keeping this
/// to a single straddle check.
fn segments_cross(p1: Point, p2: Point, q1: Point, q2: Point) -> bool {
    let d1 = orient(q1, q2, p1);
    let d2 = orient(q1, q2, p2);
    let d3 = orient(p1, p2, q1);
    let d4 = orient(p1, p2, q2);
    ((d1 > 0.0) != (d2 > 0.0)) && ((d3 > 0.0) != (d4 > 0.0))
}

/// Distance between segments `p1`–`p2` and `q1`–`q2` — `0.0` when they
/// intersect (cross, touch, or overlap), else the smallest endpoint-to-segment
/// distance.
#[must_use]
pub fn dist_segment_to_segment(p1: Point, p2: Point, q1: Point, q2: Point) -> f64 {
    if segments_cross(p1, p2, q1, q2) {
        return 0.0;
    }
    dist_point_to_segment(p1, q1, q2)
        .min(dist_point_to_segment(p2, q1, q2))
        .min(dist_point_to_segment(q1, p1, p2))
        .min(dist_point_to_segment(q2, p1, p2))
}

/// Distance from `p` to a closed polygon: `0.0` when `p` is inside, else the
/// distance to the nearest edge. Fewer than three vertices returns [`f64::MAX`]
/// (there's no polygon to be near).
#[must_use]
pub fn dist_point_to_polygon(p: Point, poly: &[Point]) -> f64 {
    let n = poly.len();
    if n < 3 {
        return f64::MAX;
    }
    if point_in_polygon(p, poly) {
        return 0.0;
    }
    (0..n)
        .map(|i| dist_point_to_segment(p, poly[i], poly[(i + 1) % n]))
        .fold(f64::MAX, f64::min)
}

/// Distance from segment `a`–`b` to a closed polygon: `0.0` when the segment
/// touches or enters the polygon, else the distance to the nearest edge. Fewer
/// than three vertices returns [`f64::MAX`].
#[must_use]
pub fn dist_segment_to_polygon(a: Point, b: Point, poly: &[Point]) -> f64 {
    let n = poly.len();
    if n < 3 {
        return f64::MAX;
    }
    if point_in_polygon(a, poly) || point_in_polygon(b, poly) {
        return 0.0;
    }
    (0..n)
        .map(|i| dist_segment_to_segment(a, b, poly[i], poly[(i + 1) % n]))
        .fold(f64::MAX, f64::min)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(x: f64, y: f64) -> Point {
        Point::new(x, y)
    }

    /// A 2×2 square centered at the origin.
    fn unit_square() -> Vec<Point> {
        vec![p(-1.0, -1.0), p(1.0, -1.0), p(1.0, 1.0), p(-1.0, 1.0)]
    }

    #[test]
    fn point_to_segment_projects_onto_the_interior() {
        // (0,3) drops straight down to (0,0) on the x-axis segment.
        assert!((dist_point_to_segment(p(0.0, 3.0), p(-5.0, 0.0), p(5.0, 0.0)) - 3.0).abs() < 1e-9);
    }

    #[test]
    fn point_to_segment_clamps_past_an_endpoint() {
        // (10,0) is nearest the (5,0) endpoint, not the infinite line.
        assert!(
            (dist_point_to_segment(p(10.0, 0.0), p(-5.0, 0.0), p(5.0, 0.0)) - 5.0).abs() < 1e-9
        );
    }

    #[test]
    fn point_to_degenerate_segment_is_point_distance() {
        assert!((dist_point_to_segment(p(3.0, 4.0), p(0.0, 0.0), p(0.0, 0.0)) - 5.0).abs() < 1e-9);
    }

    #[test]
    fn point_projects_onto_a_diagonal_segments_interior() {
        // A slanted segment (both dx and dy non-zero), so the projection uses the
        // real squared length. (-1,5.5) drops perpendicularly onto the midpoint
        // (2,1.5) of (0,0)-(4,3): distance hypot(3,4) = 5.
        assert!((dist_point_to_segment(p(-1.0, 5.5), p(0.0, 0.0), p(4.0, 3.0)) - 5.0).abs() < 1e-9);
    }

    #[test]
    fn crossing_segments_are_zero_distance() {
        assert!(
            dist_segment_to_segment(p(-1.0, 0.0), p(1.0, 0.0), p(0.0, -1.0), p(0.0, 1.0)) < 1e-12
        );
    }

    #[test]
    fn touching_at_an_endpoint_is_zero_distance() {
        assert!(
            dist_segment_to_segment(p(0.0, 0.0), p(1.0, 0.0), p(1.0, 0.0), p(1.0, 1.0)) < 1e-12
        );
    }

    #[test]
    fn asymmetric_crossings_are_zero_distance() {
        // Two off-centre crossings with opposite orientation-sign patterns, so
        // every orientation term is exercised as both positive and negative —
        // a broken `orient` (or crossing test) fails to detect the crossing and
        // returns a non-zero endpoint distance.
        assert!(
            dist_segment_to_segment(p(0.0, 0.0), p(4.0, 1.0), p(1.0, 2.0), p(2.0, -2.0)) < 1e-12
        );
        assert!(
            dist_segment_to_segment(p(0.0, 0.0), p(4.0, 1.0), p(1.0, -2.0), p(2.0, 2.0)) < 1e-12
        );
        // A diagonal × anti-diagonal X — its orientation terms flip sign under a
        // cross-product that adds instead of subtracts, so this pins the sign of
        // `orient` itself.
        assert!(
            dist_segment_to_segment(p(0.0, 0.0), p(4.0, 4.0), p(0.0, 4.0), p(4.0, 0.0)) < 1e-12
        );
    }

    #[test]
    fn parallel_segments_measure_the_gap() {
        // Two horizontal segments 2 apart.
        let d = dist_segment_to_segment(p(0.0, 0.0), p(4.0, 0.0), p(0.0, 2.0), p(4.0, 2.0));
        assert!((d - 2.0).abs() < 1e-9);
    }

    #[test]
    fn skew_segments_measure_the_nearest_endpoints() {
        // Nearest approach is endpoint (2,0) to endpoint (2,3): distance 3.
        let d = dist_segment_to_segment(p(0.0, 0.0), p(2.0, 0.0), p(2.0, 3.0), p(5.0, 3.0));
        assert!((d - 3.0).abs() < 1e-9);
    }

    #[test]
    fn point_inside_a_polygon_is_zero() {
        assert!(dist_point_to_polygon(p(0.0, 0.0), &unit_square()) < 1e-12);
    }

    #[test]
    fn point_outside_measures_to_the_nearest_edge() {
        // (3,0) is 2 east of the square's right edge at x=1.
        assert!((dist_point_to_polygon(p(3.0, 0.0), &unit_square()) - 2.0).abs() < 1e-9);
    }

    #[test]
    fn point_off_a_corner_measures_to_the_corner() {
        // (4,5) nearest the (1,1) corner: distance = hypot(3,4) = 5.
        assert!((dist_point_to_polygon(p(4.0, 5.0), &unit_square()) - 5.0).abs() < 1e-9);
    }

    #[test]
    fn a_degenerate_polygon_is_infinitely_far() {
        assert!(dist_point_to_polygon(p(0.0, 0.0), &[p(0.0, 0.0), p(1.0, 0.0)]) > 1e300);
    }

    #[test]
    fn a_triangle_is_a_real_polygon() {
        // Exactly three vertices is a polygon, not "too few" — (2,-2) is 2 below
        // the triangle's base at y=0.
        let tri = vec![p(0.0, 0.0), p(4.0, 0.0), p(2.0, 3.0)];
        assert!((dist_point_to_polygon(p(2.0, -2.0), &tri) - 2.0).abs() < 1e-9);
        assert!((dist_segment_to_polygon(p(0.0, -2.0), p(4.0, -2.0), &tri) - 2.0).abs() < 1e-9);
    }

    #[test]
    fn a_segment_off_a_long_edge_uses_that_edge_not_a_vertex() {
        // A 10×1 rectangle: a short segment centred above the middle of the long
        // top edge is nearest that edge's interior (gap 2), not any corner — so
        // every polygon edge (including the one between the last two vertices)
        // must be walked, correctly wrapping the last→first index.
        let long_thin = vec![p(0.0, 0.0), p(10.0, 0.0), p(10.0, 1.0), p(0.0, 1.0)];
        let d = dist_segment_to_polygon(p(3.0, 3.0), p(7.0, 3.0), &long_thin);
        assert!(
            (d - 2.0).abs() < 1e-9,
            "gap to the top edge's middle, got {d}"
        );
    }

    #[test]
    fn segment_entering_a_polygon_is_zero() {
        // A segment from outside into the square.
        assert!(dist_segment_to_polygon(p(-5.0, 0.0), p(0.0, 0.0), &unit_square()) < 1e-12);
    }

    #[test]
    fn segment_passing_beside_a_polygon_measures_the_gap() {
        // A horizontal segment at y=3, above the square (top edge y=1): gap 2.
        let d = dist_segment_to_polygon(p(-5.0, 3.0), p(5.0, 3.0), &unit_square());
        assert!((d - 2.0).abs() < 1e-9);
    }

    #[test]
    fn segment_crossing_a_polygon_edge_is_zero() {
        // Endpoints both outside, but the segment passes through the square.
        assert!(dist_segment_to_polygon(p(-5.0, 0.0), p(5.0, 0.0), &unit_square()) < 1e-12);
    }
}

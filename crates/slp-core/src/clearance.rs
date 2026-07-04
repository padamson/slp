//! Circle-overlap geometry for a keep-clear zone (a fire pit's safety
//! clearance): does a disk of a given radius intersect another circle, a line
//! segment (a structure edge — a house wall, a deck edge), or a polygon (an
//! object's rectangular footprint)? Pure and headless, like the rest of
//! `geom.rs` — `slp-ui` resolves catalog footprints and calls these per pair.

use crate::geom::Point;

/// Whether two circles' disks overlap (including touching).
#[must_use]
pub fn circle_overlaps_circle(c1: Point, r1: f64, c2: Point, r2: f64) -> bool {
    c1.dist(c2) <= r1 + r2
}

/// Whether a circle's disk overlaps the segment `a`–`b` — true when the
/// closest point on the segment to `center` is within `radius`. A degenerate
/// segment (`a == b`) reduces to a point check.
#[must_use]
pub fn circle_overlaps_segment(center: Point, radius: f64, a: Point, b: Point) -> bool {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let len2 = dx.mul_add(dx, dy * dy);
    let closest = if len2 <= 0.0 {
        a
    } else {
        let t = ((center.x - a.x).mul_add(dx, (center.y - a.y) * dy) / len2).clamp(0.0, 1.0);
        Point::new(dx.mul_add(t, a.x), dy.mul_add(t, a.y))
    };
    center.dist(closest) <= radius
}

/// Whether a circle's disk overlaps the interior of a closed polygon (e.g. an
/// object's rectangular footprint, or a house/deck outline) — true if the
/// circle's center is inside the polygon, or any polygon edge comes within
/// `radius` of the center. Fewer than three vertices never overlaps.
#[must_use]
pub fn circle_overlaps_polygon(center: Point, radius: f64, polygon: &[Point]) -> bool {
    let n = polygon.len();
    if n < 3 {
        return false;
    }
    crate::geom::point_in_polygon(center, polygon)
        || (0..n).any(|i| circle_overlaps_segment(center, radius, polygon[i], polygon[(i + 1) % n]))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn poly(coords: &[(f64, f64)]) -> Vec<Point> {
        coords.iter().map(|&c| c.into()).collect()
    }

    #[test]
    fn circles_overlap_when_closer_than_the_radius_sum() {
        assert!(circle_overlaps_circle(
            Point::new(0.0, 0.0),
            3.0,
            Point::new(5.0, 0.0),
            3.0
        ));
    }

    #[test]
    fn circles_touching_exactly_count_as_overlapping() {
        assert!(circle_overlaps_circle(
            Point::new(0.0, 0.0),
            3.0,
            Point::new(6.0, 0.0),
            3.0
        ));
    }

    #[test]
    fn circles_farther_than_the_radius_sum_do_not_overlap() {
        assert!(!circle_overlaps_circle(
            Point::new(0.0, 0.0),
            3.0,
            Point::new(6.01, 0.0),
            3.0
        ));
    }

    #[test]
    fn segment_overlap_uses_the_perpendicular_distance_when_the_foot_is_on_the_segment() {
        // Segment along the x-axis from (0,0) to (10,0); center (5, 2) is
        // perpendicular to the segment's midpoint at distance 2.
        let (a, b) = (Point::new(0.0, 0.0), Point::new(10.0, 0.0));
        assert!(circle_overlaps_segment(Point::new(5.0, 2.0), 2.0, a, b));
        assert!(!circle_overlaps_segment(Point::new(5.0, 2.01), 2.0, a, b));
    }

    #[test]
    fn segment_overlap_projection_uses_the_offset_from_the_start_point() {
        // A diagonal, off-origin segment (a.x, a.y, and dy all nonzero) with
        // the center exactly on its midpoint: the projection's exact zero
        // distance only holds if both `center - a` subtractions are genuine
        // subtractions — flipping either to addition sends the projected
        // point far from the segment, at a distance well past a tiny radius.
        let (a, b) = (Point::new(1.0, 2.0), Point::new(5.0, 6.0));
        assert!(circle_overlaps_segment(Point::new(3.0, 4.0), 0.05, a, b));
    }

    #[test]
    fn segment_overlap_clamps_to_the_nearest_endpoint_beyond_the_segment() {
        // Center is past the segment's end (10,0); nearest point is the
        // endpoint, not the infinite-line projection.
        let (a, b) = (Point::new(0.0, 0.0), Point::new(10.0, 0.0));
        assert!(circle_overlaps_segment(Point::new(12.0, 0.0), 2.0, a, b));
        assert!(!circle_overlaps_segment(Point::new(12.01, 0.0), 2.0, a, b));
    }

    #[test]
    fn degenerate_segment_reduces_to_a_point_check() {
        let p = Point::new(3.0, 3.0);
        assert!(circle_overlaps_segment(Point::new(3.0, 5.0), 2.0, p, p));
        assert!(!circle_overlaps_segment(Point::new(3.0, 5.01), 2.0, p, p));
    }

    #[test]
    fn circle_centered_inside_a_large_polygon_overlaps_even_far_from_every_edge() {
        let square = poly(&[(0.0, 0.0), (100.0, 0.0), (100.0, 100.0), (0.0, 100.0)]);
        assert!(circle_overlaps_polygon(
            Point::new(50.0, 50.0),
            1.0,
            &square
        ));
    }

    #[test]
    fn circle_outside_and_far_from_a_polygon_does_not_overlap() {
        let square = poly(&[(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)]);
        assert!(!circle_overlaps_polygon(
            Point::new(50.0, 50.0),
            3.0,
            &square
        ));
    }

    #[test]
    fn circle_overlapping_only_one_edge_of_a_polygon_overlaps() {
        // A 10x10 square; a circle just outside the right edge (x=10) at
        // (12, 5), radius 3, reaches the edge (distance 2 < 3).
        let square = poly(&[(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)]);
        assert!(circle_overlaps_polygon(Point::new(12.0, 5.0), 3.0, &square));
        assert!(!circle_overlaps_polygon(
            Point::new(13.01, 5.0),
            3.0,
            &square
        ));
    }

    #[test]
    fn a_triangle_the_smallest_real_polygon_can_overlap() {
        // Exactly 3 vertices — the boundary between "too few to be a polygon"
        // (n < 3) and a real one. If the guard were `n <= 3`, a triangle would
        // never be checked and this would incorrectly report no overlap.
        let tri = poly(&[(0.0, 0.0), (10.0, 0.0), (5.0, 8.0)]);
        assert!(circle_overlaps_polygon(Point::new(5.0, 4.0), 1.0, &tri));
    }

    #[test]
    fn degenerate_polygon_never_overlaps() {
        assert!(!circle_overlaps_polygon(
            Point::new(0.0, 0.0),
            100.0,
            &poly(&[(0.0, 0.0), (1.0, 1.0)])
        ));
    }
}

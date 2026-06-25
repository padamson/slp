//! Wall geometry: a wall is the edge between two consecutive house corners, and
//! a door/window sits on it at an `offset` (feet from the wall's start) spanning
//! `width` feet. Pure and headless so the renderer and take-off math share it.

use crate::Coord;

/// The point `dist` feet from `start` toward `end`, clamped to the segment.
/// A degenerate (zero-length) wall returns `start`.
#[must_use]
pub fn point_along(start: &Coord, end: &Coord, dist: f64) -> Coord {
    let (dx, dy) = (end.x - start.x, end.y - start.y);
    let len = dx.hypot(dy);
    if len <= 0.0 {
        return start.clone();
    }
    let d = dist.clamp(0.0, len);
    Coord::new(start.x + dx / len * d, start.y + dy / len * d)
}

/// The two endpoints of an opening on the wall `start`→`end`: from `offset` to
/// `offset + width` feet along it (both clamped to the wall).
#[must_use]
pub fn opening_segment(start: &Coord, end: &Coord, offset: f64, width: f64) -> (Coord, Coord) {
    (
        point_along(start, end, offset),
        point_along(start, end, offset + width),
    )
}

/// Project `p` onto the wall segment `start`→`end`: returns the offset in feet
/// from `start` to the nearest point on the segment (clamped to its length) and
/// the perpendicular distance from `p` to that point.
#[must_use]
pub fn project_onto(start: &Coord, end: &Coord, p: &Coord) -> (f64, f64) {
    let (dx, dy) = (end.x - start.x, end.y - start.y);
    let len2 = dx.mul_add(dx, dy * dy);
    if len2 <= 0.0 {
        return (0.0, (p.x - start.x).hypot(p.y - start.y));
    }
    let t = ((p.x - start.x).mul_add(dx, (p.y - start.y) * dy) / len2).clamp(0.0, 1.0);
    let foot = Coord::new(dx.mul_add(t, start.x), dy.mul_add(t, start.y));
    (t * len2.sqrt(), (p.x - foot.x).hypot(p.y - foot.y))
}

/// The wall (edge of the closed ring of `corners`) nearest to `p`: returns the
/// wall index, the offset in feet along it, and the distance. `None` if there
/// aren't enough corners to form walls.
#[must_use]
pub fn nearest_wall(corners: &[Coord], p: &Coord) -> Option<(usize, f64, f64)> {
    let n = corners.len();
    if n < 3 {
        return None;
    }
    let mut best: Option<(usize, f64, f64)> = None;
    for i in 0..n {
        let (offset, dist) = project_onto(&corners[i], &corners[(i + 1) % n], p);
        if best.is_none_or(|(_, _, bd)| dist < bd) {
            best = Some((i, offset, dist));
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_along_walks_the_segment() {
        let (a, b) = (Coord::new(0.0, 0.0), Coord::new(10.0, 0.0));
        assert_eq!(point_along(&a, &b, 3.0), Coord::new(3.0, 0.0));
        assert_eq!(point_along(&a, &b, 0.0), Coord::new(0.0, 0.0));
        // Vertical wall.
        assert_eq!(
            point_along(&Coord::new(2.0, 1.0), &Coord::new(2.0, 9.0), 4.0),
            Coord::new(2.0, 5.0)
        );
    }

    #[test]
    fn point_along_clamps_past_the_ends() {
        let (a, b) = (Coord::new(0.0, 0.0), Coord::new(10.0, 0.0));
        assert_eq!(point_along(&a, &b, 99.0), Coord::new(10.0, 0.0));
        assert_eq!(point_along(&a, &b, -5.0), Coord::new(0.0, 0.0));
    }

    #[test]
    fn degenerate_wall_returns_start() {
        let a = Coord::new(4.0, 4.0);
        assert_eq!(point_along(&a, &a, 2.0), a);
    }

    #[test]
    fn opening_segment_spans_offset_to_offset_plus_width() {
        let (a, b) = (Coord::new(0.0, 0.0), Coord::new(20.0, 0.0));
        let (p, q) = opening_segment(&a, &b, 5.0, 3.0);
        assert_eq!(p, Coord::new(5.0, 0.0));
        assert_eq!(q, Coord::new(8.0, 0.0));
    }

    #[test]
    fn project_onto_gives_offset_and_perpendicular_distance() {
        let (a, b) = (Coord::new(0.0, 0.0), Coord::new(10.0, 0.0));
        let (offset, dist) = project_onto(&a, &b, &Coord::new(4.0, 2.0));
        assert!((offset - 4.0).abs() < 1e-9);
        assert!((dist - 2.0).abs() < 1e-9);
        // Past the end clamps the offset to the wall length.
        let (offset, _) = project_onto(&a, &b, &Coord::new(99.0, 1.0));
        assert!((offset - 10.0).abs() < 1e-9);
    }

    fn square() -> Vec<Coord> {
        vec![
            Coord::new(0.0, 0.0),
            Coord::new(10.0, 0.0),
            Coord::new(10.0, 10.0),
            Coord::new(0.0, 10.0),
        ]
    }

    #[test]
    fn nearest_wall_picks_the_closest_edge() {
        let sq = square();
        // Just inside the bottom edge (edge 0): offset 5, small distance.
        let (wall, offset, dist) = nearest_wall(&sq, &Coord::new(5.0, 1.0)).unwrap();
        assert_eq!(wall, 0);
        assert!((offset - 5.0).abs() < 1e-9);
        assert!((dist - 1.0).abs() < 1e-9);
        // Near the right edge (edge 1).
        let (wall, _, _) = nearest_wall(&sq, &Coord::new(9.0, 6.0)).unwrap();
        assert_eq!(wall, 1);
    }

    #[test]
    fn nearest_wall_needs_a_ring() {
        let two = vec![Coord::new(0.0, 0.0), Coord::new(10.0, 0.0)];
        assert!(nearest_wall(&two, &Coord::new(5.0, 1.0)).is_none());
    }
}

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
}

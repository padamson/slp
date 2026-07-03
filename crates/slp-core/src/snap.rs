//! Snapping helpers for drawing corners — pure and headless so every area tool
//! (house, pavers, beds, deck) can reuse them. Two independent transforms:
//! snap a point to a grid step, and constrain an edge to be axis-aligned
//! (parallel to the grid). Units are feet.

use crate::Coord;

/// Round a point to the nearest multiple of `step` (feet) on each axis. A
/// non-positive `step` is a no-op.
#[must_use]
pub fn snap_to_grid(p: &Coord, step: f64) -> Coord {
    if step <= 0.0 {
        return p.clone();
    }
    Coord::new((p.x / step).round() * step, (p.y / step).round() * step)
}

/// New center for a dragged object: the cursor plus the grab offset `(gx, gy)` —
/// the vector from the cursor to the object's center captured when the drag
/// began, so the object tracks the cursor without jumping its center under the
/// pointer — snapped to the grid when `step > 0` (a non-positive `step` leaves
/// the position free).
#[must_use]
pub fn dragged_center(cursor: &Coord, grab: (f64, f64), step: f64) -> Coord {
    let (gx, gy) = grab;
    snap_to_grid(&Coord::new(cursor.x + gx, cursor.y + gy), step)
}

/// Constrain the edge from `prev` to `p` to be horizontal or vertical —
/// whichever is closer — so walls and walkways stay parallel to the grid. The
/// larger of the two deltas wins (ties go horizontal).
#[must_use]
pub fn snap_ortho(prev: &Coord, p: &Coord) -> Coord {
    if (p.x - prev.x).abs() >= (p.y - prev.y).abs() {
        Coord::new(p.x, prev.y) // horizontal edge: keep prev's y
    } else {
        Coord::new(prev.x, p.y) // vertical edge: keep prev's x
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_snaps_to_nearest_step() {
        assert_eq!(
            snap_to_grid(&Coord::new(1.4, 2.6), 1.0),
            Coord::new(1.0, 3.0)
        );
        assert_eq!(
            snap_to_grid(&Coord::new(1.4, 2.6), 0.5),
            Coord::new(1.5, 2.5)
        );
        assert_eq!(
            snap_to_grid(&Coord::new(-0.4, -0.6), 1.0),
            Coord::new(0.0, -1.0)
        );
    }

    #[test]
    fn grid_step_zero_or_negative_is_noop() {
        let p = Coord::new(1.4, 2.6);
        assert_eq!(snap_to_grid(&p, 0.0), p);
        assert_eq!(snap_to_grid(&p, -1.0), p);
    }

    #[test]
    fn dragged_center_applies_the_grab_offset() {
        // Free (step 0): center = cursor + grab, each axis using its own offset.
        assert_eq!(
            dragged_center(&Coord::new(10.0, 20.0), (2.0, -3.0), 0.0),
            Coord::new(12.0, 17.0)
        );
    }

    #[test]
    fn dragged_center_snaps_to_the_grid_when_step_positive() {
        let cursor = Coord::new(10.4, 20.6);
        // step > 0 rounds the offset center to the grid.
        assert_eq!(
            dragged_center(&cursor, (0.0, 0.0), 1.0),
            Coord::new(10.0, 21.0)
        );
        // step 0 leaves it free — the grab-offset center, unrounded.
        assert_eq!(dragged_center(&cursor, (0.0, 0.0), 0.0), cursor);
    }

    #[test]
    fn ortho_aligns_to_the_dominant_axis() {
        let o = Coord::new(0.0, 0.0);
        // Mostly horizontal → snap y back to prev.
        assert_eq!(snap_ortho(&o, &Coord::new(5.0, 1.0)), Coord::new(5.0, 0.0));
        // Mostly vertical → snap x back to prev.
        assert_eq!(snap_ortho(&o, &Coord::new(1.0, 5.0)), Coord::new(0.0, 5.0));
        // Tie → horizontal.
        assert_eq!(snap_ortho(&o, &Coord::new(3.0, 3.0)), Coord::new(3.0, 0.0));
    }

    #[test]
    fn ortho_uses_offset_from_a_nonzero_prev() {
        // Off-origin `prev` so each delta is `p - prev` (not `p + prev`): the
        // dominant axis is decided by the offset from prev, and the result keeps
        // prev's other coordinate.
        // dx=2, dy=3 → vertical → keep prev.x=3.
        assert_eq!(
            snap_ortho(&Coord::new(3.0, 1.0), &Coord::new(1.0, 4.0)),
            Coord::new(3.0, 4.0)
        );
        // dx=3, dy=2 → horizontal → keep prev.y=3.
        assert_eq!(
            snap_ortho(&Coord::new(1.0, 3.0), &Coord::new(4.0, 1.0)),
            Coord::new(4.0, 3.0)
        );
    }
}

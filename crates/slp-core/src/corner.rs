//! Choosing an empty yard corner to float the object-inspector window in, so it
//! doesn't cover the plan. Corners are checked in priority order and the first
//! with no content in it wins.

use crate::Point;

/// A corner of the yard. `Nw` is the north-west (top-left, since north is up).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Corner {
    Nw,
    Sw,
    Ne,
    Se,
}

/// The first corner (priority NE → NW → SE → SW) whose `region_w` × `region_h`
/// (feet) box holds none of `points` — the content the window should avoid
/// (house/deck/object vertices). Falls back to `Ne` when every corner is
/// occupied.
#[must_use]
pub fn free_corner(
    points: &[Point],
    yard_w: f64,
    yard_d: f64,
    region_w: f64,
    region_h: f64,
) -> Corner {
    // A corner box (an x-span × a y-span) is occupied if any content point falls
    // within it.
    let occupied = |(x0, x1): (f64, f64), (y0, y1): (f64, f64)| {
        points
            .iter()
            .any(|p| p.x >= x0 && p.x <= x1 && p.y >= y0 && p.y <= y1)
    };
    let (west, east) = ((0.0, region_w), (yard_w - region_w, yard_w));
    let (south, north) = ((0.0, region_h), (yard_d - region_h, yard_d));

    // Corners in priority order; the first whose box is empty wins, else NE.
    [
        (Corner::Ne, east, north),
        (Corner::Nw, west, north),
        (Corner::Se, east, south),
        (Corner::Sw, west, south),
    ]
    .into_iter()
    .find(|&(_, xs, ys)| !occupied(xs, ys))
    .map_or(Corner::Ne, |(corner, ..)| corner)
}

#[cfg(test)]
mod tests {
    use super::*;

    // 40×20 ft yard, 10×6 ft window regions.
    const W: f64 = 40.0;
    const D: f64 = 20.0;
    const RW: f64 = 10.0;
    const RH: f64 = 6.0;

    fn free(points: &[Point]) -> Corner {
        free_corner(points, W, D, RW, RH)
    }

    #[test]
    fn empty_yard_uses_the_ne_corner() {
        assert_eq!(free(&[]), Corner::Ne);
    }

    #[test]
    fn skips_occupied_corners_in_priority_order() {
        // Priority NE → NW → SE → SW. Something in the NE box (x>30, y>14) → NW.
        assert_eq!(free(&[Point::new(38.0, 18.0)]), Corner::Nw);
        // NE + NW occupied → SE.
        assert_eq!(
            free(&[Point::new(38.0, 18.0), Point::new(2.0, 18.0)]),
            Corner::Se
        );
        // NE + NW + SE occupied → SW.
        assert_eq!(
            free(&[
                Point::new(38.0, 18.0),
                Point::new(2.0, 18.0),
                Point::new(38.0, 2.0),
            ]),
            Corner::Sw
        );
    }

    #[test]
    fn all_corners_occupied_falls_back_to_ne() {
        let corners = [
            Point::new(38.0, 18.0), // NE
            Point::new(2.0, 18.0),  // NW
            Point::new(38.0, 2.0),  // SE
            Point::new(2.0, 2.0),   // SW
        ];
        assert_eq!(free(&corners), Corner::Ne);
    }

    #[test]
    fn content_in_the_middle_leaves_ne_free() {
        // A point mid-yard sits in no corner box → NE (the default) stays free.
        assert_eq!(free(&[Point::new(20.0, 10.0)]), Corner::Ne);
    }

    #[test]
    fn region_bounds_are_offsets_from_the_far_edge() {
        // The east band is [yard_w - region_w, yard_w] = [30, 40]. A mid-east
        // point at x=20 is NOT in it, so NE stays free — pins `yard_w - region_w`
        // (a wrong op like `/` → [4,40] would swallow x=20, occupy NE, give NW).
        assert_eq!(free(&[Point::new(20.0, 18.0)]), Corner::Ne);
        // The north band is [yard_d - region_h, yard_d] = [14, 20]. An east-band
        // point at y=10 sits below it, so NE stays free — pins `yard_d - region_h`
        // (a wrong op → [~3.3, 20] would swallow y=10, occupy NE, give NW).
        assert_eq!(free(&[Point::new(38.0, 10.0)]), Corner::Ne);
    }
}

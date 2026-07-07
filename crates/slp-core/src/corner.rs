//! Choosing an empty yard corner to float the object-inspector window in, so it
//! doesn't cover the plan. Corners are checked in priority order and the first
//! with no content in it wins.

use crate::{Circle, Coord, DeckLevel, Object, Point, Shape};

/// Every plan-content point an inspector window should dodge when choosing a
/// corner (see [`free_corner`]) — the house outline, each deck level's corners,
/// object centers, and each drawn area: a shape's vertices, and a circle's
/// center plus its four extent points so a large circle still registers in the
/// corner it covers. Gathering *all* placed/drawn content in one place is what
/// keeps the inspectors from floating over anything on the plan.
#[must_use]
pub fn content_points(
    house: &[Coord],
    deck: &[DeckLevel],
    objects: &[Object],
    shapes: &[Shape],
    circles: &[Circle],
) -> Vec<Point> {
    let pt = |c: &Coord| Point::new(c.x, c.y);
    let mut points: Vec<Point> = house.iter().map(pt).collect();
    points.extend(deck.iter().flat_map(|l| l.corners.iter().map(pt)));
    points.extend(objects.iter().map(|o| Point::new(o.x, o.y)));
    points.extend(shapes.iter().flat_map(|s| s.corners.iter().map(pt)));
    for c in circles {
        let (cx, cy, r) = (c.center.x, c.center.y, c.radius_ft);
        points.push(Point::new(cx, cy));
        points.push(Point::new(cx + r, cy));
        points.push(Point::new(cx - r, cy));
        points.push(Point::new(cx, cy + r));
        points.push(Point::new(cx, cy - r));
    }
    points
}

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

    fn shape_with(corners: &[(f64, f64)]) -> Shape {
        let mut s = Shape::new(0.0);
        s.corners = corners.iter().map(|&(x, y)| Coord::new(x, y)).collect();
        s
    }

    fn level_with(corners: &[(f64, f64)]) -> DeckLevel {
        let mut l = DeckLevel::new(0.0);
        l.corners = corners.iter().map(|&(x, y)| Coord::new(x, y)).collect();
        l
    }

    #[test]
    fn content_points_gathers_every_placed_and_drawn_feature() {
        let house = [Coord::new(1.0, 1.0), Coord::new(2.0, 2.0)];
        let deck = [level_with(&[(3.0, 3.0)])];
        let objects = [Object::new("chair".into(), 4.0, 5.0)];
        let shapes = [shape_with(&[(6.0, 7.0), (8.0, 9.0)])];
        let circles: [Circle; 0] = [];

        let pts = content_points(&house, &deck, &objects, &shapes, &circles);

        // House (2) + deck corner (1) + object center (1) + shape verts (2) = 6.
        assert!(pts.contains(&Point::new(1.0, 1.0)), "house corner");
        assert!(pts.contains(&Point::new(3.0, 3.0)), "deck level corner");
        assert!(pts.contains(&Point::new(4.0, 5.0)), "object center");
        assert!(pts.contains(&Point::new(6.0, 7.0)), "shape vertex");
        assert!(pts.contains(&Point::new(8.0, 9.0)), "shape vertex");
        assert_eq!(pts.len(), 6);
    }

    #[test]
    fn a_circle_contributes_its_center_and_four_extent_points() {
        let circle = Circle::new(Box::new(Coord::new(10.0, 20.0)), 0.0, 3.0);
        let pts = content_points(&[], &[], &[], &[], &[circle]);
        // Center + N/S/E/W of the rim — so a big circle registers in the corner
        // it fills even when its center sits outside the corner box.
        assert!(pts.contains(&Point::new(10.0, 20.0)), "center");
        assert!(pts.contains(&Point::new(13.0, 20.0)), "east rim");
        assert!(pts.contains(&Point::new(7.0, 20.0)), "west rim");
        assert!(pts.contains(&Point::new(10.0, 23.0)), "north rim");
        assert!(pts.contains(&Point::new(10.0, 17.0)), "south rim");
        assert_eq!(pts.len(), 5);
    }

    #[test]
    fn no_content_yields_no_points() {
        assert!(content_points(&[], &[], &[], &[], &[]).is_empty());
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

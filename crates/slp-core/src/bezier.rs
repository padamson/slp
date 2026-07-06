//! Cubic-Bézier boundary edges: a drawn area's edge can be a smooth curve
//! (an S-curve or free-form bed edge) instead of a straight chord or a
//! circular arc. An edge's curve is a cubic Bézier from its start node `p0`
//! to its end node `p3` with two control points `c1`, `c2` — the SVG/vector
//! convention (`C c1 c2 p3` in a path).
//!
//! The only headless computation needed here is the **signed area** the curve
//! adds to (or removes from) the straight-chord polygon, so
//! `slp_core::arc::boundary_area` reports a mixed straight/arc/curve boundary's
//! true enclosed area. Rendering (the `C` path command) and the interactive
//! control handles live in the UI; the SVG points are just the control points
//! in screen space, so no math is needed for them.

use crate::Point;

/// The 2D cross product `a × b = a.x·b.y − b.x·a.y` — twice the signed area of
/// the triangle `(origin, a, b)`.
fn cross(a: Point, b: Point) -> f64 {
    a.x * b.y - b.x * a.y
}

/// The signed area between the cubic Bézier `p0`→`p3` (control points `c1`,
/// `c2`) and its straight chord `p0`→`p3` — i.e. the extra area the curve
/// contributes beyond the chord that a shoelace sum already counts. Add this
/// to the polygon's signed shoelace area (see `arc::boundary_area`), the same
/// way a circular arc's segment area is added.
///
/// Uses the closed form of the Green's-theorem line integral
/// `½∮(x dy − y dx)` over a cubic Bézier, minus the chord's own `½(p0 × p3)`.
/// A curve whose control points lie on the chord (at its thirds) is a straight
/// line and contributes `0`.
#[must_use]
pub fn bezier_segment_area(p0: Point, c1: Point, c2: Point, p3: Point) -> f64 {
    // Twice the signed area swept by the curve (∫₀¹ (x y' − y x') dt), as a
    // weighted sum of the control-point cross products — the standard cubic
    // coefficients (3/5, 3/10, 1/10 on the pairs, symmetric end to end).
    let twice_curve = 0.6 * cross(p0, c1)
        + 0.3 * cross(p0, c2)
        + 0.1 * cross(p0, p3)
        + 0.3 * cross(c1, c2)
        + 0.3 * cross(c1, p3)
        + 0.6 * cross(c2, p3);
    // ½·(curve integral) − ½·(chord integral) = area between curve and chord.
    (twice_curve - cross(p0, p3)) / 2.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    /// Control points at the chord's thirds make the cubic a straight line.
    fn third(a: Point, b: Point, t: f64) -> Point {
        Point::new(a.x + t * (b.x - a.x), a.y + t * (b.y - a.y))
    }

    #[test]
    fn a_straight_cubic_contributes_no_area() {
        // Controls on the chord at 1/3 and 2/3 → the curve *is* the chord.
        let (p0, p3) = (Point::new(1.0, 2.0), Point::new(5.0, 4.0));
        let c1 = third(p0, p3, 1.0 / 3.0);
        let c2 = third(p0, p3, 2.0 / 3.0);
        assert!(approx(bezier_segment_area(p0, c1, c2, p3), 0.0));
    }

    #[test]
    fn a_symmetric_bump_has_the_expected_area_and_sign() {
        // p0=(0,0)→p3=(4,0), controls pulled up to y=3: the curve bows up,
        // enclosing a positive area between it and the chord (a parabola-ish
        // cap of base 4 peaking near 2.25 → ≈6.3 ft²). The exact closed form
        // is 6.3.
        let area = bezier_segment_area(
            Point::new(0.0, 0.0),
            Point::new(1.0, 3.0),
            Point::new(3.0, 3.0),
            Point::new(4.0, 0.0),
        );
        assert!(approx(area, -6.3), "got {area}");
    }

    #[test]
    fn reversing_the_curve_negates_its_area() {
        // Traversing the same curve the other way (swap endpoints and controls)
        // flips the sign, so a whole-boundary reversal flips everything and the
        // reported (abs) area is unchanged.
        let (p0, p3) = (Point::new(0.0, 0.0), Point::new(4.0, 0.0));
        let (c1, c2) = (Point::new(1.0, 3.0), Point::new(3.0, 3.0));
        let forward = bezier_segment_area(p0, c1, c2, p3);
        let backward = bezier_segment_area(p3, c2, c1, p0);
        assert!(approx(forward, -backward), "{forward} vs {backward}");
    }

    #[test]
    fn an_s_curve_with_balanced_lobes_nets_to_zero() {
        // Controls pulled to opposite sides by equal amounts about the chord
        // midpoint → the two lobes cancel, net area 0.
        let area = bezier_segment_area(
            Point::new(0.0, 0.0),
            Point::new(1.0, 2.0),
            Point::new(3.0, -2.0),
            Point::new(4.0, 0.0),
        );
        assert!(approx(area, 0.0), "got {area}");
    }
}

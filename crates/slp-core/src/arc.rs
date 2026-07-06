//! Circular-arc boundary edges via the CAD/DXF **bulge** convention, so a
//! drawn area's edge can bow into an arc instead of a straight chord.
//!
//! A bulge `b` on an edge is `tan(θ/4)`, where `θ` is the arc's signed
//! subtended (included) angle. `b == 0` is a straight line; `|b| == 1` is a
//! semicircle; `|b| > 1` is a major arc. The sign picks the side the arc bows
//! to: **positive bulges to the left** of the edge's travel direction (a +90°
//! / counter-clockwise turn from `a`→`b`), negative to the right.
//!
//! Two things are derived from a bulge, both pure and headless:
//! 1. [`segment_area`] — the signed area the arc adds to (or removes from) the
//!    straight-chord polygon, so [`boundary_area`] reports a mixed
//!    straight/arc boundary's true enclosed area.
//! 2. [`arc_svg`] — the radius + SVG `A`-command flags to render the arc.

use crate::{Coord, Point, area, bezier_segment_area};

/// A circular arc's SVG `A`-command parameters (radius in the caller's units,
/// plus the two boolean flags). The endpoint is the caller's own `b` point;
/// this only carries what the bulge determines.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArcSvg {
    /// The arc's circle radius, in the same units as the `chord_len` passed in.
    pub radius: f64,
    /// SVG `large-arc-flag`: the arc subtends more than a semicircle (`|b|>1`).
    pub large_arc: bool,
    /// SVG `sweep-flag`.
    pub sweep: bool,
}

/// Bulges within this of zero are treated as a straight edge (no arc).
const STRAIGHT_EPS: f64 = 1e-9;

/// The arc's circle radius for a chord of length `chord_len` and bulge `b`.
/// `r = c·(1 + b²) / (4·|b|)`. Returns `None` for a straight (≈0) bulge.
#[must_use]
pub fn bulge_radius(chord_len: f64, bulge: f64) -> Option<f64> {
    if bulge.abs() < STRAIGHT_EPS {
        return None;
    }
    Some(chord_len * (1.0 + bulge * bulge) / (4.0 * bulge.abs()))
}

/// SVG `A`-command parameters to render the arc of bulge `b` across a chord of
/// screen length `chord_len` (px). `None` for a straight edge — draw a line.
///
/// `sweep = b > 0` and `large_arc = |b| > 1`: derived for SVG's y-down user
/// space when the endpoints are passed in screen coordinates and the stored
/// bulge follows this module's world (y-up) "positive = left" convention — the
/// world→screen y-flip and the flip between "bulges left" and SVG's sweep
/// direction cancel, so no sign juggling is needed at the call site.
#[must_use]
pub fn arc_svg(chord_len: f64, bulge: f64) -> Option<ArcSvg> {
    bulge_radius(chord_len, bulge).map(|radius| ArcSvg {
        radius,
        large_arc: bulge.abs() > 1.0,
        sweep: bulge > 0.0,
    })
}

/// The signed area the arc of bulge `b` contributes across a chord of length
/// `chord_len`, added to the straight-chord polygon area. Positive bulge
/// (bows left of travel) bows *into* the interior of a counter-clockwise
/// boundary, so it **removes** area — hence the leading minus. `0.0` for a
/// straight edge.
#[must_use]
pub fn segment_area(chord_len: f64, bulge: f64) -> f64 {
    let Some(radius) = bulge_radius(chord_len, bulge) else {
        return 0.0;
    };
    let theta = 4.0 * bulge.atan(); // signed subtended angle
    -radius * radius * (theta - theta.sin()) / 2.0
}

/// The area (ft²) enclosed by a closed boundary whose edges may be straight,
/// circular **arcs**, or cubic-Bézier **curves**. `corners` are the ring's
/// nodes; `bulges[i]` is the arc bulge for edge `corners[i]`→`corners[i+1]`
/// (0/absent = not an arc); `curves` is a sparse list of `(edge_index, c1, c2)`
/// for the Bézier edges. A curve takes precedence over a bulge on the same
/// edge. A short/empty `bulges` and an empty `curves` mean all-straight.
///
/// Fewer than three nodes encloses no polygon area on its own; the arc/curve
/// terms are still added, so a two-node ring of opposing semicircles reads as
/// its circle's area.
#[must_use]
pub fn boundary_area(corners: &[Coord], bulges: &[f64], curves: &[(usize, Point, Point)]) -> f64 {
    let n = corners.len();
    if n < 2 {
        return 0.0;
    }
    let pts: Vec<Point> = corners.iter().map(|c| Point::new(c.x, c.y)).collect();
    // Signed shoelace of the straight-chord polygon (positive for a
    // counter-clockwise ring), plus each curved edge's signed correction
    // beyond its chord (Bézier if the edge has controls, else an arc segment).
    let mut signed = signed_shoelace(&pts);
    for i in 0..n {
        let (p0, p3) = (pts[i], pts[(i + 1) % n]);
        if let Some(&(_, c1, c2)) = curves.iter().find(|&&(e, _, _)| e == i) {
            signed += bezier_segment_area(p0, c1, c2, p3);
        } else {
            let bulge = bulges.get(i).copied().unwrap_or(0.0);
            signed += segment_area(p0.dist(p3), bulge);
        }
    }
    signed.abs()
}

/// Signed polygon area (positive counter-clockwise) via the shoelace sum —
/// unlike [`area`], which returns the absolute value. Kept private since the
/// arc corrections need the winding sign to combine correctly; the public
/// [`boundary_area`] takes the absolute value at the end.
fn signed_shoelace(pts: &[Point]) -> f64 {
    if pts.len() < 3 {
        // A degenerate straight polygon encloses nothing; the caller's arc
        // terms still apply (see `boundary_area`). `area` agrees on this.
        debug_assert!(area(pts).abs() < 1e-12);
        return 0.0;
    }
    let n = pts.len();
    let mut sum = 0.0;
    for i in 0..n {
        let a = pts[i];
        let b = pts[(i + 1) % n];
        sum += a.x * b.y - b.x * a.y;
    }
    sum / 2.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn a_straight_bulge_has_no_radius_no_svg_no_area() {
        assert_eq!(bulge_radius(10.0, 0.0), None);
        assert_eq!(arc_svg(10.0, 0.0), None);
        assert!(approx(segment_area(10.0, 0.0), 0.0));
    }

    #[test]
    fn a_semicircle_bulge_is_one_and_its_radius_is_half_the_chord() {
        // |b| = 1 → θ = π (semicircle); r = c(1+1)/(4·1) = c/2.
        assert!(approx(bulge_radius(10.0, 1.0).unwrap(), 5.0));
        assert!(approx(bulge_radius(10.0, -1.0).unwrap(), 5.0));
    }

    #[test]
    fn radius_at_bulges_other_than_one_pins_each_multiplication() {
        // b = 1 makes `b·b` and `c·(…)`/`(…)·|b|` coincide with their divide
        // mutants; distinct bulges separate them. r = c(1+b²)/(4|b|).
        // b=0.5: 10·1.25/2 = 6.25.
        assert!(approx(bulge_radius(10.0, 0.5).unwrap(), 6.25));
        // b=2 (a major arc): 8·5/8 = 5.
        assert!(approx(bulge_radius(8.0, 2.0).unwrap(), 5.0));
    }

    #[test]
    fn a_bulge_exactly_at_the_straightness_epsilon_is_still_an_arc() {
        // The guard is `< eps` (strict), so a bulge of exactly the epsilon is
        // an arc, not straight — pins `<` against `<=`.
        assert!(bulge_radius(10.0, 1e-9).is_some());
    }

    #[test]
    fn arc_svg_flags_track_the_bulge_magnitude_and_sign() {
        // Minor arc, bows left → small-arc, sweep on.
        let minor = arc_svg(10.0, 0.5).unwrap();
        assert!(!minor.large_arc && minor.sweep);
        // Minor arc, bows right → sweep off.
        let right = arc_svg(10.0, -0.5).unwrap();
        assert!(!right.large_arc && !right.sweep);
        // Major arc (|b| > 1) → large-arc set.
        assert!(arc_svg(10.0, 2.0).unwrap().large_arc);
        assert!(arc_svg(10.0, -2.0).unwrap().large_arc);
        // Exactly a semicircle is NOT a major arc.
        assert!(!arc_svg(10.0, 1.0).unwrap().large_arc);
    }

    #[test]
    fn a_semicircle_segment_area_is_half_its_disk() {
        // Chord 10 → r 5; a semicircle segment is half the disk = π·25/2.
        // Sign: positive bulge removes area (bows into a CCW interior).
        assert!(approx(segment_area(10.0, 1.0), -PI * 25.0 / 2.0));
        // Negative bulge (bows the other way) adds the same magnitude.
        assert!(approx(segment_area(10.0, -1.0), PI * 25.0 / 2.0));
    }

    #[test]
    fn a_quarter_arc_segment_matches_the_geometric_area() {
        // A 90° arc: θ = π/2, bulge = tan(π/8); for radius 1 the chord is √2.
        // Its circular segment area is (r²/2)(θ − sinθ) = (π/2 − 1)/2 — a
        // ground truth (not the code's own formula) that pins `θ − sinθ`
        // against `θ + sinθ` (which a semicircle, sin π = 0, can't tell apart).
        let chord = 2.0_f64.sqrt();
        let bulge = (PI / 8.0).tan();
        let expected = -(PI / 2.0 - 1.0) / 2.0;
        assert!(
            approx(segment_area(chord, bulge), expected),
            "got {}, want {expected}",
            segment_area(chord, bulge)
        );
    }

    #[test]
    fn reversing_an_edge_negates_its_segment_area() {
        // The same physical arc, traversed the other way, has bulge -b and the
        // opposite signed contribution — so a whole-boundary reversal flips the
        // sign of everything and the reported (abs) area is unchanged.
        let c = 7.0;
        assert!(approx(segment_area(c, 0.6), -segment_area(c, -0.6)));
    }

    #[test]
    fn two_opposing_semicircles_enclose_their_circle() {
        // Nodes at (-r, 0) and (r, 0); both edges are semicircles (bulge 1)
        // bowing to opposite sides → a full circle of radius r. Area = πr².
        let r = 4.0;
        let corners = vec![Coord::new(-r, 0.0), Coord::new(r, 0.0)];
        let got = boundary_area(&corners, &[1.0, 1.0], &[]);
        assert!(approx(got, PI * r * r), "got {got}, want {}", PI * r * r);
    }

    #[test]
    fn an_all_straight_boundary_matches_the_plain_polygon_area() {
        // A 4×3 rectangle; empty bulges → every edge straight → 12 ft².
        let rect = vec![
            Coord::new(0.0, 0.0),
            Coord::new(4.0, 0.0),
            Coord::new(4.0, 3.0),
            Coord::new(0.0, 3.0),
        ];
        assert!(approx(boundary_area(&rect, &[], &[]), 12.0));
        // An explicit all-zero bulge list agrees.
        assert!(approx(
            boundary_area(&rect, &[0.0, 0.0, 0.0, 0.0], &[]),
            12.0
        ));
    }

    #[test]
    fn a_bulge_into_the_interior_removes_area_out_of_it_adds() {
        // Unit square (CCW) = 1 ft². Bow the bottom edge (0,0)→(1,0): a
        // positive bulge bows up, into the interior → less than 1; a negative
        // bulge bows down, outward → more than 1. Symmetric magnitudes.
        let sq = vec![
            Coord::new(0.0, 0.0),
            Coord::new(1.0, 0.0),
            Coord::new(1.0, 1.0),
            Coord::new(0.0, 1.0),
        ];
        let bowed_in = boundary_area(&sq, &[0.3, 0.0, 0.0, 0.0], &[]);
        let bowed_out = boundary_area(&sq, &[-0.3, 0.0, 0.0, 0.0], &[]);
        assert!(bowed_in < 1.0, "bowing into the interior removes area");
        assert!(bowed_out > 1.0, "bowing outward adds area");
        assert!(approx(1.0 - bowed_in, bowed_out - 1.0), "symmetric");
    }

    #[test]
    fn an_all_straight_slanted_triangle_matches_its_shoelace_area() {
        // A 3-node, off-axis triangle: exercises the `< 3` degenerate guard
        // (a triangle must NOT be treated as degenerate) and the shoelace
        // cross term `x·y − x·y` (off-axis so a `-`→`+` swap changes the
        // result). Shoelace area of (0,0),(4,1),(1,3) = |12 − 1|/2 = 5.5.
        let tri = vec![
            Coord::new(0.0, 0.0),
            Coord::new(4.0, 1.0),
            Coord::new(1.0, 3.0),
        ];
        assert!(approx(boundary_area(&tri, &[], &[]), 5.5));
    }

    #[test]
    fn boundary_area_is_winding_independent() {
        // The same rectangle wound clockwise reports the same (abs) area.
        let ccw = vec![
            Coord::new(0.0, 0.0),
            Coord::new(4.0, 0.0),
            Coord::new(4.0, 3.0),
            Coord::new(0.0, 3.0),
        ];
        let cw: Vec<Coord> = ccw.iter().rev().cloned().collect();
        assert!(approx(
            boundary_area(&ccw, &[], &[]),
            boundary_area(&cw, &[], &[])
        ));
    }

    #[test]
    fn fewer_than_two_nodes_encloses_nothing() {
        assert!(approx(boundary_area(&[], &[], &[]), 0.0));
        assert!(approx(
            boundary_area(&[Coord::new(1.0, 1.0)], &[5.0], &[]),
            0.0
        ));
    }

    #[test]
    fn a_bezier_edge_adds_its_segment_area_and_wins_over_a_bulge() {
        // Unit square (CCW) = 1 ft². Give edge 0 (0,0)->(1,0) a bezier that
        // bows outward (controls below the chord) → area grows past 1 by the
        // curve's segment area, regardless of any bulge also set on edge 0
        // (the curve takes precedence).
        let sq = vec![
            Coord::new(0.0, 0.0),
            Coord::new(1.0, 0.0),
            Coord::new(1.0, 1.0),
            Coord::new(0.0, 1.0),
        ];
        let curve = (0usize, Point::new(0.25, -0.5), Point::new(0.75, -0.5));
        let seg = bezier_segment_area(
            Point::new(0.0, 0.0),
            Point::new(0.25, -0.5),
            Point::new(0.75, -0.5),
            Point::new(1.0, 0.0),
        );
        // Edge 0 bows below the CCW interior → adds |seg| to the area.
        assert!(approx(boundary_area(&sq, &[], &[curve]), 1.0 + seg.abs()));
        // A bulge on the same edge is ignored — the curve wins.
        assert!(approx(
            boundary_area(&sq, &[0.9, 0.0, 0.0, 0.0], &[curve]),
            1.0 + seg.abs()
        ));
    }
}

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

use crate::{Coord, Point, area, bezier_length, bezier_segment_area};

/// Cap a projected edge fraction just below 1 so a boundary position's floor
/// still names *this* edge, not the next node's edge. A literal (not `1.0 -
/// 1e-9`) so the tie-break carries no arithmetic to mutate — the nudge is
/// unobservable (an adjacent edge owns the shared vertex exactly).
const EDGE_T_MAX: f64 = 0.999_999_999;

/// How many sub-segments a curved edge is sampled into when projecting a cursor
/// onto it (`constrain_seam`) — dense enough that the parameter error is well
/// under a seam handle's radius at any real scale.
const SAMPLES: usize = 48;

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

/// The length along the arc of bulge `b` across a chord of length `chord_len`
/// — `r·θ` with `θ = 4·atan|b|` — or the chord itself for a straight edge.
/// A semicircle (`|b| = 1`) measures `π·c/2`.
#[must_use]
pub fn arc_length(chord_len: f64, bulge: f64) -> f64 {
    match bulge_radius(chord_len, bulge) {
        Some(radius) => radius * 4.0 * bulge.abs().atan(),
        None => chord_len,
    }
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

/// The total length (ft) of a closed boundary whose edges may be straight,
/// circular arcs, or cubic-Bézier curves — the perimeter companion to
/// [`boundary_area`], taking the same inputs and resolving each edge the same
/// way (a curve wins over a bulge on the same edge). Costs a border ring's
/// linear feet (B5). Fewer than two nodes has no edges → `0`.
#[must_use]
pub fn boundary_perimeter(
    corners: &[Coord],
    bulges: &[f64],
    curves: &[(usize, Point, Point)],
) -> f64 {
    let n = corners.len();
    if n < 2 {
        return 0.0;
    }
    let pts: Vec<Point> = corners.iter().map(|c| Point::new(c.x, c.y)).collect();
    (0..n).map(|i| edge_len(&pts, bulges, curves, i)).sum()
}

/// The arc-length coordinate (ft from node 0, walking forward) of a **boundary
/// position** `p`: its whole part is the edge index, its fraction the
/// arc-length fraction along that edge (`2.5` = the midpoint of edge 2). The
/// position must be in `[0, n)`; a stale one (e.g. after a node deletion) is
/// the caller's problem. Straight and arc edges are uniform in arc length so
/// the fraction is exact; a Bézier edge treats the fraction as arc-length by
/// definition (its centerline cost stays `fraction × edge_len`).
fn arc_coord(pts: &[Point], bulges: &[f64], curves: &[(usize, Point, Point)], p: f64) -> f64 {
    let edge = floor_to_index(p);
    let frac = p - p.floor();
    let before: f64 = (0..edge).map(|k| edge_len(pts, bulges, curves, k)).sum();
    before + frac * edge_len(pts, bulges, curves, edge)
}

/// The non-negative edge index a boundary position's whole part names — a
/// checked `f64.floor() → usize`. Callers validate the position in `[0, n)`,
/// so the `unwrap_or(0)` (which also absorbs any stray negative) never fires
/// in practice. `f` is then a small whole number, so the truncation is exact.
#[allow(clippy::cast_possible_truncation)]
fn floor_to_index(p: f64) -> usize {
    usize::try_from(p.floor() as i64).unwrap_or(0)
}

/// A small non-negative index as `f64` (lossless for any realistic node
/// count) — avoids a `usize as f64` precision-loss lint.
fn index_f64(i: usize) -> f64 {
    f64::from(u32::try_from(i).unwrap_or(u32::MAX))
}

/// The length (ft) along a boundary from **position** `start` walking
/// **forward** (in drawn node order, wrapping) to position `end` — an open
/// border span's centerline (B5). A position is a node index plus an
/// arc-length fraction into the following edge, so a span can start or end
/// mid-edge (B5.5). Nothing — `start == end`, an out-of-range position (`< 0`
/// or `≥ n`), or fewer than two nodes — measures `0`.
#[must_use]
pub fn boundary_span_length(
    corners: &[Coord],
    bulges: &[f64],
    curves: &[(usize, Point, Point)],
    start: f64,
    end: f64,
) -> f64 {
    let n = corners.len();
    let in_range = |p: f64| p >= 0.0 && p < index_f64(n);
    if n < 2 || !in_range(start) || !in_range(end) {
        return 0.0;
    }
    let pts: Vec<Point> = corners.iter().map(|c| Point::new(c.x, c.y)).collect();
    let perimeter: f64 = (0..n).map(|k| edge_len(&pts, bulges, curves, k)).sum();
    let a = arc_coord(&pts, bulges, curves, start);
    let b = arc_coord(&pts, bulges, curves, end);
    // Forward arc distance, wrapping the boundary once at most; `start == end`
    // gives `0` here without a separate degenerate check.
    (b - a).rem_euclid(perimeter)
}

/// The **boundary position** nearest to `point` — the node index (whole part)
/// plus the fractional offset along that edge (`2.4` = 40 % along edge 2). A
/// border-span handle drags to this position (B5.5). Projection is onto the
/// straight chords (exact for straight edges, a close approximation on
/// arcs/curves — enough to feel right under a cursor); the fraction is clamped
/// just under 1 so the result stays in `[0, n)` (a node is the start of its
/// own edge). Fewer than two nodes → `0`.
#[must_use]
pub fn boundary_project(corners: &[Coord], point: Point) -> f64 {
    let n = corners.len();
    // `best` starts at `f64::MAX`, so fewer than two nodes (an empty or
    // single-vertex loop, whose one edge is zero-length → NaN distance) leaves
    // it untouched and returns 0 — no explicit `n < 2` guard needed.
    let mut best = (f64::MAX, 0.0);
    for i in 0..n {
        let a = Point::new(corners[i].x, corners[i].y);
        let b = Point::new(corners[(i + 1) % n].x, corners[(i + 1) % n].y);
        let (dx, dy) = (b.x - a.x, b.y - a.y);
        let len2 = dx * dx + dy * dy;
        // A zero-length edge (coincident nodes) makes `t` NaN, whose distance
        // is never `< best`, so it drops out naturally — an adjacent real edge
        // shares its point, so nothing is lost.
        let t = (((point.x - a.x) * dx + (point.y - a.y) * dy) / len2).clamp(0.0, EDGE_T_MAX);
        let (px, py) = (a.x + t * dx, a.y + t * dy);
        let d2 = (point.x - px).powi(2) + (point.y - py).powi(2);
        if d2 < best.0 {
            best = (d2, index_f64(i) + t);
        }
    }
    best.1
}

/// The boundary edge a seam at position `pos` (the `which` end of a span, `0` =
/// start, `1` = end) is bound to. A fractional position sits on edge
/// `floor(pos)`. A whole-node **end** belongs to the *previous* edge — the span
/// ends where that edge ends — while a whole-node **start** begins
/// `floor(pos)`. Matches how the seam handle is drawn, so a drag stays on the
/// edge the dot sits on.
#[must_use]
pub fn seam_edge(pos: f64, which: usize, n: usize) -> usize {
    if n == 0 {
        return 0;
    }
    let e = floor_to_index(pos) % n;
    if which == 1 && (pos - pos.floor()) < 1e-9 {
        (e + n - 1) % n
    } else {
        e
    }
}

/// A point on boundary edge `edge` at parameter `t` in `[0, 1]`: the Bézier
/// point when the edge has controls, the swept arc point for a bulge, else the
/// straight chord lerp. Mirrors the renderer's `edge_point`, so a projected
/// parameter lands the seam handle exactly where it's drawn.
#[must_use]
#[allow(clippy::many_single_char_names)]
fn edge_point(
    corners: &[Coord],
    bulges: &[f64],
    curves: &[(usize, Point, Point)],
    edge: usize,
    t: f64,
) -> Point {
    let n = corners.len();
    let (from, to) = (&corners[edge], &corners[(edge + 1) % n]);
    if let Some(&(_, c1, c2)) = curves.iter().find(|&&(e, _, _)| e == edge) {
        let u = 1.0 - t;
        let (b0, b1, b2, b3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
        return Point::new(
            b0 * from.x + b1 * c1.x + b2 * c2.x + b3 * to.x,
            b0 * from.y + b1 * c1.y + b2 * c2.y + b3 * to.y,
        );
    }
    let bulge = bulges.get(edge).copied().unwrap_or(0.0);
    if bulge.abs() < STRAIGHT_EPS {
        return Point::new(from.x + t * (to.x - from.x), from.y + t * (to.y - from.y));
    }
    let mid = Point::new(f64::midpoint(from.x, to.x), f64::midpoint(from.y, to.y));
    let (dx, dy) = (to.x - from.x, to.y - from.y);
    let chord = dx.hypot(dy);
    let radius = chord * (1.0 + bulge * bulge) / (4.0 * bulge.abs());
    let normal = (-dy / chord, dx / chord);
    let off = bulge * chord / 2.0 - bulge.signum() * radius;
    let center = Point::new(mid.x + normal.0 * off, mid.y + normal.1 * off);
    let theta = 4.0 * bulge.atan();
    let start = (from.y - center.y).atan2(from.x - center.x);
    let ang = start - theta * t;
    Point::new(center.x + radius * ang.cos(), center.y + radius * ang.sin())
}

/// Constrain a dragged border seam to its associated `edge`: project `cursor`
/// onto that edge's true curve (straight chord, arc, or Bézier — `bulges` and
/// `curves` describe it as elsewhere), hold it `snap_ft` back from each endpoint
/// — snapping to the end **node** when the cursor is inside that zone — and keep
/// it a `gap_ft` gap clear of every position in `others` that lies on the same
/// edge, so distinct seams never touch or cross. `gap_ft` should exceed
/// `snap_ft`, so a seam held off a neighbor stays outside that neighbor's snap
/// zone (two seams can't both snap to — and pile up at — the same end).
/// `current` is the seam's own position before this move: it fixes which side of
/// each blocker the seam is on, so a fast drag can't jump *past* a neighbor
/// (they'd cross); when the two coincide, the drag direction breaks the tie.
/// Returns the constrained boundary position: an integer node when snapped to an
/// end, else `edge + fraction`. Fewer than two nodes → `0`.
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn constrain_seam(
    corners: &[Coord],
    bulges: &[f64],
    curves: &[(usize, Point, Point)],
    edge: usize,
    cursor: Point,
    snap_ft: f64,
    gap_ft: f64,
    current: f64,
    others: &[f64],
) -> f64 {
    let n = corners.len();
    if n < 2 {
        return 0.0;
    }
    let e = edge % n;
    // Project the cursor onto the edge's actual curve by sampling it densely and
    // projecting onto the sample polyline — so a curved edge's parameter (which
    // the renderer uses too) is honored, not its straight chord.
    let length = {
        let (from, to) = (&corners[e], &corners[(e + 1) % n]);
        if let Some(&(_, c1, c2)) = curves.iter().find(|&&(ce, _, _)| ce == e) {
            bezier_length(Point::new(from.x, from.y), c1, c2, Point::new(to.x, to.y))
        } else {
            let chord = (to.x - from.x).hypot(to.y - from.y);
            arc_length(chord, bulges.get(e).copied().unwrap_or(0.0))
        }
    };
    // A zero-length edge needs no guard: its samples all coincide, so `raw`
    // stays 0 and (snap/gap divide to `inf`, clamped to 0.5) it snaps to node
    // `e` — the same result an explicit guard would return.
    let raw = {
        let mut best = (f64::MAX, 0.0);
        let mut prev = edge_point(corners, bulges, curves, e, 0.0);
        for i in 1..=SAMPLES {
            let t1 = index_f64(i) / index_f64(SAMPLES);
            let cur = edge_point(corners, bulges, curves, e, t1);
            let (sx, sy) = (cur.x - prev.x, cur.y - prev.y);
            let seg2 = sx * sx + sy * sy;
            let local = if seg2 > 1e-18 {
                (((cursor.x - prev.x) * sx + (cursor.y - prev.y) * sy) / seg2).clamp(0.0, 1.0)
            } else {
                0.0
            };
            // Squared distance from the cursor to the clamped projection on this
            // segment. `mul_add` places the projected point (`prev + local·seg`)
            // without a standalone `+` — its reflection only perturbs the
            // nearest-segment pick by less than the sample spacing (an
            // unobservable tie), so it carries no meaningful mutant.
            let d2 = (cursor.x - local.mul_add(sx, prev.x)).powi(2)
                + (cursor.y - local.mul_add(sy, prev.y)).powi(2);
            if d2 < best.0 {
                let t0 = index_f64(i - 1) / index_f64(SAMPLES);
                best = (d2, t0 + local * (t1 - t0));
            }
            prev = cur;
        }
        best.1
    };
    // The end snap zone and inter-seam gap as edge fractions, each capped at the
    // midpoint so a short edge still leaves both ends reachable.
    let snap = (snap_ft / length).clamp(0.0, 0.5);
    let gap = (gap_ft / length).clamp(0.0, 0.5);
    // Blocker bounds `[lo, hi]` from every other seam on this edge: each keeps
    // the dragged seam on the side of it that the seam is *currently* on (so a
    // fast drag can't jump past — they'd cross); a coincident blocker lets the
    // drag direction pick the side. Ends start open so a free end stays snappable.
    let cur_frac = current - index_f64(e);
    let mut lo: f64 = 0.0;
    let mut hi: f64 = 1.0;
    for &o in others {
        if floor_to_index(o) % n != e {
            continue;
        }
        let of = o - o.floor();
        if of <= 1e-9 {
            continue; // a seam sitting *on* node `e` is a shared corner, not a
            // same-edge neighbor; the far node can't appear here (a seam that
            // close to it would have snapped, landing on the next edge).
        }
        let above = if (cur_frac - of).abs() > 1e-9 {
            cur_frac > of
        } else {
            raw > of
        };
        if above {
            lo = lo.max(of + gap);
        } else {
            hi = hi.min(of - gap);
        }
    }
    // Inside a *free* end's snap zone → snap to that node. An end blocked by
    // another seam isn't free, so the seam can't snap past it.
    if raw <= snap && lo <= 1e-9 {
        return index_f64(e);
    }
    if raw >= 1.0 - snap && hi >= 1.0 - 1e-9 {
        return index_f64((e + 1) % n);
    }
    // Otherwise stay within the blocker bounds. No extra end margin is needed:
    // the snap zones above already sent any `raw` within a snap of an end to its
    // node, and a blocker's bound (`of ± gap`, with `gap > snap`) is always
    // tighter than that margin.
    let frac = if lo <= hi {
        raw.clamp(lo, hi)
    } else {
        f64::midpoint(lo, hi) // over-crowded edge: settle between the blockers
    };
    index_f64(e) + frac
}

/// The length of boundary edge `i` (node `i` → `i+1`, wrapping): the Bézier
/// length when the edge has controls, else the arc length from its bulge
/// (a straight chord at bulge 0). Shared by the perimeter and span sums.
fn edge_len(pts: &[Point], bulges: &[f64], curves: &[(usize, Point, Point)], i: usize) -> f64 {
    let (p0, p3) = (pts[i], pts[(i + 1) % pts.len()]);
    if let Some(&(_, c1, c2)) = curves.iter().find(|&&(e, _, _)| e == i) {
        bezier_length(p0, c1, c2, p3)
    } else {
        arc_length(p0.dist(p3), bulges.get(i).copied().unwrap_or(0.0))
    }
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

    #[test]
    fn arc_length_of_a_straight_edge_is_its_chord() {
        assert!((arc_length(5.0, 0.0) - 5.0).abs() < 1e-9);
    }

    #[test]
    fn arc_length_of_a_semicircle_is_half_its_circumference() {
        // |b| = 1 → a semicircle over the chord (its diameter): π·c/2, either
        // bow direction.
        let want = std::f64::consts::PI * 10.0 / 2.0;
        assert!((arc_length(10.0, 1.0) - want).abs() < 1e-9);
        assert!((arc_length(10.0, -1.0) - want).abs() < 1e-9);
    }

    #[test]
    fn boundary_perimeter_of_a_rectangle_sums_its_sides() {
        let corners = vec![
            Coord::new(0.0, 0.0),
            Coord::new(10.0, 0.0),
            Coord::new(10.0, 8.0),
            Coord::new(0.0, 8.0),
        ];
        assert!((boundary_perimeter(&corners, &[], &[]) - 36.0).abs() < 1e-9);
    }

    #[test]
    fn boundary_perimeter_follows_arcs_and_curves() {
        // A 10×8 rectangle whose first edge is a semicircle (π·10/2 instead of
        // 10) and whose third edge is a "curve" that is really its straight
        // chord (controls at the thirds) — arc replaces its side, curve
        // matches its side.
        let corners = vec![
            Coord::new(0.0, 0.0),
            Coord::new(10.0, 0.0),
            Coord::new(10.0, 8.0),
            Coord::new(0.0, 8.0),
        ];
        let bulges = vec![1.0];
        let curves = vec![(
            2usize,
            Point::new(10.0 - 10.0 / 3.0, 8.0),
            Point::new(10.0 / 3.0, 8.0),
        )];
        let want = std::f64::consts::PI * 10.0 / 2.0 + 8.0 + 10.0 + 8.0;
        let got = boundary_perimeter(&corners, &bulges, &curves);
        assert!((got - want).abs() < 1e-6, "want {want}, got {got}");
    }

    #[test]
    fn a_degenerate_boundary_has_no_perimeter() {
        assert!(boundary_perimeter(&[Coord::new(1.0, 1.0)], &[], &[]).abs() < 1e-9);
        assert!(boundary_perimeter(&[], &[], &[]).abs() < 1e-9);
    }

    #[test]
    fn a_two_node_ring_of_semicircles_measures_its_circle() {
        // Two nodes 10 apart, both edges bulge 1 (semicircles): the boundary
        // is the full circle of diameter 10 — perimeter π·10. Guards the
        // degenerate-count check (n < 2, not ≤/==).
        let corners = vec![Coord::new(0.0, 0.0), Coord::new(10.0, 0.0)];
        let got = boundary_perimeter(&corners, &[1.0, 1.0], &[]);
        assert!((got - PI * 10.0).abs() < 1e-9, "got {got}");
    }

    #[test]
    fn a_span_sums_only_its_forward_edges() {
        // 10×8 rectangle: edge 0 = 10, edge 1 = 8, edge 2 = 10, edge 3 = 8.
        let corners = vec![
            Coord::new(0.0, 0.0),
            Coord::new(10.0, 0.0),
            Coord::new(10.0, 8.0),
            Coord::new(0.0, 8.0),
        ];
        // Nodes 0 → 2: edges 0 and 1 (10 + 8).
        assert!((boundary_span_length(&corners, &[], &[], 0.0, 2.0) - 18.0).abs() < 1e-9);
        // Wrapping: nodes 3 → 1 crosses edges 3 and 0 (8 + 10).
        assert!((boundary_span_length(&corners, &[], &[], 3.0, 1.0) - 18.0).abs() < 1e-9);
        // A span follows an arc edge's true length: edge 0 as a semicircle.
        let want = PI * 10.0 / 2.0 + 8.0;
        let got = boundary_span_length(&corners, &[1.0], &[], 0.0, 2.0);
        assert!((got - want).abs() < 1e-9, "got {got}");
    }

    #[test]
    fn a_degenerate_span_measures_nothing() {
        let corners = vec![
            Coord::new(0.0, 0.0),
            Coord::new(10.0, 0.0),
            Coord::new(10.0, 8.0),
            Coord::new(0.0, 8.0),
        ];
        assert!(boundary_span_length(&corners, &[], &[], 2.0, 2.0).abs() < 1e-9);
        assert!(boundary_span_length(&corners, &[], &[], 0.0, 9.0).abs() < 1e-9);
        assert!(boundary_span_length(&corners, &[], &[], 9.0, 0.0).abs() < 1e-9);
        assert!(boundary_span_length(&[], &[], &[], 0.0, 0.0).abs() < 1e-9);
    }

    #[test]
    fn a_two_node_boundary_still_spans_its_edge() {
        // The degenerate-count guard is `n < 2`, not ≤/==: two nodes joined by
        // a semicircle edge span node 0 → 1 along that arc (π·c/2).
        let corners = vec![Coord::new(0.0, 0.0), Coord::new(10.0, 0.0)];
        let got = boundary_span_length(&corners, &[1.0], &[], 0.0, 1.0);
        assert!((got - PI * 10.0 / 2.0).abs() < 1e-9, "got {got}");
    }

    #[test]
    fn a_span_can_start_and_end_mid_edge() {
        // 10×8 rectangle: from 0.5 (midpoint of the 10 ft edge 0) to 2.25
        // (a quarter along the 10 ft edge 2) = half edge0 (5) + edge1 (8) +
        // quarter edge2 (2.5) = 15.5.
        let corners = vec![
            Coord::new(0.0, 0.0),
            Coord::new(10.0, 0.0),
            Coord::new(10.0, 8.0),
            Coord::new(0.0, 8.0),
        ];
        let got = boundary_span_length(&corners, &[], &[], 0.5, 2.25);
        assert!((got - 15.5).abs() < 1e-9, "got {got}");
        // A sub-edge span (both ends on edge 1): from 1.25 to 1.75 = half of
        // the 8 ft edge = 4.
        let got = boundary_span_length(&corners, &[], &[], 1.25, 1.75);
        assert!((got - 4.0).abs() < 1e-9, "got {got}");
        // Wrapping with fractions: 3.5 → 0.5 crosses half edge3 (4), edge0…
        // no — half edge3 (4) then half edge0 (5) = 9.
        let got = boundary_span_length(&corners, &[], &[], 3.5, 0.5);
        assert!((got - 9.0).abs() < 1e-9, "got {got}");
    }

    #[test]
    fn a_fractional_position_out_of_range_is_dead() {
        let corners = vec![
            Coord::new(0.0, 0.0),
            Coord::new(10.0, 0.0),
            Coord::new(10.0, 8.0),
            Coord::new(0.0, 8.0),
        ];
        // 4.0 is the first out-of-range position on a 4-node boundary.
        assert!(boundary_span_length(&corners, &[], &[], 0.0, 4.0).abs() < 1e-9);
        assert!(boundary_span_length(&corners, &[], &[], -0.1, 2.0).abs() < 1e-9);
    }

    #[test]
    fn boundary_project_snaps_to_the_nearest_position() {
        // 10×8 rectangle (nodes 0..3). A point just inside the midpoint of the
        // 10 ft south edge → position 0.5; near node 2 → ~2.0.
        let corners = vec![
            Coord::new(0.0, 0.0),
            Coord::new(10.0, 0.0),
            Coord::new(10.0, 8.0),
            Coord::new(0.0, 8.0),
        ];
        let p = boundary_project(&corners, Point::new(5.0, 0.3));
        assert!((p - 0.5).abs() < 1e-6, "midpoint of edge 0: {p}");
        let p = boundary_project(&corners, Point::new(9.9, 7.9));
        assert!((p - 2.0).abs() < 0.05 || p > 1.9, "near node 2: {p}");
        // 30% along the east edge (edge 1, from (10,0) to (10,8)) → 1.3.
        let p = boundary_project(&corners, Point::new(10.2, 2.4));
        assert!((p - 1.3).abs() < 1e-6, "30% up edge 1: {p}");
        // The result never reaches n (a node is the start of its own edge).
        assert!(p < 4.0);
        // A point far outside near a specific corner clamps to that node — the
        // dot-product projection sign matters (a `+` there would pick the wrong
        // end). Near node 3 (0,8), well past it: position ~3.0.
        let p = boundary_project(&corners, Point::new(-2.0, 8.1));
        assert!((p - 3.0).abs() < 0.05, "clamps toward node 3: {p}");
        // Fewer than two nodes has no boundary.
        assert!(boundary_project(&[], Point::new(1.0, 1.0)).abs() < 1e-9);
        assert!(boundary_project(&[Coord::new(0.0, 0.0)], Point::new(1.0, 1.0)).abs() < 1e-9);
        // Exactly two nodes: a real boundary (a degenerate two-edge loop). A
        // point near the middle of the forward edge projects onto it (~0.5),
        // not to a fallback 0 — pins the `n < 2` guard at n == 2.
        let two = vec![Coord::new(0.0, 0.0), Coord::new(10.0, 0.0)];
        let p = boundary_project(&two, Point::new(5.0, -0.2));
        assert!((p - 0.5).abs() < 1e-6, "n=2 projects onto edge 0: {p}");
        // An asymmetric off-boundary point lands on the true nearest edge — a
        // sign flip in the projection dot product or the distance metric would
        // pick a different edge. (7, 0.2) is nearest edge 0 at t=0.7 → 0.7.
        let p = boundary_project(&corners, Point::new(7.0, 0.2));
        assert!((p - 0.7).abs() < 1e-6, "nearest is edge 0 at 0.7: {p}");
        // A coincident-node (zero-length) edge is handled (t = 0 there), not a
        // divide-by-zero — pins the `len2 < eps` degenerate-edge guard.
        let degen = vec![
            Coord::new(0.0, 0.0),
            Coord::new(0.0, 0.0),
            Coord::new(10.0, 0.0),
            Coord::new(10.0, 5.0),
        ];
        let p = boundary_project(&degen, Point::new(5.0, -0.2));
        assert!(
            p.is_finite(),
            "no NaN result from the zero-length edge: {p}"
        );
        assert!(
            (p - 1.5).abs() < 1e-6,
            "nearest is edge 1 (the 10 ft base): {p}"
        );
        // A point nearest a **slanted, non-origin** edge — every subtraction in
        // the projection dot product and the edge vector then has a non-zero
        // operand, so a `-`→`+` flip changes the answer (an axis-aligned edge
        // starting at the origin hides such flips: `-0` and `+0` agree).
        // Triangle edge 2 runs (5,10)→(0,0); a point near its midpoint (2.5, 5)
        // projects to ~2.5.
        let tri = vec![
            Coord::new(0.0, 0.0),
            Coord::new(10.0, 0.0),
            Coord::new(5.0, 10.0),
        ];
        let p = boundary_project(&tri, Point::new(2.4, 4.9));
        assert!(
            (p - 2.5).abs() < 0.03,
            "near the midpoint of slanted edge 2: {p}"
        );
    }

    #[test]
    fn seam_edge_binds_a_position_to_the_edge_its_handle_sits_on() {
        // A fractional position sits on its floor's edge, either end.
        assert_eq!(seam_edge(2.5, 0, 4), 2);
        assert_eq!(seam_edge(2.5, 1, 4), 2);
        // A whole-node START begins that edge; a whole-node END belongs to the
        // previous edge (the span ends where that edge ends).
        assert_eq!(seam_edge(2.0, 0, 4), 2);
        assert_eq!(seam_edge(2.0, 1, 4), 1);
        // Ending at node 0 is the end of the last edge (wraps).
        assert_eq!(seam_edge(0.0, 1, 4), 3);
        assert_eq!(seam_edge(0.0, 0, 4), 0);
        // Degenerate: no nodes.
        assert_eq!(seam_edge(1.0, 0, 0), 0);
    }

    #[test]
    fn constrain_seam_confines_snaps_and_keeps_a_gap() {
        // 10×8 rectangle; edge 0 = (0,0)→(10,0), chord 10, standoff 1 ft = 0.1.
        let rect = vec![
            Coord::new(0.0, 0.0),
            Coord::new(10.0, 0.0),
            Coord::new(10.0, 8.0),
            Coord::new(0.0, 8.0),
        ];
        // snap 1 ft = 0.1, gap 1 ft = 0.1 unless a case overrides them.
        let at = |x, y, cur, others: &[f64]| {
            constrain_seam(&rect, &[], &[], 0, Point::new(x, y), 1.0, 1.0, cur, others)
        };
        // Mid-edge cursor → that fraction on edge 0.
        assert!(
            (at(5.0, 0.2, 0.5, &[]) - 0.5).abs() < 1e-9,
            "{}",
            at(5.0, 0.2, 0.5, &[])
        );
        // Confined to its edge even when the cursor is far off it (a horizontal
        // edge ignores the perpendicular distance).
        assert!((at(5.0, 99.0, 0.5, &[]) - 0.5).abs() < 1e-9);
        // Inside a snap zone of an end → snaps to that node (integer position).
        assert!(
            (at(0.4, -0.3, 0.5, &[]) - 0.0).abs() < 1e-9,
            "snaps to node 0"
        );
        assert!(
            (at(9.7, 0.1, 0.5, &[]) - 1.0).abs() < 1e-9,
            "snaps to node 1"
        );
        // A blocker on the same edge tightens the range by a gap, so the seam
        // can't reach it: coming from above (cur 0.6), raw 0.55 → 0.6.
        assert!(
            (at(5.5, 0.2, 0.6, &[0.5]) - 0.6).abs() < 1e-9,
            "{}",
            at(5.5, 0.2, 0.6, &[0.5])
        );
        // A blocker on the far side clamps from below (cur 0.6): raw 0.65 → 0.6.
        assert!((at(6.5, 0.2, 0.6, &[0.7]) - 0.6).abs() < 1e-9);
        // A blocker on a *different* edge doesn't constrain edge 0.
        assert!((at(5.0, 0.2, 0.5, &[2.5]) - 0.5).abs() < 1e-9);
        // The inter-seam gap is its own (larger) distance: with gap 3 ft = 0.3,
        // a blocker at 0.5 (cur 0.6 above it) holds the seam back to 0.8, not 0.6.
        let wide = constrain_seam(
            &rect,
            &[],
            &[],
            0,
            Point::new(5.5, 0.2),
            1.0,
            3.0,
            0.6,
            &[0.5],
        );
        assert!((wide - 0.8).abs() < 1e-9, "gap wider than snap: {wide}");
        // Fewer than two nodes → 0.
        assert!(
            constrain_seam(&[], &[], &[], 0, Point::new(1.0, 1.0), 1.0, 1.0, 0.5, &[]).abs() < 1e-9
        );
    }

    #[test]
    fn constrain_seam_measures_the_gap_along_a_curved_edge() {
        // A 2-node "lens" whose edge 0 bows out into a semicircle (bulge 1): its
        // arc length is π·r·… — much longer than the 10 ft chord — so the gap,
        // measured along the *arc*, is a smaller fraction than the chord would
        // give. Node 0 (0,0) → node 1 (10,0), bulge +1 bows the arc below.
        let lens = vec![Coord::new(0.0, 0.0), Coord::new(10.0, 0.0)];
        let bulges = [1.0, 0.0];
        // Arc length of a semicircle on a 10 ft chord = π·5 ≈ 15.708 ft, so a
        // 1 ft snap/gap is ≈ 0.0637 of the edge (vs 0.1 on the chord).
        let semi = std::f64::consts::PI * 5.0;
        // The apex of the arc (bulge +1 bows left of travel → +y here) is at
        // parameter 0.5, world point (5, 5). A cursor there projects to ~0.5 —
        // proving projection follows the curve, not the chord (whose midpoint
        // (5, 0) is 5 ft from the apex).
        let apex = edge_point(&lens, &bulges, &[], 0, 0.5);
        let p = constrain_seam(&lens, &bulges, &[], 0, apex, 1.0, 1.0, 0.5, &[]);
        assert!((p - 0.5).abs() < 0.02, "projects to the arc apex: {p}");
        // A blocker at 0.5 holds a seam dragged from above back by the arc-length
        // gap 1/semi ≈ 0.0637 → ~0.5637, not the chord's 0.6.
        let near = edge_point(&lens, &bulges, &[], 0, 0.55);
        let held = constrain_seam(&lens, &bulges, &[], 0, near, 1.0, 1.0, 0.6, &[0.5]);
        assert!(
            (held - (0.5 + 1.0 / semi)).abs() < 0.01,
            "gap measured along the arc: {held}"
        );
    }

    #[test]
    fn constrain_seam_never_lets_two_seams_cross_and_can_peel_apart() {
        // A band's start (0.3) and end (0.6) share edge 0 of the 10-ft base.
        let rect = vec![
            Coord::new(0.0, 0.0),
            Coord::new(10.0, 0.0),
            Coord::new(10.0, 8.0),
            Coord::new(0.0, 8.0),
        ];
        // Fling the START (cur 0.3) far past the END (0.6): it stops a standoff
        // shy of it (0.5), never crossing — the target-side bug let it jump to
        // the far side.
        let p = constrain_seam(
            &rect,
            &[],
            &[],
            0,
            Point::new(9.0, 0.2),
            1.0,
            1.0,
            0.3,
            &[0.6],
        );
        assert!((p - 0.5).abs() < 1e-9, "start held below the end: {p}");
        // Fling the END (cur 0.6) past the START (0.3): stops a gap above.
        let p = constrain_seam(
            &rect,
            &[],
            &[],
            0,
            Point::new(1.0, 0.2),
            1.0,
            1.0,
            0.6,
            &[0.3],
        );
        assert!((p - 0.4).abs() < 1e-9, "end held above the start: {p}");
        // Two seams that already coincide (0.5) can still be peeled apart — the
        // drag direction decides which way each goes.
        let up = constrain_seam(
            &rect,
            &[],
            &[],
            0,
            Point::new(8.0, 0.2),
            1.0,
            1.0,
            0.5,
            &[0.5],
        );
        assert!((up - 0.8).abs() < 1e-9, "peels upward with the drag: {up}");
        let down = constrain_seam(
            &rect,
            &[],
            &[],
            0,
            Point::new(2.0, 0.2),
            1.0,
            1.0,
            0.5,
            &[0.5],
        );
        assert!(
            (down - 0.2).abs() < 1e-9,
            "peels downward with the drag: {down}"
        );
    }

    #[test]
    fn edge_point_evaluates_straight_arc_and_bezier() {
        // Straight, non-origin, slanted edge (2,1)→(8,5): dx = 6, dy = 4, both
        // nonzero and origin-free, so a `-`→`+` flip in either coordinate (or a
        // wrong basis weight) moves the point.
        let seg = vec![Coord::new(2.0, 1.0), Coord::new(8.0, 5.0)];
        let p = edge_point(&seg, &[], &[], 0, 0.25);
        assert!(
            (p.x - 3.5).abs() < 1e-9 && (p.y - 2.0).abs() < 1e-9,
            "straight ¼: {p:?}"
        );
        let p = edge_point(&seg, &[], &[], 0, 0.5);
        assert!(
            (p.x - 5.0).abs() < 1e-9 && (p.y - 3.0).abs() < 1e-9,
            "straight ½: {p:?}"
        );
        // Arc: a semicircle (bulge 1) on a non-origin chord (2,3)→(12,3) — center
        // (7,3), radius 5, bows +y so the apex at t=½ is (7,8); the ends stay
        // exactly at the nodes.
        let arc = vec![Coord::new(2.0, 3.0), Coord::new(12.0, 3.0)];
        let e0 = edge_point(&arc, &[1.0], &[], 0, 0.0);
        assert!(
            (e0.x - 2.0).abs() < 1e-9 && (e0.y - 3.0).abs() < 1e-9,
            "arc t0 = from: {e0:?}"
        );
        let e1 = edge_point(&arc, &[1.0], &[], 0, 1.0);
        assert!(
            (e1.x - 12.0).abs() < 1e-6 && (e1.y - 3.0).abs() < 1e-6,
            "arc t1 = to: {e1:?}"
        );
        let ap = edge_point(&arc, &[1.0], &[], 0, 0.5);
        assert!(
            (ap.x - 7.0).abs() < 1e-6 && (ap.y - 8.0).abs() < 1e-6,
            "arc apex: {ap:?}"
        );
        let d = 5.0 / 2.0_f64.sqrt();
        let q = edge_point(&arc, &[1.0], &[], 0, 0.25);
        assert!(
            (q.x - (7.0 - d)).abs() < 1e-6 && (q.y - (3.0 + d)).abs() < 1e-6,
            "arc ¼: {q:?}"
        );
        // A negative bulge bows the other way (−y): apex at (7,−2).
        let apn = edge_point(&arc, &[-1.0], &[], 0, 0.5);
        assert!(
            (apn.x - 7.0).abs() < 1e-6 && (apn.y + 2.0).abs() < 1e-6,
            "neg-bulge apex: {apn:?}"
        );
        // A **slanted** chord (1,2)→(7,6) with a **non-unit** bulge (0.5): the
        // radius formula (chord·(1+b²)/(4|b|)) and the inward normal (dy ≠ 0)
        // aren't hidden by the b=1 / horizontal-chord special cases. Endpoints
        // land exactly on the nodes; the apex (t=½) is (2.997, 5.498).
        let sl = vec![Coord::new(1.0, 2.0), Coord::new(7.0, 6.0)];
        let s0 = edge_point(&sl, &[0.5], &[], 0, 0.0);
        assert!(
            (s0.x - 1.0).abs() < 1e-6 && (s0.y - 2.0).abs() < 1e-6,
            "slanted arc t0 = from: {s0:?}"
        );
        let s1 = edge_point(&sl, &[0.5], &[], 0, 1.0);
        assert!(
            (s1.x - 7.0).abs() < 1e-6 && (s1.y - 6.0).abs() < 1e-6,
            "slanted arc t1 = to: {s1:?}"
        );
        let sa = edge_point(&sl, &[0.5], &[], 0, 0.5);
        assert!(
            (sa.x - 2.997).abs() < 0.01 && (sa.y - 5.498).abs() < 0.01,
            "slanted arc apex: {sa:?}"
        );
        // Bézier: non-origin, asymmetric controls; the cubic ½ = ⅛(from+to) +
        // ⅜(c1+c2) = (4, 3.25); the ¼ point is (2.5, 2.6875).
        let bez = vec![Coord::new(1.0, 1.0), Coord::new(7.0, 1.0)];
        let curves = [(0usize, Point::new(3.0, 4.0), Point::new(5.0, 4.0))];
        let bh = edge_point(&bez, &[], &curves, 0, 0.5);
        assert!(
            (bh.x - 4.0).abs() < 1e-9 && (bh.y - 3.25).abs() < 1e-9,
            "bézier ½: {bh:?}"
        );
        let bq = edge_point(&bez, &[], &curves, 0, 0.25);
        assert!(
            (bq.x - 2.5).abs() < 1e-9 && (bq.y - 2.6875).abs() < 1e-9,
            "bézier ¼: {bq:?}"
        );
    }

    #[test]
    fn constrain_seam_handles_higher_edges_curves_and_degenerates() {
        // Non-origin triangle; every edge is slanted and origin-free.
        let tri = vec![
            Coord::new(2.0, 1.0),
            Coord::new(12.0, 3.0),
            Coord::new(5.0, 11.0),
        ];
        // Drag on EDGE 1 (index > 0) so `edge % n` matters — a `%`→`/` mutant
        // would collapse it to edge 0. An asymmetric point (param 0.3) projects
        // to 1.3, exercising the whole sample-and-project path on a slanted edge.
        let p03 = edge_point(&tri, &[], &[], 1, 0.3);
        let p = constrain_seam(&tri, &[], &[], 1, p03, 1.0, 1.5, 1.5, &[]);
        // Tight: a straight edge projects exactly, so a corrupted `seg²`/dot
        // (which snaps the result to a sample boundary) shows up here.
        assert!((p - 1.3).abs() < 1e-6, "projects onto edge 1 at 0.3: {p}");
        // Edge 1's length is √113 ≈ 10.63 ft, so the 1 ft snap zone ≈ 0.094. A
        // point at param 0.08 (exact on a straight edge) is inside it → snaps to
        // node 1; a chord mutant that lengthens the edge shrinks the zone and
        // would miss it.
        let near1 = edge_point(&tri, &[], &[], 1, 0.08);
        let p = constrain_seam(&tri, &[], &[], 1, near1, 1.0, 1.5, 0.5, &[]);
        assert!((p - 1.0).abs() < 1e-9, "snaps to node 1: {p}");
        // A blocker at 1.5 with the seam currently *below* it (1.4): `cur_frac =
        // current − e` must use e = 1 (a `−`→`+` flip would read 2.4 and put the
        // seam on the wrong side). Held a gap below → 1 + (0.5 − 1.5/√113) ≈ 1.3589.
        let below = edge_point(&tri, &[], &[], 1, 0.45);
        let held = constrain_seam(&tri, &[], &[], 1, below, 1.0, 1.5, 1.4, &[1.5]);
        let gap = 1.5 / 113.0_f64.sqrt();
        assert!(
            (held - (1.5 - gap)).abs() < 0.01,
            "held a gap below the blocker: {held}"
        );
        // A Bézier edge is projected along its curve (exercises the
        // `bezier_length` branch and the `ce == e` match). Its apex projects to
        // ~0.5, far from the chord midpoint.
        let bez = vec![
            Coord::new(0.0, 0.0),
            Coord::new(10.0, 0.0),
            Coord::new(10.0, 8.0),
            Coord::new(0.0, 8.0),
        ];
        let curves = [(0usize, Point::new(2.0, 6.0), Point::new(8.0, 6.0))];
        let apex = edge_point(&bez, &[], &curves, 0, 0.5);
        let p = constrain_seam(&bez, &[], &curves, 0, apex, 1.0, 1.5, 0.5, &[]);
        assert!(
            (p - 0.5).abs() < 0.05,
            "projects onto the bézier curve: {p}"
        );
        // A degenerate (zero-length) edge 0 collapses to node 0.
        let degen = vec![
            Coord::new(3.0, 3.0),
            Coord::new(3.0, 3.0),
            Coord::new(9.0, 3.0),
        ];
        let p = constrain_seam(
            &degen,
            &[],
            &[],
            0,
            Point::new(4.0, 5.0),
            1.0,
            1.5,
            0.0,
            &[],
        );
        assert!(p.abs() < 1e-9, "degenerate edge 0 → node 0: {p}");
        // An **off-edge** cursor (2 ft perpendicular to edge 1) still projects
        // to the foot of the perpendicular — param 0.4 → 1.4 — exercising the
        // sample-and-project dot product / seg² (an exactly-on-edge point is too
        // forgiving to catch a flipped term).
        let foot = edge_point(&tri, &[], &[], 1, 0.4);
        let inv = 1.0 / 113.0_f64.sqrt();
        let off = Point::new(foot.x + 2.0 * 8.0 * inv, foot.y + 2.0 * 7.0 * inv); // ⟂ to (−7,8)
        let p = constrain_seam(&tri, &[], &[], 1, off, 1.0, 1.5, 1.5, &[]);
        // The perpendicular component drops out of the dot product, so a
        // straight edge still projects exactly to the foot (0.4) — tight.
        assert!(
            (p - 1.4).abs() < 1e-6,
            "off-edge cursor → perpendicular foot: {p}"
        );
        // The Bézier gap is measured along its (longer) arc length, so the
        // `bezier_length` branch matters: a blocker at 0.5 with the seam above
        // holds it `1.5 / bezier_length` back — not the chord's `1.5 / 10`.
        let blen = bezier_length(
            Point::new(0.0, 0.0),
            Point::new(2.0, 6.0),
            Point::new(8.0, 6.0),
            Point::new(10.0, 0.0),
        );
        let low = edge_point(&bez, &[], &curves, 0, 0.4);
        let held = constrain_seam(&bez, &[], &curves, 0, low, 1.0, 1.5, 0.6, &[0.5]);
        assert!(
            (held - (0.5 + 1.5 / blen)).abs() < 0.01,
            "bézier gap along the arc ({blen:.2} ft): {held}"
        );
        // A node-shared blocker (another seam exactly at node 1 = position 1.0)
        // sits at an end (fraction 0) and is skipped, so a seam dragged into node
        // 1's snap zone still snaps there — turning the `<= 1e-9` skip into `==`
        // or `>` would (wrongly) let it constrain and stop short.
        let near_n1 = edge_point(&tri, &[], &[], 1, 0.05);
        let p = constrain_seam(&tri, &[], &[], 1, near_n1, 1.0, 1.5, 1.5, &[1.0]);
        assert!(
            (p - 1.0).abs() < 1e-9,
            "node-shared blocker at node 1 is skipped: {p}"
        );
    }
}

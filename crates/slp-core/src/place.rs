//! The node-placement engine: one small, pure state machine that every drawing
//! tool (house outline, door, window — and later trees, beds, decks) shares.
//!
//! A tool supplies three things, all pure and headless so they unit-test without
//! a browser:
//! 1. [`snap_node`] — where the next node lands given the cursor + nodes so far,
//! 2. [`commit_kind`] — whether a click adds a node, closes, or finishes,
//! 3. [`opening_from_nodes`] — finalize a door/window from its two nodes.
//!
//! The reactive shell (slp-ui) just drives it: on pointer move it previews
//! `snap_node`; on click it acts on `commit_kind`.

use crate::{
    Coord, Opening, OpeningKind, nearest_wall, point_along, project_onto, snap_ortho, snap_to_grid,
};

/// Snap radius (ft): clicking within this of the first node closes the outline.
pub const CLOSE_FT: f64 = 2.0;
/// Standard step rise (ft) — 7 inches.
pub const STEP_RISE_FT: f64 = 7.0 / 12.0;
/// Standard step tread depth (ft) — 11 inches.
pub const STEP_TREAD_FT: f64 = 11.0 / 12.0;

/// Which object the placement tool is currently building.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    /// The house outline: an open chain of corners, closed by clicking the first.
    House,
    /// The deck footprint: a closed outline of corners, drawn like the house.
    Deck,
    /// A door: a two-node span on a wall.
    Door,
    /// A window: a two-node span on a wall.
    Window,
    /// A run of steps: a two-node span on an edge, running outward to grade.
    Steps,
    /// A catalog object placed at a single point: one click drops it at that
    /// position (the object's center).
    Furniture,
}

impl Tool {
    /// The opening kind this tool places, if it places openings.
    #[must_use]
    pub fn opening_kind(self) -> Option<OpeningKind> {
        match self {
            Tool::Door => Some(OpeningKind::door),
            Tool::Window => Some(OpeningKind::window),
            Tool::House | Tool::Deck | Tool::Steps | Tool::Furniture => None,
        }
    }

    /// Whether this tool draws a closed outline (house / deck footprint).
    #[must_use]
    pub fn is_outline(self) -> bool {
        matches!(self, Tool::House | Tool::Deck)
    }

    /// Whether this tool places a two-node span on an existing edge
    /// (door/window on a wall, stair on a deck edge).
    #[must_use]
    pub fn is_span(self) -> bool {
        matches!(self, Tool::Door | Tool::Window | Tool::Steps)
    }

    /// Whether this tool drops an object at a single clicked point (furniture
    /// and, later, point objects like trees or a fire pit).
    #[must_use]
    pub fn is_point(self) -> bool {
        matches!(self, Tool::Furniture)
    }
}

/// The number of steps and total run depth (ft) for a stair dropping `elevation`
/// feet, using standard rise/tread. At least one step.
#[must_use]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn step_run(elevation: f64) -> (u32, f64) {
    // `elevation` is a positive height; the step count is small and bounded.
    let steps = (elevation / STEP_RISE_FT).ceil().max(1.0) as u32;
    (steps, f64::from(steps) * STEP_TREAD_FT)
}

/// The outward unit direction for a stair on edge `a`→`b`: perpendicular to the
/// edge, pointing away from `away_from` (the deck centroid). Zero for a
/// degenerate edge.
#[must_use]
pub fn step_outward(a: &Coord, b: &Coord, away_from: &Coord) -> (f64, f64) {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let len = dx.hypot(dy);
    if len <= 0.0 {
        return (0.0, 0.0);
    }
    let (px, py) = (-dy / len, dx / len);
    let (mx, my) = (a.x.midpoint(b.x), a.y.midpoint(b.y));
    let (tx, ty) = (mx - away_from.x, my - away_from.y);
    if px.mul_add(tx, py * ty) >= 0.0 {
        (px, py)
    } else {
        (-px, -py)
    }
}

/// What a click does, given the nodes placed so far this gesture.
#[derive(Debug, PartialEq, Eq)]
pub enum Commit {
    /// Append the node and keep placing.
    Add,
    /// Finish the object from the nodes already placed (the click is a closing
    /// gesture, not a node) — used to close the house ring.
    Finish,
    /// Append the node, then finish — used for the second node of an opening.
    FinishWith,
}

/// Where the next node lands: snapped to the grid (+ ortho) for the house, or to
/// the nearest wall for an opening (the second node is constrained to the first
/// node's wall). Falls back to the raw point when there is nothing to snap to.
#[must_use]
pub fn snap_node(
    tool: Tool,
    corners: &[Coord],
    placed: &[Coord],
    raw: &Coord,
    grid: bool,
    ortho: bool,
    grid_step: f64,
) -> Coord {
    match tool {
        Tool::House | Tool::Deck => {
            let mut p = if grid {
                snap_to_grid(raw, grid_step)
            } else {
                raw.clone()
            };
            if ortho && let Some(prev) = placed.last() {
                p = snap_ortho(prev, &p);
            }
            p
        }
        Tool::Furniture => {
            if grid {
                snap_to_grid(raw, grid_step)
            } else {
                raw.clone()
            }
        }
        Tool::Door | Tool::Window | Tool::Steps => snap_to_wall(corners, placed, raw),
    }
}

/// Snap `raw` onto a wall: the first node picks the nearest wall, the second is
/// constrained to that same wall (so an opening can't straddle a corner).
fn snap_to_wall(corners: &[Coord], placed: &[Coord], raw: &Coord) -> Coord {
    let n = corners.len();
    if n < 3 {
        return raw.clone();
    }
    let anchor = placed.first().unwrap_or(raw);
    let Some((wall, _, _)) = nearest_wall(corners, anchor) else {
        return raw.clone();
    };
    let (a, b) = (&corners[wall], &corners[(wall + 1) % n]);
    let (offset, _) = project_onto(a, b, raw);
    point_along(a, b, offset)
}

/// Decide what a click on `next` does, given the nodes already `placed`.
#[must_use]
pub fn commit_kind(tool: Tool, placed: &[Coord], next: &Coord) -> Commit {
    match tool {
        Tool::House | Tool::Deck => {
            let near_start = placed
                .first()
                .is_some_and(|c| (c.x - next.x).hypot(c.y - next.y) <= CLOSE_FT);
            if placed.len() >= 3 && near_start {
                Commit::Finish
            } else {
                Commit::Add
            }
        }
        // First click adds the start node; the second finishes the span.
        Tool::Door | Tool::Window | Tool::Steps => {
            if placed.is_empty() {
                Commit::Add
            } else {
                Commit::FinishWith
            }
        }
        // A single click is the whole gesture: drop the object and finish.
        Tool::Furniture => Commit::FinishWith,
    }
}

/// Build an opening from its two nodes: both are projected onto the first node's
/// wall, and the opening spans between them. `None` without a wall to place on.
#[must_use]
pub fn opening_from_nodes(
    corners: &[Coord],
    kind: OpeningKind,
    a: &Coord,
    b: &Coord,
) -> Option<Opening> {
    let n = corners.len();
    if n < 3 {
        return None;
    }
    let (wall, off_a, _) = nearest_wall(corners, a)?;
    let (off_b, _) = project_onto(&corners[wall], &corners[(wall + 1) % n], b);
    let offset = off_a.min(off_b);
    let width = (off_a - off_b).abs();
    Some(Opening::new(
        kind,
        offset,
        i64::try_from(wall).unwrap_or(0),
        width,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square() -> Vec<Coord> {
        vec![
            Coord::new(0.0, 0.0),
            Coord::new(10.0, 0.0),
            Coord::new(10.0, 10.0),
            Coord::new(0.0, 10.0),
        ]
    }

    #[test]
    fn house_node_snaps_to_grid_and_ortho() {
        let prev = [Coord::new(2.0, 2.0)];
        // grid + ortho: round to whole feet, then axis-align to the previous node.
        let p = snap_node(
            Tool::House,
            &[],
            &prev,
            &Coord::new(10.4, 2.6),
            true,
            true,
            1.0,
        );
        assert_eq!(p, Coord::new(10.0, 2.0));
        // grid off, ortho off: passes through.
        let p = snap_node(
            Tool::House,
            &[],
            &prev,
            &Coord::new(10.4, 2.6),
            false,
            false,
            1.0,
        );
        assert_eq!(p, Coord::new(10.4, 2.6));
    }

    #[test]
    fn opening_first_node_snaps_to_nearest_wall() {
        let sq = square();
        // Near the bottom edge → lands on it (y = 0).
        let p = snap_node(
            Tool::Door,
            &sq,
            &[],
            &Coord::new(5.0, 1.0),
            false,
            false,
            1.0,
        );
        assert_eq!(p, Coord::new(5.0, 0.0));
    }

    #[test]
    fn opening_second_node_is_constrained_to_the_first_nodes_wall() {
        let sq = square();
        let placed = [Coord::new(3.0, 0.0)]; // first node on the bottom wall
        // A raw point nearer the right wall is still projected onto the bottom wall.
        let p = snap_node(
            Tool::Door,
            &sq,
            &placed,
            &Coord::new(8.0, 4.0),
            false,
            false,
            1.0,
        );
        assert_eq!(p, Coord::new(8.0, 0.0), "stays on the bottom wall (y=0)");
    }

    #[test]
    fn house_commit_adds_until_the_ring_closes() {
        let three = [
            Coord::new(0.0, 0.0),
            Coord::new(10.0, 0.0),
            Coord::new(10.0, 8.0),
        ];
        assert_eq!(
            commit_kind(Tool::House, &three, &Coord::new(0.4, 0.4)),
            Commit::Finish
        );
        assert_eq!(
            commit_kind(Tool::House, &three, &Coord::new(5.0, 5.0)),
            Commit::Add
        );
        assert_eq!(
            commit_kind(Tool::House, &[], &Coord::new(0.0, 0.0)),
            Commit::Add
        );
    }

    #[test]
    fn opening_commit_adds_then_finishes() {
        assert_eq!(
            commit_kind(Tool::Door, &[], &Coord::new(3.0, 0.0)),
            Commit::Add
        );
        assert_eq!(
            commit_kind(Tool::Door, &[Coord::new(3.0, 0.0)], &Coord::new(6.0, 0.0)),
            Commit::FinishWith
        );
    }

    #[test]
    fn opening_from_two_nodes_spans_between_them() {
        let sq = square();
        let o = opening_from_nodes(
            &sq,
            OpeningKind::door,
            &Coord::new(3.0, 0.0),
            &Coord::new(7.0, 0.0),
        )
        .unwrap();
        assert_eq!(o.wall, 0);
        assert!((o.offset - 3.0).abs() < 1e-9);
        assert!((o.width - 4.0).abs() < 1e-9);
        // Order-independent: same span if the nodes are reversed.
        let o2 = opening_from_nodes(
            &sq,
            OpeningKind::door,
            &Coord::new(7.0, 0.0),
            &Coord::new(3.0, 0.0),
        )
        .unwrap();
        assert!((o2.offset - 3.0).abs() < 1e-9 && (o2.width - 4.0).abs() < 1e-9);
    }

    #[test]
    fn deck_draws_like_the_house_outline() {
        // Same grid/ortho snap and close-on-first-corner behaviour as the house.
        let prev = [Coord::new(2.0, 2.0)];
        assert_eq!(
            snap_node(
                Tool::Deck,
                &[],
                &prev,
                &Coord::new(10.4, 2.6),
                true,
                true,
                1.0
            ),
            Coord::new(10.0, 2.0)
        );
        let three = [
            Coord::new(0.0, 0.0),
            Coord::new(8.0, 0.0),
            Coord::new(8.0, 6.0),
        ];
        assert_eq!(
            commit_kind(Tool::Deck, &three, &Coord::new(0.3, 0.3)),
            Commit::Finish
        );
        assert_eq!(Tool::Deck.opening_kind(), None);
        assert!(Tool::Deck.is_outline());
    }

    #[test]
    fn tool_classification() {
        assert_eq!(Tool::Door.opening_kind(), Some(OpeningKind::door));
        assert_eq!(Tool::Window.opening_kind(), Some(OpeningKind::window));
        assert_eq!(Tool::House.opening_kind(), None);
        assert_eq!(Tool::Furniture.opening_kind(), None);
        assert!(Tool::House.is_outline() && Tool::Deck.is_outline());
        assert!(!Tool::Door.is_outline() && !Tool::Steps.is_outline());
        assert!(Tool::Door.is_span() && Tool::Window.is_span() && Tool::Steps.is_span());
        assert!(!Tool::House.is_span() && !Tool::Deck.is_span());
        // Furniture is a point placement — neither an outline nor a span.
        assert!(Tool::Furniture.is_point());
        assert!(!Tool::Furniture.is_outline() && !Tool::Furniture.is_span());
        assert!(!Tool::House.is_point() && !Tool::Door.is_point());
    }

    #[test]
    fn furniture_is_a_single_click_grid_snapped_placement() {
        // One click drops the object; commit finishes immediately, with no prior
        // nodes needed.
        assert_eq!(
            commit_kind(Tool::Furniture, &[], &Coord::new(5.0, 5.0)),
            Commit::FinishWith
        );
        // Grid on: the drop point rounds to whole feet.
        assert_eq!(
            snap_node(
                Tool::Furniture,
                &[],
                &[],
                &Coord::new(5.4, 7.6),
                true,
                true,
                1.0
            ),
            Coord::new(5.0, 8.0)
        );
        // Grid off: the raw point passes through unchanged (ortho is irrelevant —
        // a point placement has no previous node to align to).
        assert_eq!(
            snap_node(
                Tool::Furniture,
                &[],
                &[],
                &Coord::new(5.4, 7.6),
                false,
                true,
                1.0
            ),
            Coord::new(5.4, 7.6)
        );
    }

    #[test]
    fn step_run_steps_and_run_from_elevation() {
        // 7" rise: a 7" (0.583 ft) drop is one step.
        assert_eq!(step_run(STEP_RISE_FT), (1, STEP_TREAD_FT));
        // 2 ft / (7/12) = 3.43 → 4 steps; run = 4 treads.
        let (steps, run) = step_run(2.0);
        assert_eq!(steps, 4);
        assert!((run - 4.0 * STEP_TREAD_FT).abs() < 1e-9);
        // Absolute run (literal, not via the const) pins the tread depth: a one-
        // step run is one 11" tread ≈ 0.9167 ft.
        assert!((step_run(0.5).1 - 11.0_f64 / 12.0).abs() < 1e-9);
        // A zero/negative drop still yields at least one step.
        assert_eq!(step_run(0.0).0, 1);
    }

    #[test]
    fn step_outward_is_the_unit_perpendicular_away_from_center() {
        // Diagonal edge off the origin (so a.x/a.y matter): (1,2)→(4,6), dx=3,
        // dy=4, len=5 → perpendicular unit is (-0.8, 0.6) or its negation;
        // midpoint (2.5, 4).
        let (a, b) = (Coord::new(1.0, 2.0), Coord::new(4.0, 6.0));
        let keep = |out: (f64, f64)| (out.0 + 0.8).abs() < 1e-9 && (out.1 - 0.6).abs() < 1e-9;
        let flip = |out: (f64, f64)| (out.0 - 0.8).abs() < 1e-9 && (out.1 + 0.6).abs() < 1e-9;

        // Center far in -x (tx>0 dominates) → keep.
        assert!(keep(step_outward(&a, &b, &Coord::new(20.0, 4.0))));
        // Center far in +x → flips (exercises the tx term + the sign branch).
        assert!(flip(step_outward(&a, &b, &Coord::new(-20.0, 4.0))));
        // Center far in +y (ty term dominates) → flips.
        assert!(flip(step_outward(&a, &b, &Coord::new(2.5, 30.0))));
        // Center far in -y → keep.
        assert!(keep(step_outward(&a, &b, &Coord::new(2.5, -20.0))));
        // Sign-boundary: tx=ty=5 → dot = -0.8·5 + 0.6·5 = -1 (<0, flips). If the
        // `py*ty` term were `py+ty` the dot would be +1.6 (kept) — so this kills
        // the *→+ mutant on the dot.
        assert!(flip(step_outward(&a, &b, &Coord::new(-2.5, -1.0))));
    }

    #[test]
    fn opening_needs_a_house() {
        assert!(
            opening_from_nodes(
                &[],
                OpeningKind::window,
                &Coord::new(0.0, 0.0),
                &Coord::new(1.0, 0.0)
            )
            .is_none()
        );
    }

    #[test]
    fn house_close_distance_to_the_first_corner() {
        // Off-origin, asymmetric ring so each subtraction in the distance check
        // matters. First corner (10,20).
        let ring = [
            Coord::new(10.0, 20.0),
            Coord::new(30.0, 20.0),
            Coord::new(30.0, 40.0),
        ];
        // Within the snap radius of the first corner → close.
        assert_eq!(
            commit_kind(Tool::House, &ring, &Coord::new(11.0, 21.0)),
            Commit::Finish
        );
        // Just outside → keep adding. These also pin the subtraction: a `/` in
        // place of either `-` would read these as "close" and wrongly finish.
        assert_eq!(
            commit_kind(Tool::House, &ring, &Coord::new(8.0, 20.5)),
            Commit::Add
        );
        assert_eq!(
            commit_kind(Tool::House, &ring, &Coord::new(10.5, 22.0)),
            Commit::Add
        );
    }

    #[test]
    fn spans_work_on_a_triangle() {
        // Exactly three corners — the `n < 3` boundary in snap_to_wall /
        // opening_from_nodes. A door snaps onto, and spans, the bottom edge.
        let tri = [
            Coord::new(0.0, 0.0),
            Coord::new(10.0, 0.0),
            Coord::new(5.0, 8.0),
        ];
        assert_eq!(
            snap_node(
                Tool::Door,
                &tri,
                &[],
                &Coord::new(5.0, 1.0),
                false,
                false,
                1.0
            ),
            Coord::new(5.0, 0.0)
        );
        let o = opening_from_nodes(
            &tri,
            OpeningKind::door,
            &Coord::new(2.0, 0.0),
            &Coord::new(6.0, 0.0),
        )
        .unwrap();
        assert_eq!(o.wall, 0);
        assert!((o.offset - 2.0).abs() < 1e-9 && (o.width - 4.0).abs() < 1e-9);
    }
}

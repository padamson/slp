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

/// Which object the placement tool is currently building.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    /// The house outline: an open chain of corners, closed by clicking the first.
    House,
    /// A door: a two-node span on a wall.
    Door,
    /// A window: a two-node span on a wall.
    Window,
}

impl Tool {
    /// The opening kind this tool places, if it places openings.
    #[must_use]
    pub fn opening_kind(self) -> Option<OpeningKind> {
        match self {
            Tool::Door => Some(OpeningKind::door),
            Tool::Window => Some(OpeningKind::window),
            Tool::House => None,
        }
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
        Tool::House => {
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
        Tool::Door | Tool::Window => snap_to_wall(corners, placed, raw),
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
        Tool::House => {
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
        Tool::Door | Tool::Window => {
            if placed.is_empty() {
                Commit::Add
            } else {
                Commit::FinishWith
            }
        }
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
}

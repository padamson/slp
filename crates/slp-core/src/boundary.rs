//! Node-editing primitives shared by every drawn boundary (a ring of
//! `Coord`s) — moving a node is just overwriting `corners[i]`, so this module
//! only needs adjacency, insert, and delete. Entity-agnostic: `House`, `Deck`
//! levels, and `Shape` all share the same ring shape, so these free functions
//! work on a plain `&[Coord]` rather than any one of those types.

use crate::Coord;

/// Whether nodes `a` and `b` (indices into a ring of `len` nodes) are
/// consecutive — including the wrap-around pair (`0`, `len - 1`). A node is
/// never adjacent to itself.
#[must_use]
pub fn are_adjacent(len: usize, a: usize, b: usize) -> bool {
    if len == 0 || a == b {
        return false;
    }
    let diff = a.abs_diff(b);
    diff == 1 || diff == len - 1
}

/// Insert a new node at the midpoint of the edge between adjacent nodes `a`
/// and `b`, preserving ring order. `None` if `a`/`b` are out of bounds or
/// aren't adjacent.
#[must_use]
pub fn insert_node_between(corners: &[Coord], a: usize, b: usize) -> Option<Vec<Coord>> {
    let len = corners.len();
    if a >= len || b >= len || !are_adjacent(len, a, b) {
        return None;
    }
    let mid = Coord::new(
        corners[a].x.midpoint(corners[b].x),
        corners[a].y.midpoint(corners[b].y),
    );
    // The wrap-around pair (0, len - 1) inserts after the last node; every
    // other adjacent pair inserts after the earlier of the two.
    let insert_after = if a.abs_diff(b) == 1 {
        a.min(b)
    } else {
        len - 1
    };
    let mut out = corners.to_vec();
    out.insert(insert_after + 1, mid);
    Some(out)
}

/// Remove node `i`, reconnecting its two neighbors directly. `None` if `i` is
/// out of bounds or removing it would leave fewer than 3 nodes (a boundary's
/// drawable minimum).
#[must_use]
pub fn delete_node(corners: &[Coord], i: usize) -> Option<Vec<Coord>> {
    if i >= corners.len() || corners.len() <= 3 {
        return None;
    }
    let mut out = corners.to_vec();
    out.remove(i);
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square() -> Vec<Coord> {
        vec![
            Coord::new(0.0, 0.0),
            Coord::new(4.0, 0.0),
            Coord::new(4.0, 3.0),
            Coord::new(0.0, 3.0),
        ]
    }

    #[test]
    fn consecutive_pairs_are_adjacent() {
        assert!(are_adjacent(4, 0, 1));
        assert!(are_adjacent(4, 1, 2));
        assert!(are_adjacent(4, 2, 3));
    }

    #[test]
    fn the_wraparound_pair_is_adjacent() {
        assert!(are_adjacent(4, 3, 0));
        assert!(are_adjacent(4, 0, 3));
    }

    #[test]
    fn opposite_corners_are_not_adjacent() {
        assert!(!are_adjacent(4, 0, 2));
        assert!(!are_adjacent(4, 1, 3));
    }

    #[test]
    fn a_node_is_never_adjacent_to_itself() {
        assert!(!are_adjacent(4, 2, 2));
    }

    #[test]
    fn an_empty_ring_has_no_adjacency() {
        assert!(!are_adjacent(0, 0, 0));
    }

    #[test]
    fn a_single_node_ring_is_never_self_adjacent() {
        // len - 1 == 0 here, the same value a non-empty diff would use — pins
        // that self-adjacency is refused directly, not just by the diff check
        // (which would otherwise coincidentally read this case as adjacent).
        assert!(!are_adjacent(1, 0, 0));
    }

    #[test]
    fn every_pair_in_a_triangle_is_adjacent() {
        assert!(are_adjacent(3, 0, 1));
        assert!(are_adjacent(3, 1, 2));
        assert!(are_adjacent(3, 0, 2));
    }

    #[test]
    fn inserts_at_the_midpoint_of_an_interior_edge() {
        let out = insert_node_between(&square(), 0, 1).expect("adjacent");
        assert_eq!(out.len(), 5);
        assert_eq!(out[1], Coord::new(2.0, 0.0), "midpoint of (0,0)-(4,0)");
        assert_eq!(
            out[0],
            Coord::new(0.0, 0.0),
            "original nodes keep their order"
        );
        assert_eq!(out[2], Coord::new(4.0, 0.0));
    }

    #[test]
    fn inserts_at_the_end_for_the_wraparound_edge() {
        // The edge from the last node back to the first (3 -> 0) has no
        // "index between them" — the new node lands after the last node.
        let out = insert_node_between(&square(), 3, 0).expect("adjacent");
        assert_eq!(out.len(), 5);
        assert_eq!(out[4], Coord::new(0.0, 1.5), "midpoint of (0,3)-(0,0)");
    }

    #[test]
    fn insert_argument_order_does_not_matter() {
        let forward = insert_node_between(&square(), 0, 1).expect("adjacent");
        let backward = insert_node_between(&square(), 1, 0).expect("adjacent");
        assert_eq!(forward, backward);
    }

    #[test]
    fn refuses_to_insert_between_non_adjacent_nodes() {
        assert_eq!(insert_node_between(&square(), 0, 2), None);
    }

    #[test]
    fn refuses_to_insert_with_an_out_of_bounds_index() {
        assert_eq!(insert_node_between(&square(), 0, 9), None);
    }

    #[test]
    fn refuses_to_insert_when_only_one_index_is_out_of_bounds_even_if_the_diff_looks_adjacent() {
        // b == len (one past the last valid index) has a diff of 1 from a
        // valid a, so it "looks" adjacent by raw index math — pins that a's
        // and b's bounds are each checked independently, not skipped just
        // because the pair looks adjacent.
        assert_eq!(insert_node_between(&square(), 3, 4), None);
    }

    #[test]
    fn deletes_a_node_and_reconnects_its_neighbors() {
        let out = delete_node(&square(), 1).expect("above the minimum");
        assert_eq!(
            out,
            vec![
                Coord::new(0.0, 0.0),
                Coord::new(4.0, 3.0),
                Coord::new(0.0, 3.0)
            ]
        );
    }

    #[test]
    fn refuses_to_delete_below_three_nodes() {
        let triangle = vec![
            Coord::new(0.0, 0.0),
            Coord::new(4.0, 0.0),
            Coord::new(0.0, 3.0),
        ];
        assert_eq!(delete_node(&triangle, 0), None);
    }

    #[test]
    fn refuses_to_delete_an_out_of_bounds_index() {
        assert_eq!(delete_node(&square(), 9), None);
    }
}

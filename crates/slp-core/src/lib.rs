//! Headless, renderer-agnostic core for the Simple Landscape Planner.
//!
//! This crate holds the geometry and material take-off math that both the 2D
//! SVG renderer and the future 3D view consume. It is deliberately WASM-free so
//! it can be unit- and mutation-tested fast on the native target, independent of
//! the UI — keeping the math separate from the DOM.

pub mod arc;
pub mod boundary;
pub mod clearance;
pub mod corner;
pub mod generated;
pub mod geom;
pub mod pick;
pub mod place;
pub mod snap;
pub mod takeoff;
pub mod wall;

pub use arc::{ArcSvg, arc_svg, boundary_area, bulge_radius, segment_area};
pub use boundary::{are_adjacent, delete_node, insert_node_between};
pub use clearance::{circle_overlaps_circle, circle_overlaps_polygon, circle_overlaps_segment};
pub use corner::{Corner, free_corner};
pub use generated::slp::{
    CatalogItem, Circle, Coord, Deck, DeckLevel, FootprintShape, House, ItemStatus, Object,
    Opening, OpeningKind, Plan, Shape, StepRun,
};
pub use geom::{
    Point, area, circle_area, footprint_corners, heading, point_in_polygon, polyline_length,
    within_a_single,
};
pub use pick::object_at;
pub use place::{Commit, Tool, commit_kind, opening_from_nodes, snap_node, step_outward, step_run};
pub use snap::{dragged_center, snap_ortho, snap_to_grid};
pub use takeoff::{BillOfMaterials, LineItem, take_off};
pub use wall::{nearest_wall, opening_segment, point_along, project_onto};

//! Headless, renderer-agnostic core for the Simple Landscape Planner.
//!
//! This crate holds the geometry and material take-off math that both the 2D
//! SVG renderer and the future 3D view consume. It is deliberately WASM-free so
//! it can be unit- and mutation-tested fast on the native target, independent of
//! the UI — keeping the math separate from the DOM.

pub mod arc;
pub mod bezier;
pub mod boundary;
pub mod catalog;
pub mod clearance;
pub mod corner;
pub mod dist;
pub mod generated;
pub mod geom;
pub mod pick;
pub mod place;
pub mod plan_io;
pub mod snap;
pub mod takeoff;
pub mod wall;

pub use arc::{
    ArcSvg, arc_length, arc_svg, boundary_area, boundary_perimeter, boundary_project,
    boundary_span_length, bulge_radius, constrain_seam, seam_edge, segment_area,
};
pub use bezier::{bezier_length, bezier_segment_area};
pub use boundary::{are_adjacent, delete_node, insert_node_between};
pub use catalog::reference_count;
pub use clearance::{circle_overlaps_circle, circle_overlaps_polygon, circle_overlaps_segment};
pub use corner::{Corner, content_points, free_corner};
pub use dist::{
    dist_point_to_polygon, dist_point_to_segment, dist_segment_to_polygon, dist_segment_to_segment,
};
pub use generated::slp::{
    Border, CatalogItem, Circle, Coord, Course, CurveEdge, Deck, DeckLevel, FootprintShape, House,
    ItemStatus, Object, Opening, OpeningKind, Plan, PriceUnit, Shape, StepRun,
};
pub use geom::{
    Point, area, circle_area, footprint_corners, heading, point_in_polygon, polyline_length,
    within_a_single,
};
pub use pick::object_at;
pub use place::{Commit, Tool, commit_kind, opening_from_nodes, snap_node, step_outward, step_run};
pub use plan_io::{DEFAULT_PLAN_STEM, PLAN_EXT, plan_filename};
pub use snap::{dragged_center, snap_ortho, snap_to_grid};
pub use takeoff::{
    BillOfMaterials, DEFAULT_TILE_FT, LineItem, default_courses, take_off, tile_size_ft,
};
pub use wall::{nearest_wall, opening_segment, point_along, project_onto};

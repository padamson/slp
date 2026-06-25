//! Headless, renderer-agnostic core for the Simple Landscape Planner.
//!
//! This crate holds the geometry and material take-off math that both the 2D
//! SVG renderer and the future 3D view consume. It is deliberately WASM-free so
//! it can be unit- and mutation-tested fast on the native target, independent of
//! the UI — keeping the math separate from the DOM.

pub mod generated;
pub mod geom;
pub mod place;
pub mod snap;
pub mod wall;

pub use generated::slp::{Coord, House, Opening, OpeningKind, Plan};
pub use geom::{Point, area, point_in_polygon, polyline_length};
pub use place::{Commit, Tool, commit_kind, opening_from_nodes, snap_node};
pub use snap::{snap_ortho, snap_to_grid};
pub use wall::{nearest_wall, opening_segment, point_along, project_onto};

//! Headless, renderer-agnostic core for the Simple Landscape Planner.
//!
//! This crate holds the geometry and material take-off math that both the 2D
//! SVG renderer and the future 3D view consume. It is deliberately WASM-free so
//! it can be unit- and mutation-tested fast on the native target, independent of
//! the UI — keeping the math separate from the DOM.

pub mod generated;
pub mod geom;

pub use generated::slp::{Coord, House, Plan};
pub use geom::{Point, area, point_in_polygon, polyline_length};

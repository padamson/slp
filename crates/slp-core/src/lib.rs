//! Headless, renderer-agnostic core for the Simple Landscape Planner.
//!
//! This crate holds the geometry and material take-off math that both the 2D
//! SVG renderer and the future 3D view consume. It is deliberately WASM-free so
//! it can be unit- and mutation-tested fast on the native target, independent of
//! the UI — the biggest correctness win over the original HTML/JS spikes, where
//! the math and the DOM were entangled.

pub mod geom;

pub use geom::{Point, area, point_in_polygon, polyline_length};

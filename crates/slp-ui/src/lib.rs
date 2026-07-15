//! Leptos UI components for the Simple Landscape Planner.
//!
//! Runtime-agnostic: this library never enables a Leptos runtime feature. The
//! consumer chooses — `slp-app` selects `csr` (via the `csr` passthrough
//! feature) for the browser; `dokime` renders these components under `ssr` for
//! fast native component tests.
//!
//! Components live in `src/components/` Storybook-style: `<name>.rs` (component),
//! `<name>.stories.rs` (theoria stories, behind the `stories` feature), and
//! `<name>.tests.rs` (dokime tests).

// Coordinate math feeds many f64 -> SVG-attribute conversions; allow the
// pedantic casts crate-wide. `must_use_candidate` is a false positive for
// Leptos `#[component]` fns (the framework consumes the returned view).
#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::must_use_candidate
)]

mod api_key;
mod components;
mod fs_access;
mod plan_file;
mod style;
mod vision;

pub use components::{Grid, Planner, ScaleBar, Transform, Yard, YardControls};

/// All slp-ui components as theoria stories (aggregated for the gallery build).
#[cfg(feature = "stories")]
pub use components::stories as all_stories;

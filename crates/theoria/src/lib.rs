//! A tiny component explorer (gallery) for Leptos.
//!
//! `theoria` (Greek θεωρία, "viewing / contemplation") renders a list of named
//! component previews ("stories") with a sidebar to switch between them, so a
//! component can be developed and eyeballed in isolation — Storybook for Leptos,
//! in miniature.
//!
//! Built component-driven and TDD'd with dokime. Components live in
//! `src/components/` Storybook-style: `<name>.rs` (component),
//! `<name>.stories.rs` (theoria stories, behind the `stories` feature), and
//! `<name>.tests.rs` (dokime tests). theoria's own components double as e2e
//! fixtures; we never nest a `Gallery` inside a `Gallery` (the infinite mirror).
//!
//! Incubated in the slp workspace; slp-agnostic so it can graduate to its own
//! crate later.

// `must_use_candidate` is a false positive for Leptos `#[component]` fns — the
// framework consumes the returned view.
#![allow(clippy::must_use_candidate)]

use leptos::prelude::*;

mod components;

#[cfg(feature = "stories")]
pub use components::stories;
pub use components::{Gallery, StoryNav};

/// One named preview in the gallery.
pub struct Story {
    name: &'static str,
    // `Send + Sync`: Leptos 0.8's reactive closures (the stage re-renders when
    // the selection changes) require their captures to be thread-safe.
    view: Box<dyn Fn() -> AnyView + Send + Sync>,
}

impl Story {
    /// Build a story from a name and a closure that produces the view.
    pub fn new<F, V>(name: &'static str, view_fn: F) -> Self
    where
        F: Fn() -> V + Send + Sync + 'static,
        V: IntoView + 'static,
    {
        Self {
            name,
            view: Box::new(move || view_fn().into_any()),
        }
    }
}

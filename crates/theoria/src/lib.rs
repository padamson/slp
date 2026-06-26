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

// So the `#[story]` macro's `theoria::…` paths (which are written for downstream
// crates) also resolve when theoria uses the macro on itself (e.g. in tests).
extern crate self as theoria;

use leptos::prelude::*;

mod components;

#[cfg(feature = "stories")]
pub use components::stories;
pub use components::{Gallery, StoryNav};
/// The `#[story]` attribute macro (see `theoria-macros`).
pub use theoria_macros::story;

/// A live, editable arg ("knob") backing a control widget. Each variant holds
/// the reactive signal the control reads/writes and the story's view reads.
#[derive(Clone, Copy)]
pub enum ArgControl {
    Bool(RwSignal<bool>),
    Num(RwSignal<f64>),
    Text(RwSignal<String>),
}

/// One named preview in the gallery.
pub struct Story {
    name: &'static str,
    // `Send + Sync`: Leptos 0.8's reactive closures (the stage re-renders when
    // the selection changes) require their captures to be thread-safe.
    view: Box<dyn Fn() -> AnyView + Send + Sync>,
    /// Editable args ("knobs"), in declaration order. Empty for plain stories.
    args: Vec<(&'static str, ArgControl)>,
    /// The story body source, for "show code".
    source: Option<&'static str>,
    /// A Markdown description, for autodocs.
    description: Option<&'static str>,
}

impl Story {
    /// Build a plain story from a name and a closure that produces the view.
    pub fn new<F, V>(name: &'static str, view_fn: F) -> Self
    where
        F: Fn() -> V + Send + Sync + 'static,
        V: IntoView + 'static,
    {
        Self {
            name,
            view: Box::new(move || view_fn().into_any()),
            args: Vec::new(),
            source: None,
            description: None,
        }
    }

    /// Constructor used by the `#[story]` macro — carries args, source, and the
    /// description. Not part of the stable API.
    #[doc(hidden)]
    pub fn __from_macro<F>(
        name: &'static str,
        view_fn: F,
        args: Vec<(&'static str, ArgControl)>,
        source: &'static str,
        description: Option<&'static str>,
    ) -> Self
    where
        F: Fn() -> AnyView + Send + Sync + 'static,
    {
        Self {
            name,
            view: Box::new(view_fn),
            args,
            source: Some(source),
            description,
        }
    }

    /// The story's display name (a `/`-delimited path; see the sidebar tree).
    #[must_use]
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// The editable args ("knobs"), in declaration order.
    #[must_use]
    pub fn args(&self) -> &[(&'static str, ArgControl)] {
        &self.args
    }

    /// The captured body source, if any (for "show code").
    #[must_use]
    pub fn source(&self) -> Option<&'static str> {
        self.source
    }

    /// The Markdown description, if any (for autodocs).
    #[must_use]
    pub fn description(&self) -> Option<&'static str> {
        self.description
    }
}

#[cfg(test)]
mod story_macro_tests {
    use super::*;

    /// A demo widget for the macro test.
    #[story(name = "Demo/Widget", active = true, width = 12.0)]
    fn widget(active: bool, width: f64, label: String) -> impl IntoView {
        // Reference each arg so the body genuinely depends on them.
        view! { <div data-active=active data-width=width>{label}</div> }
    }

    #[test]
    fn story_macro_captures_name_args_source_and_description() {
        let owner = Owner::new();
        owner.with(|| {
            let s = widget();
            assert_eq!(s.name(), "Demo/Widget", "name override");
            assert_eq!(s.description(), Some("A demo widget for the macro test."));

            let args = s.args();
            assert_eq!(args.len(), 3, "one control per typed param");
            assert_eq!(args[0].0, "active");
            assert_eq!(args[1].0, "width");
            assert_eq!(args[2].0, "label");
            assert!(matches!(args[0].1, ArgControl::Bool(_)), "bool → toggle");
            assert!(matches!(args[1].1, ArgControl::Num(_)), "f64 → number");
            assert!(matches!(args[2].1, ArgControl::Text(_)), "String → text");

            // Defaults from the attribute (and Default for the unspecified one).
            if let ArgControl::Bool(b) = args[0].1 {
                assert!(b.get_untracked(), "active default = true");
            }
            if let ArgControl::Num(n) = args[1].1 {
                assert!(
                    (n.get_untracked() - 12.0).abs() < 1e-9,
                    "width default = 12.0"
                );
            }
            if let ArgControl::Text(t) = args[2].1 {
                assert!(t.get_untracked().is_empty(), "label default = \"\"");
            }

            assert!(
                s.source().unwrap().contains("<div"),
                "captured body source: {:?}",
                s.source()
            );
        });
    }
}

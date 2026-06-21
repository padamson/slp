//! Minimal component testing for Leptos.
//!
//! `dokime` (Greek δοκιμή, "a test / proof") renders a Leptos view to an HTML
//! string under a fresh reactive owner so components can be asserted on in fast,
//! native unit tests — no browser, no `wasm-bindgen-test` runner. It is the
//! cheap inner-loop complement to the playwright-rust end-to-end gate.
//!
//! Incubated in the slp workspace and dogfooded against `slp-ui` / `theoria`;
//! kept framework-only (no slp deps) so it can graduate to its own crate. Its
//! capabilities are **pulled by demand** — a new helper lands when an slp (or
//! theoria) component test needs it. See `docs/PLAN.md`.
//!
//! ```
//! use leptos::prelude::*;
//!
//! let html = dokime::render(|| view! { <p>"hello"</p> });
//! assert!(html.contains("hello"));
//! ```

use leptos::prelude::*;
use leptos::tachys::view::RenderHtml;

/// Render a Leptos view to its server-side HTML string.
///
/// Takes a closure so the view (and any signals it creates) is constructed
/// inside the temporary [`Owner`], matching how Leptos renders in production.
#[must_use]
pub fn render<F, V>(view_fn: F) -> String
where
    F: FnOnce() -> V,
    V: RenderHtml,
{
    let owner = Owner::new();
    owner.with(|| view_fn().to_html())
}

/// Number of non-overlapping occurrences of `needle` in `haystack`. Handy for
/// counting rendered elements, e.g. `count(&html, "<line")`.
#[must_use]
pub fn count(haystack: &str, needle: &str) -> usize {
    haystack.matches(needle).count()
}

#[cfg(test)]
mod tests {
    //! dokime testing dokime — the tool proves itself.
    use super::*;

    #[test]
    fn renders_a_view_to_html() {
        let html = render(|| view! { <p>"hello"</p> });
        assert!(html.contains("hello"));
        assert!(html.contains("<p"));
    }

    #[test]
    fn renders_dynamic_text() {
        let name = "world";
        let html = render(move || view! { <span>{name}</span> });
        assert!(html.contains("world"));
    }

    #[test]
    fn count_counts_non_overlapping_occurrences() {
        assert_eq!(count("<line/><line/><line/>", "<line"), 3);
        assert_eq!(count("nothing here", "<line"), 0);
    }
}

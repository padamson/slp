# D1 — Render a component and assert on its markup natively

*Pulled by: slp/theoria needing fast native component tests (no browser).*

## Story

As a Leptos developer, I want to render a component to HTML in a plain `#[test]`
and assert on the result, so that I get a millisecond TDD loop without a browser
or a wasm test runner.

## Vertical slices

- **D1.0 — render**
  - [x] `render(|| view! { … })` returns SSR HTML under a fresh reactive `Owner`
- **D1.1 — count**
  - [x] `count(&html, needle)` for simple structural checks

## Notes / refs

- Tests are plain `#[test]` fns discovered by nextest; dokime adds no attribute
  or runner of its own.
- `count` is substring-based; pull **D4** (selector queries) when that gets
  brittle. dokime sees SSR/initial render only; interaction is the e2e's job.

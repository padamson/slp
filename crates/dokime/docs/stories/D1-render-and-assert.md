# D1 — Render a component and assert on its markup natively

**Status:** ✅ done · **Pulled by:** slp/theoria needing fast native component
tests without a browser.

## Story

As a Leptos developer, I want to render a component to HTML in a plain `#[test]`
and assert on the result, so that I get a millisecond TDD loop without a browser
or a wasm test runner.

## Acceptance criteria

- `render(|| view! { … })` returns the component's server-side HTML string,
  built inside a fresh reactive `Owner` (so signals work).
- A helper to count occurrences for simple structural checks.
- Tests are ordinary `#[test]` fns discovered by `cargo test`/`nextest` — dokime
  adds no attribute or runner of its own.

## Vertical slices

- **1a — `render`** ✅ — `Owner` + `RenderHtml::to_html()`.
- **1b — `count`** ✅ — substring occurrence count (`count(&html, "<line")`).

Both are exercised by real tests in `slp-ui` and `theoria`.

## Notes / refs

- `count` is substring-based and will get brittle for structural/attribute
  assertions; that's the trigger to pull **D4** (CSS-selector `Dom`). Not pulled
  yet — current components are simple enough.
- dokime sees SSR/initial render only; browser interaction is the e2e's job.

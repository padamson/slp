# dokime — plan

Testing-Library-style component testing for Leptos: render a component to HTML
under a reactive owner and assert on it, **natively** (no browser, no
`wasm-bindgen-test`). The fast inner-loop complement to the playwright-rust e2e
gate. Incubated as a crate in the slp workspace; spun out later once stable.

## Demand-driven (the governing rule)

dokime helpers are **pulled by need, not built speculatively**: a new capability
lands only when an **slp or theoria** component test needs it. This backlog is a
holding pen.

## Decisions (locked)

- **Renders via Leptos `ssr`** (`Owner` + `RenderHtml::to_html()`). Native, fast.
- **Not a test runner** — `cargo nextest` / `cargo test` is the runner (the
  Vitest/Jest analog). dokime is the Testing-Library analog: render + assert.
- **Complements, doesn't replace, playwright.** dokime sees the initial/SSR
  render (and signal-driven reactivity); real DOM *events* (clicks) need a
  browser → that's the e2e's job. There is no jsdom for Rust/wasm.
- **Framework-only** (no slp deps); spin out when the API stabilizes.

## Backlog

The story catalog and one sliced doc per story are the **single source of
truth** in [`docs/stories/`](stories/README.md) — this overview does not repeat
them. Stories are pulled on demand (see the rule above); the index marks status
and the "pull when…" trigger for each.

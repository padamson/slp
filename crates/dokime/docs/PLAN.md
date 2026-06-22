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

## Testing-Library parity roadmap

Where dokime should head to approach the JS testing stack's value, prioritized
for a **solo dev unit-testing a few Leptos components**. From a 2026 deep-research
pass over Testing Library + jest-dom docs; priorities/feasibility are engineering
judgment for the Rust/Leptos/WASM context.

**The load-bearing boundary.** dokime tests the **SSR HTML string** (parse +
query); it has no live DOM, no layout/CSS computation, and no event loop. So
JS-testing capabilities split cleanly:

- **Native (dokime's lane — parse the SSR HTML and assert):** element queries by
  CSS selector and by **ARIA role** (implicit roles like `button`/`link`/`heading`
  are computable from the parsed tree); `get`/`query` semantics (throw-on-miss vs
  return-`Option`); and the *static* jest-dom matchers — `in_document`/exists,
  `has_attribute`, `has_class`, `has_text`, `contains_element`, element counts.
- **Browser-only (NOT dokime — belongs in the playwright-rs e2e):** `toBeVisible`
  / `toHaveStyle` (need layout + computed CSS); **user-event/fireEvent**
  interaction; `findBy` async retries (presume a *changing* DOM); and any
  post-hydration reactivity (DOM updates after events). There is no jsdom for
  Rust/wasm, so this genuinely requires a real browser.
- **Runner layer:** `cargo nextest` already is the runner (the Vitest/Jest
  analog); dokime does not and should not provide one.

### Tier 1 — replace brittle substring matching (native)
- **CSS-selector query API** — a parsed `Dom` with `count`/`exists`/`text`/`attr`
  by selector, instead of `count(&html, "<line")`. The foundation everything else
  builds on. (This is backlog **D4**.)
- **jest-dom-style assertions** — ergonomic helpers with readable failures:
  `assert_in_document`, `has_attribute`, `has_class`, `has_text`, `count`. Cheap
  once the `Dom` exists. (Backlog **D3**.)

### Tier 2 — accessibility-aligned queries (native, more effort)
- **Role-based queries** (`by_role(role, name)`) — Testing Library's *top* query
  ("test like a user"); feasible natively from a minimal implicit-role map
  (`button`, `link`, `heading`, `textbox`, `checkbox`, …). Full ARIA
  accessible-name computation is complex — start with common roles. Highest
  *quality* signal (tests resemble real use + nudge accessible markup).

### Tier 3 — nice-to-have (native)
- **Snapshot testing** — golden HTML files with an update flag. (Backlog **D6**.)

### Explicitly out of scope (the e2e layer's job)
Visibility/layout assertions, user-event interaction, `findBy` async, and
post-hydration reactivity — covered by `slp-e2e` / `theoria-e2e` (playwright-rs),
not dokime. dokime stays the fast *render-and-assert-structure* layer.

Sources: testing-library.com/docs (queries/about, byrole), github.com/testing-library/jest-dom.

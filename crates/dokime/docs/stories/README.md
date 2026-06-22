# dokime user stories

`docs/PLAN.md` is the overview. **This folder holds one detailed doc per user
story**, each broken into **vertical slices** — same convention as slp's
`docs/stories/`. A story is fleshed out and built **when pulled** (an slp or
theoria component test needs it), not before.

## Doc template

Four sections only: heading, Story, Vertical slices, Notes/refs. Acceptance
criteria live **inside each slice** as checkboxes; behavior is specified by the
tests in code.

```
# <ID> — <one-line title>
*Pulled by: <which slp/theoria need>.*

## Story
As a <persona>, I want <capability>, so that <value>.

## Vertical slices
- **<ID>.0 — <slice name>**
  - [ ] <acceptance criterion>

## Notes / refs
- <refs, dependencies, decisions>
```

Persona: a Rust/Leptos developer doing component-driven TDD who wants a fast
native inner loop (render → assert in milliseconds), with playwright reserved for
real browser interaction.

## Index

Tiers + the native-vs-browser boundary come from the **Testing-Library parity
roadmap** in [`../PLAN.md`](../PLAN.md). All items are pulled on demand.

**Done**

| # | Story | Doc |
|---|---|---|
| D1 | Render a component and assert on its markup natively | [D1](D1-render-and-assert.md) |

**Backlog** (native — dokime's lane; pull when a test needs it)

| # | Story | Tier |
|---|---|---|
| D4 | Query the SSR output by CSS selector (`Dom`: count/exists/text/attr) | 1 |
| D3 | jest-dom-style assertions (`in_document`/`has_attribute`/`has_class`/`has_text`) with readable failures | 1 |
| D7 | Role-based queries (`by_role(role, name)`) from implicit ARIA roles | 2 |
| D6 | Snapshot testing (golden HTML, update flag) | 3 |
| D5 | Set a signal + re-render + assert (native reactivity, no DOM events) | 3 |

**Out of scope** (browser layer — `slp-e2e` / `theoria-e2e`, not dokime)

Visibility/layout (`toBeVisible`/`toHaveStyle`), user-event interaction,
`findBy` async retries, and post-(DOM)-event reactivity — these need a real
browser, which is the playwright-rs e2e's job.

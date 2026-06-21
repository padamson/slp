# dokime user stories

`docs/PLAN.md` is the overview. **This folder holds one detailed doc per user
story**, each broken into **vertical slices** — same convention as slp's
`docs/stories/`. A story is fleshed out and built **when pulled** (an slp or
theoria component test needs it), not before.

## Doc template

```
# <ID> — <one-line story title>
**Status:** … · **Pulled by:** <which slp/theoria need, or "not yet">
## Story        (As a <persona>, I want <capability>, so that <value>.)
## Acceptance criteria
## Vertical slices    (ID.0, ID.1, … each shippable)
## Notes / refs
```

Persona: a Rust/Leptos developer doing component-driven TDD who wants a fast
native inner loop (render → assert in milliseconds), with playwright reserved for
real browser interaction.

## Index

| # | Story | Doc | Status |
|---|---|---|---|
| D1 | Render a component and assert on its markup natively | [D1](D1-render-and-assert.md) | ✅ done |
| D3 | Ergonomic assertions (`assert_contains`/`assert_count`) with readable failures | _pull when needed_ | backlog |
| D4 | Query the output by CSS selector (`Dom`: count/text/attr) | _pull when substring matching gets brittle_ | backlog |
| D5 | Drive a signal/event, re-render, assert (native reactivity) | _pull when a component test needs interaction_ | backlog |
| D6 | Snapshot testing (golden HTML, update flag) | _pull when needed_ | backlog |

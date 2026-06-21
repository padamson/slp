# theoria — plan

A component explorer (gallery) for Leptos: preview components in isolation via
"stories" — Storybook, in miniature. Incubated as a crate in the slp workspace
and dogfooded against `slp-ui`; spun out to its own repo (via `git subtree
split`) once the API stabilizes.

## Demand-driven (the governing rule)

theoria capabilities are **pulled by need, not built speculatively**: a theoria
story/slice is tackled only when an **slp** slice needs it. (In turn, dokime
grows only when slp *or* theoria needs it.) So this backlog is a holding pen;
items move to "in progress" when slp pulls them.

## Decisions (locked)

- **Leptos component library, runtime-agnostic** (`csr`/`ssr`/`hydrate`
  passthrough features); the consumer picks the runtime — same reason as
  `slp-ui` (so `dokime` can render it under `ssr` and apps under `csr`).
- **Components in `src/components/`**, Storybook-style: `<name>.rs` +
  `<name>.stories.rs` (behind the `stories` feature) + `<name>.tests.rs`
  (dokime).
- **TDD / component-driven:** each component is spec'd with dokime; real browser
  behavior is covered by a playwright e2e via `theoria-demo` (fixtures are
  theoria's *own* components — never a `Gallery` inside a `Gallery`).
- **Spin-out unit:** `theoria` + `theoria-demo` + `theoria-e2e` travel together.

## Backlog

The story catalog and one sliced doc per story are the **single source of
truth** in [`docs/stories/`](stories/README.md) — this overview does not repeat
them. Stories are pulled on demand (see the rule above); the index marks status.

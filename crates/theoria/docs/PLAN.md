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
  behavior is covered by a playwright e2e (`theoria-e2e`) driving a gallery built
  from theoria's *own* components as fixtures — never a `Gallery` inside a
  `Gallery`.
- **Spin-out unit:** `theoria` + `theoria-cli` + `theoria-e2e` travel together.

## Backlog

The story catalog and one sliced doc per story are the **single source of
truth** in [`docs/stories/`](stories/README.md) — this overview does not repeat
them. Stories are pulled on demand (see the rule above); the index marks status.

## Storybook parity roadmap

Where theoria should head to approach Storybook's value, prioritized for a
**solo dev iterating on a handful of Leptos components** (the slp use case).
From a 2026 deep-research pass over Storybook's primary docs (storybook.js.org);
priorities/difficulty are engineering judgment for a Rust/Leptos/WASM port.

**The load-bearing insight:** Storybook's power comes from *automatic*
reflection — it infers `argTypes` from prop types (react-docgen), auto-generates
controls, and auto-discovers stories from ES-module exports. Rust has **no
runtime reflection over `#[component]` props**, so every "auto" in Storybook
becomes **explicit** in theoria (typed arg structs, hand-written argTypes, or a
proc-macro). Prioritize the *value*, accept explicit authoring.

> **Decision (2026-06, pulled forward to help slp design):** go **macro-first**.
> A `#[story]` / `Meta` proc-macro (T5) derives `argTypes` from a story fn's typed
> params and captures the body source for *show code*. Everything below builds on
> it: **Controls (T6)**, **Autodocs (T13)**, **Show code (T16)**. Being built now,
> not deferred. (Earlier open question — dynamic `Vec<(name, ArgValue)>` vs. typed
> — resolved in favor of the macro emitting the arg list + source.)

### Tier 1 — converts the passive gallery into an interactive iteration loop
- **Args + Controls ("knobs")** — live-edit a component's props in the browser
  instead of edit-recompile. *The single highest-leverage feature.* Rust path:
  a small `ArgValue` enum (bool→checkbox, f64→number/slider, string→text,
  enum→select, color→picker) fed through Leptos signals; no auto-inference.
  Open question: typed per-component arg struct (type-safe, needs a derive) vs.
  dynamic `Vec<(name, ArgValue)>` (easy to render, loses type safety).
- **Actions** — log a component's callbacks + their args to a panel ("did my
  `on:click`/`Callback` fire with the right payload?"). Pairs with args (a
  callback is just an arg value); needs `T: Debug`. Build alongside Controls.
- **Meta / CSF parity** — a per-component `Meta` (title, default args, params)
  that controls/docs read from. theoria has named stories but no meta layer;
  optionally a `#[story]` proc-macro for auto-registration when the explicit
  `stories()` Vec gets tedious.

### Tier 2 — cheap, high-value wins
- **Deep-linking** — encode the selected story in the URL (query/hash), not just
  `localStorage`; enables share/bookmark and lets `theoria-e2e` target a story
  by URL. Small extension of the existing persistence.
- **a11y checks** — run axe-core (JS) against each rendered story in the
  playwright e2e; catches a useful subset (~half) of WCAG issues for ~free.
- **Globals / theme toolbar** — a reactive theme signal all stories read + a
  toolbar toggle; genuinely useful for a styled UI.
- **Autodocs (T13)** — a per-component page = a **Markdown-rendered description**
  + args/argTypes table + the controls panel + the live story + show-code; falls
  out of the macro + args work. Markdown prose (headings/lists/code) **is in
  scope**; only the MDX *authoring format* (arbitrary JSX interleaved in Markdown
  pages) is out of scope. **Show code (T16)** renders the source the `#[story]`
  macro captured.

### Tier 3 — polish / already-covered elsewhere
- **Viewport + backgrounds** — preview size presets + background swatch (a
  width wrapper + a CSS var on the stage).
- **Story groups / nesting** — hierarchical sidebar from title paths. ✅ **Done**
  (T8; pulled forward when slp-ui's story list grew). See
  [`docs/stories/T8-story-groups.md`](stories/T8-story-groups.md).
- **Interaction tests (play functions)** — largely covered by dokime (Rust
  component tests) + playwright (e2e); don't duplicate.

### Tier 4 — defer, but promote when rendered fidelity grows
- **Visual regression** (screenshot-diff stories against a baseline). Skip *now*:
  the current visuals are simple geometry you already eyeball in the review
  gallery, and pixel-diffing is flaky across dev (macOS) vs CI (Linux) — fonts,
  antialiasing — so it mostly produces false diffs + baseline churn for little
  signal at ~5 components. **But slp is visualization-first**, so VR's value rises
  with fidelity — *promote it when there's a complex rendered visual worth
  locking*: realistic material textures (epic M) and the 3D view (R2). Pragmatic
  form when pulled: a few *targeted* `to_have_screenshot` checks on specific
  stories, with baselines generated in CI (Linux), not snapshot-everything.
  (Screenshots-as-artifacts — publishing gallery images without a diff gate — are
  cheap today and a separate, lower-commitment option.)

### Already shipped (parity achieved)
Isolation (one component per stage), hot reload (incl. source-crate watch),
selection persistence, and a "every story renders" smoke test.

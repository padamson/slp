# theoria user stories

`docs/PLAN.md` is the overview. **This folder holds one detailed doc per user
story**, each broken into **vertical slices** — same convention as slp's
`docs/stories/`. A story doc is fleshed out **when the story is pulled** (i.e.
when an slp slice needs that theoria capability), not before.

## Doc template

```
# <ID> — <one-line story title>
**Status:** … · **Pulled by:** <which slp/theoria need, or "not yet">
## Story        (As a <persona>, I want <capability>, so that <value>.)
## Acceptance criteria
## Vertical slices    (ID.0, ID.1, … each shippable + how tested: dokime / e2e)
## Notes / refs
```

Persona: a Leptos developer building a component who wants to see and poke it in
isolation while developing it.

## Index

| # | Story | Doc | Status |
|---|---|---|---|
| T1 | Browse components in a sidebar; preview the selected one on a stage | [T1](T1-browse-and-preview.md) | ✅ done |
| T2 | Clicking a name switches the preview; active item highlighted | (T1 slice 1b) | ✅ unit; browser e2e pending |
| T3 | Styled two-pane layout, mountable in a host app | (T1 slice 1c) | ✅ done |
| T4 | Browser e2e: drive the gallery, click a story, assert the stage swaps (fixtures = theoria's own components) | _pull when wiring the demo/e2e_ | backlog |
| T9 | `theoria serve` CLI: discover stories + **generate and serve** the gallery (the real Storybook analog, so consumers don't hand-write a gallery bin) | [T9](T9-serve-cli.md) | ✅ done |
| T5 | `#[story]` attribute macro + auto-registration (ergonomic authoring; needs a `theoria-macros` crate + wasm-safe registry) | _pull when manual aggregation hurts_ | backlog |
| T6 | Per-story prop controls ("knobs") | _pull when needed_ | backlog |
| T7 | Per-story stage options (background, viewport size) | _pull when needed_ | backlog |
| T8 | Story groups / nesting in the sidebar | _pull when needed_ | backlog |

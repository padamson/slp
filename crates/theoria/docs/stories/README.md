# theoria user stories

`docs/PLAN.md` is the overview. **This folder holds one detailed doc per user
story**, each broken into **vertical slices** — same convention as slp's
`docs/stories/`. A story doc is fleshed out **when the story is pulled** (i.e.
when an slp slice needs that theoria capability), not before.

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

Persona: a Leptos developer building a component who wants to see and poke it in
isolation while developing it.

## Index

Backlog priority tiers (and Rust/Leptos difficulty + rationale) come from the
**Storybook parity roadmap** in [`../PLAN.md`](../PLAN.md). Items are still
**pulled on demand** — the tier is "value if/when pulled," not a schedule.

**Done**

| # | Story | Doc |
|---|---|---|
| T1 | Browse + preview (sidebar/stage, click-to-switch, styling, selection persists across reload) | [T1](T1-browse-and-preview.md) |
| T9 | `theoria serve`: discover stories + generate & serve the gallery, with source-crate hot reload | [T9](T9-serve-cli.md) |
| T4 | Browser e2e: every story renders + selection persists (`theoria-e2e`) | (T1 slices) |
| T8 | Story groups / nesting in the sidebar (hierarchical from `/`-delimited names) | [T8](T8-story-groups.md) |

**Pulling forward now** (to help slp component design — macro-first; see PLAN.md)

| # | Story | Status |
|---|---|---|
| T5 | `#[story]` macro + `Meta`: derive argTypes from a story fn's params + capture body source | ✅ done |
| T6 | **Args + Controls ("knobs")** — live-edit a component's props in the gallery | ✅ done |
| T16 | **Show code** — render the source the `#[story]` macro captured | ✅ done |
| T13 | **Autodocs** — per-component page: Markdown description + args table + controls + live story + show-code | ✅ done |

**Backlog** (pull when an slp slice needs it)

| # | Story | Tier |
|---|---|---|
| T10 | **Actions** — log a component's callbacks + args to a panel | 1 |
| T14 | Deep-linking — per-story URL (share/bookmark; drive e2e by URL) | 2 |
| T11 | a11y — run axe-core against each story in the e2e | 2 |
| T12 | Globals/toolbar — theme switch all stories read | 2 |
| T7 | Viewport + backgrounds (stage size / bg presets) | 3 |
| T15 | Visual regression (screenshot-diff) — promote when material textures (epic M) / 3D (R2) land | 4 (defer) |

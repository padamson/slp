# H1 — Draw the house outline + place doors & windows

*Epic H — Structures · drawn and saved, never hardcoded.*

## Story

As a DIY homeowner, I want to draw my house's outline and mark its doors and
windows, so that I can plan the yard around the real building (entries,
sightlines) — for **any** house, since it's drawn and saved, not baked in.

## Vertical slices

- **H1.0 — house renders to scale** ✅
  - [x] `Plan` carries an optional `House` (a closed outline of corner points)
  - [x] a `House` component draws the outline inside `Yard`, to scale
  - [x] a plan with no house renders nothing extra (no empty outline)
- **H1.1 — draw the outline** ✅
  - [x] click in the yard to drop corners; close the loop to finish the outline
        (toggle "Draw house"; snap-to-close by clicking near the first corner)
  - [x] the drawn house is saved to the `Plan` and survives a reload
  - [ ] the house is flagged **existing** by default (not costed) — deferred to
        the cost milestone (no cost engine yet; YAGNI)
- **H1.2 — place doors & windows** *(composable components)*
  - [ ] refactor `House` to compose one `Wall` component per edge; each `Wall`
        hosts its `Door`/`Window` openings (own `.stories.rs` + `.tests.rs`)
  - [ ] pick a wall and place a door or window on it (offset along wall + width)
  - [ ] doors/windows render as marks/gaps on their wall, to scale
  - [ ] they are saved in the `Plan` and survive a reload

## Notes / refs

- The house outline is an ordered list of corners (a closed polygon); each
  **wall** is the edge between consecutive corners. Doors/windows reference a
  wall by index plus an offset + width along it (H1.2).
- **Composition (component-driven):** `House` = outline + a `Wall` per edge; each
  `Wall` composes its `Door`/`Window` openings. Storage stays flat — corners are
  the source of truth and openings are a list keyed by wall index — while the UI
  composes `Wall`/`Door`/`Window` components (each filtered to its wall). So the
  plan-file has no corner/wall duplication, yet the render tree is fully
  composable.
- Schema lives in `schema/slp.yaml` (panschema → `slp_core`). `House` is a normal
  plan entity — see the existing/virtual model in `docs/PLAN.md` §4.
- **Dogfood watch:** H1.0 is static render (dokime). H1.1 introduces *click-to-draw*
  interaction — if dokime/theoria lack what we need to drive/preview pointer
  interaction, pull the smallest dokime/theoria slice that unblocks it (per the
  demand-driven rule), don't build it speculatively.

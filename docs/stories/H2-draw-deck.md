# H2 — Draw the deck

*Epic H — Structures · drawn and saved, never hardcoded. Reuses the
[node-placement engine](H1-draw-house.md) (`slp_core::place`).*

## Story

As a DIY homeowner, I want to draw my deck/patio footprint, so that I can plan
the yard around it — for **any** deck, since it's drawn and saved, not baked in.

## Vertical slices

- **H2.0 — deck footprint renders to scale** ✅
  - [x] `Plan` carries an optional `Deck` (a closed outline of corner points)
  - [x] a `Deck` component draws the footprint polygon inside `Yard`, to scale
- **H2.1 — draw the footprint** ✅
  - [x] a `Draw deck` tool draws a closed outline via the shared engine
        (`Tool::Deck` — same grid/ortho snap + close-on-first-corner as the house)
  - [x] the drawn deck is saved to the `Plan` and survives a reload
- **H2.2a — multiple levels** ✅
  - [x] `Deck` is a list of `DeckLevel { corners, elevation }`; "Draw deck" adds
        a level at the current **Elev (ft)** input (decks are additive)
  - [x] levels render stacked (lowest first) with an elevation label; persist
- **H2.2b — steps** ✅
  - [x] click two points on a deck edge → a step run extends outward (`Tool::Steps`)
  - [x] **step count + run computed from the level's elevation** (standard
        rise/tread) and rendered as treads; persist
  - [x] modeled as a reusable `StepRun` + standalone `Steps` component (no
        railings) — same primitive will serve **house steps** out of a door
  - [x] *(today's assumption, generalized by H2.3)* the drop is always to grade
        (elevation `0.0`) — fine for a deck standing alone in the yard
- **H2.3 — steps target any area, or grade** *(generalizes H2.2b; enables house
  steps)*
  - [ ] a step run captures **both ends' elevation at draw time** —
        `from_elevation` (the edge it's drawn from) and `to_elevation`
        (`0.0`/grade by default, or whichever area's elevation is found at the
        run's outward end: another deck level, the house — using
        [H1.4](H1-draw-house.md)'s new elevation — or, once
        [F3](F3-draw-edit-shapes.md) lands, a paver area)
  - [ ] rise/tread math uses `from − to` in place of today's fixed-to-grade
        `elevation`; H2.2b's grade case is just `to_elevation = 0.0`, unchanged
        in behavior
  - [ ] **house steps**: place a `Steps` run on a house wall exactly like a
        deck edge, using the house's elevation as `from_elevation`
  - [ ] captured once at draw time, not live — editing an area's elevation
        later doesn't retroactively change steps already drawn to/from it
        (same limitation H2.2b already has for its single deck level)
  - [ ] e2e: steps from a deck to grade (today's case, unchanged); steps from a
        deck to the house use the house's elevation instead
- **H2.4 — railing** — _deferred (optional add-on)_
- existing flag (not costed) — _deferred to the cost milestone (YAGNI; no cost
  engine yet), same as the house outline_

Needed now because **furniture placement** depends on the deck's surfaces
(levels at their elevations) and where stairs consume usable space.

## Notes / refs

- The deck is a closed polygon, drawn exactly like the house outline — adding it
  was a new `Tool::Deck` variant (grouped with `Tool::House` in `snap_node` /
  `commit_kind`) + a `Deck` render component + Planner wiring, **not** a new
  interaction. This is the payoff of the [engine](H1-draw-house.md).
- Schema: `Plan.deck : Deck { corners }` (panschema → `slp_core::Deck`).
- The `Placement` overlay is reused for the in-progress outline; `Deck` draws
  only the committed footprint.
- **H2.3's schema change:** `StepRun.elevation` (today, one field, implicitly
  "drop to grade") becomes `from_elevation` + `to_elevation`. No format-
  migration story exists yet (no released `.slp.json` consumers before
  [G1](G1-save-load-plan.md) — see G1's note that versioning is out of scope
  for its own v1), so this is a plain field change, not an additive one.
- **The paver-area target case depends on [F3](F3-draw-edit-shapes.md)** (paver
  shapes don't exist until then); deck-to-house and deck-to-deck-level don't —
  H2.3 can land ahead of F3 for those cases.

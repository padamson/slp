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
- **H2.2 — stairs, railing, multi-level** — _deferred (future slice)_
- existing flag (not costed) — _deferred to the cost milestone (YAGNI; no cost
  engine yet), same as the house outline_

## Notes / refs

- The deck is a closed polygon, drawn exactly like the house outline — adding it
  was a new `Tool::Deck` variant (grouped with `Tool::House` in `snap_node` /
  `commit_kind`) + a `Deck` render component + Planner wiring, **not** a new
  interaction. This is the payoff of the [engine](H1-draw-house.md).
- Schema: `Plan.deck : Deck { corners }` (panschema → `slp_core::Deck`).
- The `Placement` overlay is reused for the in-progress outline; `Deck` draws
  only the committed footprint.

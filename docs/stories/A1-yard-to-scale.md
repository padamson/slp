# A1 — Yard at dimensions I set, with house + deck reference

**Epic:** A — Canvas · **Layer:** cross-cutting · **Status:** in progress
(A1.0 done in Milestone 0; A1.1–A1.3 in Milestone 1)

## Story

As a DIY homeowner, I want to see my backyard drawn to scale **at dimensions I
set**, with the house wall and the existing deck shown for reference, so that I'm
planning against the real space.

## Acceptance criteria

- Yard renders to scale (feet → px) with a foot grid and a scale bar.
- I can set yard **width** and **depth** in feet; the canvas updates live.
- The house back wall (with its bump-out) is drawn along the bottom.
- The existing deck is drawn for reference and can be slid left/right (`deckX`)
  to match its real position.
- Reasonable bounds: width/depth can't go below sane minimums.

## Vertical slices

- **A1.0 — static yard** ✅ *(Milestone 0)* — ground rect + grid + scale bar render;
  `slp-ui::Yard/Grid/ScaleBar` components; playwright e2e asserts the SVG mounts.
- **A1.1 — editable dimensions** — yard W/D become a `Plan` signal driven by
  number inputs; `Yard` re-renders reactively. *Tests:* dokime asserts grid line
  count scales with W/D; e2e changes a dimension and asserts the viewBox/last grid
  line moved.
- **A1.2 — house reference** — back wall + bump-out polygon (from spike `HW`
  constants, recreated). *Tests:* dokime asserts the wall polyline renders.
- **A1.3 — deck reference** — existing deck polygon + stairs + railing, draggable
  `deckX`. *Tests:* dokime renders deck; e2e drags it and asserts it moved.

## Notes / refs

- Geometry constants (`HW`, `DECK`, stair/railing layout) come from the spike
  (gitignored `spike/Paver Planner (interactive).html`) — recreate as typed
  constants in `slp-ui`/`slp-core`, not by importing the spike.
- A1.1 introduces the first real `Plan` state; coordinate with the schema work
  (LinkML `Plan { yard {w,d}, deck_x }`) at the start of Milestone 1.

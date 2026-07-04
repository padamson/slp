# SLP user stories

`docs/PLAN.md` is the overview (decisions, backlog table, delivery order). **This
folder holds one detailed document per user story**, where the story is broken
into **vertical slices** (thin, shippable increments — each a path through
schema → core → render → interact → estimate → test).

A story doc is fleshed out **when its slice is scheduled** (per "develop the
detail as we go"); until then it may be a stub. The `docs/PLAN.md` §6 delivery
order says *which* stories ship together in a release; each story doc says *how*
that story is decomposed.

## Doc template

Four sections only: heading, Story, Vertical slices, Notes/refs. Acceptance
criteria live **inside each slice** as checkboxes, ticked as they land. Behavior
is specified by the tests in code — don't restate it here.

```
# <ID> — <one-line title>
*<epic / layer> — one-line context.*

## Story
As a <persona>, I want <capability>, so that <value>.

## Vertical slices
- **<ID>.0 — <slice name>**
  - [ ] <acceptance criterion>
  - [ ] <acceptance criterion>
- **<ID>.1 — <slice name>**
  - [ ] <acceptance criterion>

## Notes / refs
- <refs, dependencies, decisions>
```

## Index

Listed in **delivery order** (see `docs/PLAN.md` §6). Items marked *enabler* are
cross-cutting machinery folded into that point in the sequence.

| # | Story | Doc | Status |
|---|---|---|---|
| — | Walking skeleton (yard renders, WASM boots, e2e + CD) | (Milestone 0) | ✅ done |
| A1 | Yard at the dimensions I set | [A1](A1-yard-to-scale.md) | ✅ done |
| H1 | Draw the house outline + place doors & windows (saved in the plan) | [H1](H1-draw-house.md) | ✅ done (H1.3 snap granularity next) |
| H2 | Draw the deck (footprint, stairs, railing); flag existing | [H2](H2-draw-deck.md) | ✅ footprint (stairs/railing later) |
| F1 | *Enabler:* select / move / reshape / delete | [F1](F1-select-move-delete.md) | 🚧 object move + delete done; geometry select/move/reshape next |
| F2 | *Enabler:* place objects from a palette (click-to-place, modifiers, preview ghost) | [F2](F2-object-palette.md) | ✅ done |
| G1 | *Enabler:* save/load `.slp.json` | _doc pending_ | backlog |
| M1–M3 | *Enabler:* materials/catalog + cost engine (folded into E1) | [E1](E1-place-furniture.md) | ✅ cost engine (headless) |
| E1 | Place deck furniture (look + cost) | [E1](E1-place-furniture.md) | ✅ place + estimate + select/inspect/rotate + status/legend + e2e |
| D2 | Fire pit | [D2](D2-fire-pit.md) | ✅ done |
| D1 | Trees | _doc pending_ | backlog |
| B4 | Draw mulch beds; mulch volume/cost | _doc pending_ | backlog |
| D3 | Bushes / shrubs | _doc pending_ | backlog |
| B1 | Draw a paver area by clicking corners | [B1](B1-draw-paver-area.md) | backlog |
| B2 | See an area's ft² + material cost | [B2](B2-area-cost.md) | backlog |
| D4 | Grill | _doc pending_ | backlog |
| D5 | Hot tub | _doc pending_ | backlog |
| M4–M5 | Material ingestion, swap-&-compare | _doc pending_ | backlog |
| C1 | Walls / edging / steps | _doc pending_ | backlog |
| B5 | Soldier-course border | _doc pending_ | backlog |
| E2 | Deck seating / presets | _doc pending_ | backlog |
| G2 | Print | _doc pending_ | backlog |
| R1–R3 | 2D / 3D view / 3D designer | _doc pending_ | backlog |

(Everything the user places — yard, **house**, **deck**, pavers, beds, walls,
steps, trees, equipment, furniture — is drawn and saved; nothing is hardcoded to
a specific property.)

The dogfood sub-projects follow this same convention in their own trees:
`crates/dokime/docs/{PLAN.md,stories/}` and
`crates/theoria/docs/{PLAN.md,stories/}`. Their stories are **pulled by slp's
needs** (see each crate's PLAN.md), not built speculatively.

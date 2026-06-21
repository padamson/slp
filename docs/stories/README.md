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

```
# <ID> — <one-line story title>
**Epic:** … · **Layer:** yard / deck / cross-cutting · **Status:** …
## Story        (As a <persona>, I want <capability>, so that <value>.)
## Acceptance criteria   (bulleted, testable)
## Vertical slices        (ID.0, ID.1, … each independently shippable + how tested)
## Notes / refs           (spike constants, dependencies, open questions)
```

Tests per slice: `slp-core` unit tests for math, **dokime** component tests for
new `slp-ui` components, **playwright-rust** e2e for user-visible behavior.

## Index

| # | Story | Doc | Status |
|---|---|---|---|
| — | Walking skeleton (yard renders, WASM boots, e2e + CD) | (Milestone 0) | ✅ done |
| A1 | Yard at dimensions I set + house/deck reference | [A1](A1-yard-to-scale.md) | in progress |
| B1 | Draw a paver area by clicking corners | [B1](B1-draw-paver-area.md) | planned (Milestone 1) |
| B2 | See an area's ft² + material cost | [B2](B2-area-cost.md) | planned (Milestone 1) |
| B4 | Draw mulch beds; mulch volume/cost | _doc pending_ | backlog |
| B5 | Soldier-course border | _doc pending_ | backlog |
| M1–M5 | Materials catalog, ingestion, swap-&-compare | _doc pending_ | backlog |
| C1 | Walls / edging / steps | _doc pending_ | backlog |
| D1 | Trees + equipment | _doc pending_ | backlog |
| E1–E2 | Deck layer: furniture, seating, presets | _doc pending_ | backlog |
| F1 | Select / move / reshape / delete | _doc pending_ | backlog |
| G1–G2 | Save/load `.slp.json`; print | _doc pending_ | backlog |
| R1–R3 | 2D / 3D view / 3D designer | _doc pending_ | backlog |

The dogfood sub-projects follow this same convention in their own trees:
`crates/dokime/docs/{PLAN.md,stories/}` and
`crates/theoria/docs/{PLAN.md,stories/}`. Their stories are **pulled by slp's
needs** (see each crate's PLAN.md), not built speculatively.

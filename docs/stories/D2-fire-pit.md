# D2 — Fire pit (round object + safety clearance)

*Epic D — placed catalog objects. The first object *after* furniture, so it
reuses everything E1 built (catalog, placement, inspector, estimate, status/
virtual) and adds only the **fire-pit-specific geometry**: a **round** footprint
and a **safety clearance** zone — the thing a DIYer most needs to get right about
a fire pit is "will it fit *and* is it a safe distance from the furniture, deck
railing, and house?".*

## Story

As a DIY homeowner, I want to place a fire pit — round, to scale, with its
recommended clearance shown — so that I can budget for it and check it sits a
safe distance from combustibles before I buy.

## Vertical slices

- **D2.0 — round catalog objects + a fire pit** *(enabler + first round object)* ✅ *done (placement e2e deferred to the palette slice, F2)*
  - [x] schema: `CatalogItem` gains a `shape` (`rectangle` default | `circle`);
        a `circle` uses `width_ft` as its **diameter**
  - [x] `Furnishings` renders a circular footprint (a `<circle>`) for round
        items — reusing the same fill / status line-style / selection / overflow
        treatment as rectangles (an `existing` fire pit is a double *ring*, a
        `virtual` one is dashed, …); the fit-check + hit-test use the circle's
        bounding square (a conservative approximation — noted, tightened later
        only if it bites)
  - [x] a fire pit is seeded into the starter catalog (category `fire-pit`,
        round, priced); it places, costs, selects, moves, rotates, and toggles
        status exactly like furniture — no new machinery
  - [x] the inspector shows a round item's size as a **diameter** (`⌀ N ft`),
        not `W × D`
  - [x] dokime: a circular item renders a `<circle>`, not a `<rect>` (+ an
        existing round item is a double ring); the inspector shows `⌀ N ft`
  - [ ] e2e: place the fire pit → a round footprint renders + the estimate
        updates — *deferred to F2 so it drives the new palette flow, not the
        dropdown+button about to be replaced*
- **D2.1 — safety clearance ring** *(the fire-pit value-add)*
  - [ ] schema: `CatalogItem` gains `clearance_ft` — the recommended clear
        radius *beyond* the footprint edge (a fire-pit product/guideline value)
  - [ ] a fire pit draws a dashed **clearance ring** at `radius + clearance_ft`,
        so the keep-clear zone is visible to scale
  - [ ] `slp-core`: a headless intrusion check — any *other* object's footprint
        (or a structure edge) overlapping the ring is flagged; unit + mutation
        tested
  - [ ] intruding objects/edges are highlighted (or the ring turns red), so an
        unsafe layout is obvious at a glance; e2e covers a clear vs. an intruded
        layout
  - [ ] a legend entry for the clearance ring

## Notes / refs

- **No new placement machinery.** D2 is "a new catalog category + item-specific
  geometry", exactly as PLAN.md §6 frames items 4–11. Placement, the inspector,
  the estimate, move/delete, and the status/virtual model all come from E1/F1
  unchanged.
- **Circle bounds = its bounding square.** The fit-check (`within_a_single`) and
  hit-test (`object_at`) reuse `footprint_corners` with `depth = width` for a
  circle, so a round item is treated as its bounding square. Conservative (it
  can read "doesn't fit" in a tight corner where the circle actually clears);
  acceptable for v1, revisited only if it matters in practice.
- **Clearance is a guideline, carried per catalog item.** Real fire pits cite a
  clearance (commonly ~10 ft from structures, ~3 ft from furniture). We carry a
  single recommended `clearance_ft` on the catalog item now; a from-what-kind
  distinction (structure vs. furniture) is a later refinement if needed.
- **Reuses the shared style module** (`slp-ui/src/style.rs`) for the ring/fill
  so the canvas and legend stay in sync (same pattern as E1.5/E1.6).

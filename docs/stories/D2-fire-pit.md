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

- **D2.0 — round catalog objects + a fire pit** *(enabler + first round object)* ✅ *done*
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
  - [x] e2e: place the fire pit from the palette → a round footprint renders +
        the estimate updates (landed with F2.0, which drives placement now)
- **D2.1 — safety clearance ring** *(the fire-pit value-add)* ✅ *done*
  - [x] schema: `CatalogItem` gains `clearance_ft` — the recommended clear
        radius *beyond* the footprint edge (a fire-pit product/guideline value)
  - [x] a fire pit draws a dashed **clearance ring** at `radius + clearance_ft`,
        so the keep-clear zone is visible to scale — always shown, not just
        when selected, since it's a safety check
  - [x] `slp-core` (`clearance.rs`): a headless intrusion check —
        `circle_overlaps_circle`/`_segment`/`_polygon` — any *other* object's
        footprint or a structure edge (house **and** deck) overlapping the ring
        is flagged; unit + mutation tested (0 missed)
  - [x] the ring turns red the instant something intrudes (reusing the
        overflow-red convention); the intruding object/edge itself isn't
        separately highlighted — the ring alone carries the signal
  - [x] a legend entry for the clearance ring (a dashed, unfilled ring icon —
        the first legend entry that isn't a filled shape)
  - [x] e2e covers a clear layout, then a nearby object, a nearby deck edge,
        and a nearby house wall each independently tripping the ring red

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
- **Nothing is allowed inside the stay-out zone, full stop.** The clearance
  check treats a deck edge exactly like a house wall or another object's
  footprint — even the deck the fire pit is standing on. A fire pit on a deck
  too shallow for its own keep-clear zone genuinely should read as unsafe; that
  surprised us in testing but is the intended behavior, not a bug to work around.
- **Default clearance = the fire pit's own radius** (so the total stay-out
  radius is 2x its radius) — a starting guideline, not a fixed constant:
  `clearance_ft` is a plain per-catalog-item field, so a specific fire pit
  product can carry any value once catalog authoring lands.
- **Reuses the shared style module** (`slp-ui/src/style.rs`) for the ring/fill
  so the canvas and legend stay in sync (same pattern as E1.5/E1.6).

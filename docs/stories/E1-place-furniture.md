# E1 — Place deck furniture (look + cost)

*Epic E — Objects placed from the catalog. Folds in the **M1–M3** enabler
(materials/catalog + cost engine): furniture is the first costed catalog object,
so the catalog + take-off machinery lands with it. Reuses the
[node-placement engine](H1-draw-house.md) for placement.*

## Story

As a DIY homeowner, I want to place furniture from a catalog onto my plan and see
its footprint to scale and its cost, so that I can decide what to buy — for **any**
product, since the catalog and placements are saved in the plan, not hardcoded.

## Vertical slices

- **E1.0 — catalog + cost engine (headless, `slp-core`)** ✅ *done*
  - [x] schema grows a `CatalogItem` (id, name, category, unit_price, footprint
        `width_ft`/`depth_ft`, `height_ft`) and an `Object` (catalog_ref, x, y,
        rot, status), wired into `Plan` as `catalog[]` + `objects[]`; generated
        into `slp-core` (the `virtual` status escapes to `r#virtual`, wire name
        unchanged)
  - [x] `ItemStatus` enum (planned / existing / virtual); take-off counts only
        **planned** (excludes existing + virtual), per the domain rule
  - [x] `takeoff::take_off(&Plan)` returns a bill of materials (per catalog item:
        qty, unit_price, line total) + grand total — a pure fn, unit + mutation
        tested; unresolved `catalog_ref`s are excluded
- **E1.1 — place + render furniture** ✅ *done*
  - [x] a furniture `Tool` places an `Object` at a clicked point (catalog item
        chosen from a picker); the footprint renders to scale inside `Yard`
  - [x] the placement is saved to the `Plan` and survives a reload
  - [x] an object whose footprint isn't fully inside a single deck/paver surface
        is outlined in red, so it's obvious what doesn't fit (paver surfaces join
        the check when that slice lands)
- **E1.2 — estimate panel** ✅ *done*
  - [x] a side panel shows the BOM (line items + total), reacting live as
        furniture is placed/removed
- **E1.3 — select + inspect + rotate + e2e** ✅ *done*
  - [x] click a placed object to select it (`slp_core::pick::object_at` hit-tests
        topmost footprint); the selected object shows a selection tint
  - [x] a floating **object inspector** appears in the first empty yard corner
        (priority NE → NW → SE → SW, fallback NE — `slp_core::corner::free_corner`
        over the placed points; corner measured against the grid's rendered
        screen-rect, computed once per mount/resize as `CanvasMetrics`)
  - [x] the inspector shows the object's metadata + a **planned/existing/virtual**
        status toggle; existing/virtual drop it from the estimate
  - [x] a **drag-to-rotate handle** on the selected object turns it (free rotation;
        snaps to 15° when "Snap to grid" is on) via `slp_core::geom::heading`;
        a **Reset** button zeroes the rotation
  - [x] e2e: inspector hops through all four corners by the placement rules;
        dragging the handle east rotates the object to 90°
- **E1.4 — visually distinguish status on the canvas** ✅ *done*
  - [x] an **existing** object renders with a dashed outline + reduced fill
        opacity — visibly "already there, not a purchase" without opening the
        inspector
  - [x] a **virtual** object renders as a lighter **ghost** (dashed, more
        transparent than existing) — visibly a what-if duplicate, per the
        domain rule (PLAN.md's "not a second real item")
  - [x] a **planned** object's look is unchanged (today's solid fill/stroke) —
        no regression for the common case
  - [x] the selection tint and overflow (red) outline still take precedence, so
        a selected or overflowing existing/virtual object stays legible
  - [x] dokime tests per status; e2e: toggle an object's status in the
        inspector and assert its canvas markup changes accordingly
- **E1.5 — a legend for the plan's visual conventions** ✅ *done (superseded by
  E1.6 — the legend's icons will be redrawn under the real/virtual ×
  planned/existing model, but the shared-style architecture stands)*
  - [x] a **shared style module** (`slp-ui/src/style.rs`) is the single source
        of truth for each plan entity's color, fill, and outline —
        `Furnishings`, `House`, `Deck`, `Wall`, `Steps` and the new `Legend`
        all read from it, so a style change (e.g. the existing/virtual dash
        pattern) updates the canvas and the legend together, from one edit
  - [x] a **Legend** renders along the bottom strip, starting to the right of
        the scale bar: one small icon + label per entity — House, Deck,
        Furniture (planned / existing / virtual), Selected, Doesn't fit —
        so color, fill (solid vs. ghost), outline (solid vs. dashed), and
        corner style (square footprint vs. node-marked outline) are all
        readable at a glance without selecting anything
  - [x] dokime tests assert each entry's icon matches its live canvas styling
        (same source, so this mostly guards the wiring, not the values)
  - [x] e2e: the legend renders, one entry per convention, and sits to the
        right of the scale bar's rendered end
- **E1.6 — real/virtual × planned/existing (domain model rework)**
  - [ ] schema: `ItemStatus` narrows to `{planned, existing}` (drop the
        `virtual` value); `Object` gains `is_virtual: bool` (default `false`) —
        virtual is an orthogonal "what-if ghost" flag, not a third status
  - [ ] schema: `House` and `DeckLevel` gain the same (real-only) `ItemStatus`,
        so a structure can be flagged **planned** (an addition you intend to
        build) vs. **existing** (already there) — no virtual variant, since a
        structure is always real; cost math for structures stays a future
        slice (no per-sqft pricing exists yet — the flag just carries now)
  - [ ] `takeoff::take_off`: virtual objects are always excluded (never a real
        purchase); among real objects, only `planned` counts — same net
        behavior as today, derived from two fields instead of one enum
  - [ ] canvas + legend line-style vocabulary (color = what it is, line =
        status): **solid** = real, **dashed** = virtual; **single** thin line =
        planned, **double** thin line = existing — so existing-real = double
        solid, planned-real = single solid, planned-virtual = single dashed,
        existing-virtual = double dashed; structures render real-only (single
        or double solid, never dashed)
  - [ ] inspector UI: the current single 3-way planned/existing/virtual button
        row becomes two independent controls — an existing/planned toggle and
        a real/virtual toggle
  - [ ] dokime + e2e updated for the new controls and line styles; mutation
        tests re-run over the changed `slp-core` schema/take-off logic

## Notes / refs

- **Catalog lives in the plan** (`Plan.catalog[]`); objects reference items by
  `id`. Keeps `.slp.json` self-contained (save/load just works) and honors
  "nothing hardcoded". The committed `materials/manifest.toml` + `slp-ingest`
  become an *import-into-the-plan's-catalog* path later (M4–M5), not a competing
  source of truth.
- **Starter catalog:** a small furniture catalog is seeded into the plan the
  first time a deck is drawn (the surface furniture sits on) — plan data the user
  can place, ignore, or replace, like the default yard size, not hardcoded
  geometry. Catalog *authoring* (add / edit / import) is its own later slice.
- **Cost model:** furniture is **count × unit_price**. Later categories add
  area/volume pricing (pavers ft², mulch yd³) — same BOM/total shape, new
  per-category math in `slp-core`.
- Footprint dims + `height_ft` are carried now (unused by cost) so the 2D render
  and the future 3D view need no schema churn — 2.5D from the start.
- Enablers F1 (select/move/delete) and G1 (save/load `.slp.json`) are sequenced
  near here in PLAN §6 but are separable; placement reuses the existing
  reload-persistence the house/deck already use.
- **`status` default:** schema-encoded — the `status` slot declares
  `ifabsent: ItemStatus(planned)`, so an absent status deserializes to `planned`
  from a single source of truth (`status` is a non-`Option` field with a serde
  default; the cost engine needs no None-means-planned convention).

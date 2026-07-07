# B4 — Draw mulch beds; mulch volume & cost

*Epic B — Hardscape/planting areas · yard layer. First to need
[F3](F3-draw-edit-shapes.md) (freeform boundary drawing + editing, incl. arcs
and curves — pulled forward for this story's own curved/circular bed designs).
Reuses F3's boundary (drawn) and circle (round bed) shapes unchanged; B4 adds
only what's mulch-specific: the bed's material/depth and the volume it costs.*

## Story

As a DIY homeowner, I want to draw a mulch bed — straight-edged, curved, or a
plain circle — and see how many cubic yards of mulch it needs and what that
costs, so that I can budget the beds I'm actually planning, shaped the way I
actually want them.

## Vertical slices

- **B4.0 — a mulch bed is a costed area** ✅ *done*
  - [x] a drawn area (boundary or circle) is tagged with the armed **material**
        via `Shape`/`Circle.material_ref` (mulch by default — the only area
        material so far); the "Area" toolbar group is now "Mulch bed" (Draw
        bed / Round bed, same `draw-shape`/`draw-circle` tools). A material
        *picker* joins the group when pavers/gravel land ([B1](B1-draw-paver-area.md))
  - [x] it renders filled in a **mulch color** — the fill is resolved from the
        material's catalog category (`mulch-bed` → bark brown), so pavers etc.
        get their own look for free later
  - [x] a **depth** (in) is set on the bed via a toolbar field (default 3 in),
        stored as `Shape`/`Circle.depth_in`
  - [x] `slp-core::takeoff` reports the bed's volume — `yd³ = ft²·depth_in/324`
        — and its cost from the mulch material's `unit_price` (per yd³, via the
        new `CatalogItem.price_unit`); a per-ft² path is in place for pavers
        too. Unit + mutation tested (0 missed on `takeoff.rs`)
  - [x] the estimate panel shows a **Mulch** line reading its quantity in yd³
        (not a bare count) alongside every object line, in the grand total
- **B4.1 — live editing** ✅ *done (reactive); reshape-of-arc/curve caveat*
  - [x] moving a bed's boundary nodes ([F3.1](F3-draw-edit-shapes.md))
        recomputes its area → volume → cost live (the estimate derives from the
        reactive `shapes`/`circles` signals). Inserting/deleting a node resets
        the bed's arc/curve edges to straight (F3.1/F3.2/F3.3 interim) but
        still recomputes cost
  - [x] editing the bed's depth recomputes volume/cost live
  - [x] e2e: draw a mulch bed, set its depth, confirm the estimate's Mulch line
        (yd³ × $) and a non-zero total; persists across a reload

## Notes / refs

- **All boundary drawing/editing (straight, arc, bezier, circle) is
  [F3](F3-draw-edit-shapes.md), not repeated here.** B4 is "place one more
  shape category + cost it by volume" — the same shape the [B1](B1-draw-paver-area.md)
  paver story places, priced differently.
- **Volume, not area, is the costed quantity** — the first area-based story
  where the take-off needs a **depth** input alongside the shape's area, unlike
  [B1](B1-draw-paver-area.md)'s flat per-ft² paver cost. Confirms the
  `price_unit` schema field ([M4](M4-M5-material-ingestion.md)) needs to
  distinguish "per ft²" from "per yd³ at a given depth," not just tag a unit
  string.
- **A circular or curved bed costs exactly like a rectangular one** — the
  take-off only reads the shape's area (from F3) and multiplies by depth; nothing
  volume/cost-side cares which F3 shape/edge-kinds produced that area.

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

- **B4.0 — a mulch bed is a costed area**
  - [ ] the mulch tool places an [F3](F3-draw-edit-shapes.md) boundary or
        circle, tagged category `mulch-bed`; it renders filled in a mulch color
  - [ ] a **depth** (in) is set on the bed (a sensible default, e.g. 3 in,
        editable)
  - [ ] `slp-core::takeoff` reports the bed's volume — `yd³ = ft²·depth_in/324`
        (the same formula already noted for gravel/sand in
        [B2](B2-area-cost.md)) — and its cost from a mulch `unit_price` per
        yd³; unit + mutation tested
  - [ ] the estimate panel shows a mulch line (yd³ × $/yd³) alongside every
        other category
- **B4.1 — live editing**
  - [ ] moving/inserting/deleting a bed's boundary nodes ([F3.1](F3-draw-edit-shapes.md))
        recomputes its area, volume, and cost live
  - [ ] editing the bed's depth recomputes volume/cost live
  - [ ] e2e: draw a mulch bed, set its depth, confirm the estimate's mulch line;
        reshape the bed and confirm the line updates

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

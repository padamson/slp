# B1 — Draw a paver area by clicking corners

*Epic B — Hardscape areas · yard layer. Boundary drawing/editing (straight
edges, arcs, curves, circles, snapping, cancel/keyboard) is
[F3](F3-draw-edit-shapes.md), pulled by [B4](B4-draw-mulch-beds.md) and reused
unchanged here. B1 is "place that shape as a paver area" — the paver-specific
look + which category it costs as.*

## Story

As a DIY homeowner, I want to draw a paver patio — straight-edged, curved, or a
circle — so that I can lay out exactly where pavers go before costing it
([B2](B2-area-cost.md)).

## Vertical slices

- **B1.0 — a paver area is a placed shape**
  - [ ] the paver tool places an [F3](F3-draw-edit-shapes.md) boundary or
        circle, tagged category `paver`; it renders filled in a paver look
        (distinct from a mulch bed's)
  - [ ] the finished area persists in the plan and survives a reload

## Notes / refs

- Pointer→feet is the inverse of `slp-ui::Transform`.
- Depends on A1.1 (reactive state) — extend it to a plan/shape list.
- Editing finished shapes (move/insert/delete a node) is **F3.1**, not here.
- Costing (ft² × $/ft²) is **B2**, not here — B1 only places + renders the shape.

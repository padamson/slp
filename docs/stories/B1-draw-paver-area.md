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

- **B1.0 — a paver area is a placed shape** ✅ *done*
  - [x] the "Area" toolbar group gained a **material picker** ([B4](B4-draw-mulch-beds.md)
        laid the groundwork with mulch); arming **Pavers** tags the next drawn
        [F3](F3-draw-edit-shapes.md) boundary or circle with `material_ref`
        → the `paver` catalog category
  - [x] it renders filled in a **paver look** — a cool stone gray, resolved
        from the material's category (distinct from a mulch bed's brown),
        via the same `area_style` path mulch uses
  - [x] the finished area persists in the plan and survives a reload
  - [x] costing falls out of [B4](B4-draw-mulch-beds.md)'s take-off for free:
        the seeded paver material is `price_unit: per_square_foot`, so the
        estimate already shows a **Pavers** line reading ft² × $/ft². (The
        gravel/sand sub-lines and live depth settings are still [B2](B2-area-cost.md).)

## Notes / refs

- Pointer→feet is the inverse of `slp-ui::Transform`.
- Depends on A1.1 (reactive state) — extend it to a plan/shape list.
- Editing finished shapes (move/insert/delete a node) is **F3.1**, not here.
- Costing (ft² × $/ft²) is **B2**, not here — B1 only places + renders the shape.
  (In practice the per-ft² line falls out of B4's generalized take-off the
  moment a priced paver material exists; B2's remaining work is the gravel +
  bedding-sand breakdown and live depth/price settings.)

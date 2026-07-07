# B2 — See an area's ft² and material cost

*Epic B — Hardscape areas · yard layer. The area itself comes from
[F3](F3-draw-edit-shapes.md) (a boundary or circle, any mix of straight/arc/
curved edges) — B2 only costs it.*

## Story

As a DIY homeowner, I want each paver area to show its square footage and material
cost (pavers + base gravel + bedding sand), so that I can budget the project and
decide what to buy.

## Vertical slices

- **B2.0 — area readout**
  - [x] each area shows its area (ft²) on the canvas
- **B2.1 — paver cost line**
  - [x] an estimate panel shows pavers (area × $/ft²) and a grand total
- **B2.2 — gravel + sand**
  - [ ] base + bedding volume + cost lines, using yd³ = ft²·in/324
- **B2.3 — live settings**
  - [x] selecting an area floats an inspector (material · ft² · cost) beside the
        canvas — the area counterpart of the object inspector, per the "metadata
        panels for all areas" ask
  - [x] editing depth/elevation in that panel recomputes the area's cost and the
        estimate live; Remove deletes the area

## Notes / refs

- Money/volume math lives in `slp-core::takeoff` (headless, unit-tested) — never
  in components.
- `unit_price` comes from the seeded `materials/manifest.toml`; the full catalog
  is epic M.
- **Per-measure pricing (`price_unit`) is where this differs from every object
  costed so far.** Furniture/fire pit/trees are priced *per item*
  (`qty × unit_price`); a paver area is priced *per ft²* (and gravel/sand *per
  yd³*) — a different `take_off` shape, not just a new category. `price_unit`
  (per-item vs per-ft²/yd³/linear-ft) is one of the fields
  [M4](M4-M5-material-ingestion.md) adds to the catalog schema.
  [B4](B4-draw-mulch-beds.md) (mulch beds, earlier in delivery order) is
  actually the first story to need the yd³-by-depth case read; B2 reuses that,
  adding the plain per-ft² case for pavers themselves.
- **`AreaInspector`** (`slp-ui`) is the region counterpart of `ObjectInspector`:
  it floats in the first empty yard corner when a shape, circle, the house, or a
  deck level is selected. For a **drawn area** it resolves the material through
  the catalog for name/category/cost and hosts B2.3's live depth/elevation edits;
  in **structure mode** (house / deck level) it swaps material+cost for a
  build-status control (existing/planned) — with elevation for a deck level, none
  for the grade-level house. Either can be removed. This closes the "metadata
  panels for all areas (house, deck, pavers, mulch, …)" ask.
- The inspector's corner placement dodges *every* placed/drawn feature via
  `slp_core::content_points` (house, deck, objects, shape vertices, circle
  center+rim), so a panel never floats over content.

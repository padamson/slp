# B2 — See an area's ft² and material cost

*Epic B — Hardscape areas · yard layer.*

## Story

As a DIY homeowner, I want each paver area to show its square footage and material
cost (pavers + base gravel + bedding sand), so that I can budget the project and
decide what to buy.

## Vertical slices

- **B2.0 — area readout**
  - [ ] each area shows its area (ft²) on the canvas
- **B2.1 — paver cost line**
  - [ ] an estimate panel shows pavers (area × $/ft²) and a grand total
- **B2.2 — gravel + sand**
  - [ ] base + bedding volume + cost lines, using yd³ = ft²·in/324
- **B2.3 — live settings**
  - [ ] editing depths/prices recomputes the estimate

## Notes / refs

- Money/volume math lives in `slp-core::takeoff` (headless, unit-tested) — never
  in components.
- `unit_price` comes from the seeded `materials/manifest.toml`; the full catalog
  is epic M.
- **This is where per-measure pricing (`price_unit`) first bites.** Every
  catalog item costed so far (furniture, fire pit, trees) is priced *per item*
  (`qty × unit_price` in `slp-core::takeoff`). A paver area is priced *per ft²*
  (and gravel/sand *per yd³*) — a different `take_off` shape, not just a new
  category. `price_unit` (per-item vs per-ft²/yd³/linear-ft) is one of the
  fields [M4](M4-M5-material-ingestion.md) adds to the catalog schema; B2 is
  the first story that actually needs it read.

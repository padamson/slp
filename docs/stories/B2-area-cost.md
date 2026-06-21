# B2 — See an area's ft² and material cost

**Epic:** B — Hardscape areas · **Layer:** yard · **Status:** planned (Milestone 1)

## Story

As a DIY homeowner, I want each paver area to show its square footage and material
cost (pavers + base gravel + bedding sand), so that I can budget the project and
decide what to buy.

## Acceptance criteria

- Each area displays its area (ft²) on the canvas.
- An estimate panel lists, per line item: pavers (ft² × $/ft²), base gravel
  (yd³ × $/yd³), bedding sand (yd³ × $/yd³), with a grand total.
- Volumes use the spike rule `yd³ = ft² × depth_in / 324`.
- Editing depths/prices (settings) updates the estimate live.

## Vertical slices

- **B2.0 — area readout** — compute polygon area in `slp-core` (already have
  `area()`); show `NN ft²` label on the shape. *Tests:* `slp-core` area tests
  (done); dokime asserts the label renders.
- **B2.1 — paver cost line** — estimate panel with pavers line (area × default
  material `unit_price`) + grand total. *Tests:* `slp-core` take-off unit test;
  dokime asserts the panel shows the dollar figure.
- **B2.2 — gravel + sand** — add base/bedding volume + cost lines from depth
  settings via `yd³ = ft²·in/324`. *Tests:* `slp-core::takeoff` unit tests for
  the cubic-yard math and rollup.
- **B2.3 — live settings** — editable depths/prices recompute the estimate.
  *Tests:* e2e edits a price and asserts the total changes.

## Notes / refs

- This is the first real `slp-core::takeoff` module — keep all money/volume math
  here (headless, unit-tested), never in components.
- Material `unit_price` comes from the (seeded) `materials/manifest.toml`; full
  catalog/ingestion is epic M (later). For Milestone 1 a single seeded paver material
  is enough.
- Hot-tub "separate total" and other categories are out of scope for B2.
